use std::{net::SocketAddr, path::PathBuf, sync::OnceLock};

use clap::Parser;
use eyre::{Context, Result};

static SERVER_ENDPOINT: OnceLock<String> = OnceLock::new();

#[derive(Debug, Parser)]
struct ListParams {
    #[clap(long)]
    offset: Option<u32>,

    #[clap(long)]
    limit: Option<u32>,
}

#[derive(Debug, Parser)]
struct Args {
    #[clap(long, default_value = "http://localhost:3000", env = "SONAR_SERVER")]
    server: String,

    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Parser)]
enum Command {
    Artist(ArtistArgs),
    Server(ServerArgs),
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    color_eyre::install()?;

    let args = Args::parse();

    SERVER_ENDPOINT.set(args.server.clone()).unwrap();

    match args.command {
        Command::Artist(cargs) => match cargs.command {
            ArtistCommand::List(cargs) => cmd_artist_list(cargs).await?,
            ArtistCommand::Create(cargs) => cmd_artist_create(cargs).await?,
            ArtistCommand::Update(cargs) => cmd_artist_update(cargs).await?,
            ArtistCommand::Delete(cargs) => cmd_artist_delete(cargs).await?,
        },
        Command::Server(cargs) => cmd_server(cargs).await?,
    }

    Ok(())
}

async fn create_client() -> Result<sonar_grpc::Client> {
    let endpoint = SERVER_ENDPOINT.get().unwrap();
    sonar_grpc::client(&endpoint)
        .await
        .with_context(|| format!("connecting to grpc server at {}", endpoint))
}

#[derive(Debug, Parser)]
struct ArtistArgs {
    #[clap(subcommand)]
    command: ArtistCommand,
}

#[derive(Debug, Parser)]
enum ArtistCommand {
    List(ArtistListArgs),
    Create(ArtistCreateArgs),
    Update(ArtistUpdateArgs),
    Delete(ArtistDeleteArgs),
}

#[derive(Debug, Parser)]
struct ArtistListArgs {
    #[clap(flatten)]
    params: ListParams,
}

async fn cmd_artist_list(args: ArtistListArgs) -> Result<()> {
    let mut client = create_client().await?;
    let response = client
        .artist_list(sonar_grpc::ArtistListRequest {
            offset: args.params.offset,
            count: args.params.limit,
        })
        .await?;
    for artist in response.into_inner().artists {
        println!("{:?}", artist);
    }
    Ok(())
}

#[derive(Debug, Parser)]
struct ArtistCreateArgs {
    name: String,

    #[clap(long)]
    cover_art: Option<PathBuf>,

    #[clap(long, default_value = "")]
    genres: sonar::Genres,
}

async fn cmd_artist_create(args: ArtistCreateArgs) -> Result<()> {
    let mut client = create_client().await?;
    let image_id = match args.cover_art {
        Some(path) => {
            tracing::info!("uploading cover art from {}", path.display());
            let content = tokio::fs::read(path).await.context("reading cover art")?;
            let response = client
                .image_create(sonar_grpc::ImageCreateRequest { content })
                .await
                .context("uploading image")?;
            Some(response.into_inner().image_id)
        }
        None => None,
    };

    let response = client
        .artist_create(sonar_grpc::ArtistCreateRequest {
            name: args.name,
            coverart: image_id,
            genres: args.genres.into(),
            ..Default::default()
        })
        .await?;
    println!("{:?}", response.into_inner());

    Ok(())
}

#[derive(Debug, Parser)]
struct ArtistUpdateArgs {}

async fn cmd_artist_update(args: ArtistUpdateArgs) -> Result<()> {
    todo!()
}

#[derive(Debug, Parser)]
struct ArtistDeleteArgs {}

async fn cmd_artist_delete(args: ArtistDeleteArgs) -> Result<()> {
    todo!()
}

#[derive(Debug, Parser)]
struct ServerArgs {
    #[clap(long, default_value = "0.0.0.0:3000", env = "SONAR_ADDRESS")]
    address: SocketAddr,

    #[clap(
        long,
        default_value = "0.0.0.0:3001",
        env = "SONAR_OPENSUBSONIC_ADDRESS"
    )]
    address_opensubsonic: SocketAddr,

    #[clap(long, default_value = ".", env = "SONAR_DATA_DIR")]
    data_dir: PathBuf,
}

async fn cmd_server(args: ServerArgs) -> Result<()> {
    let data_dir = args
        .data_dir
        .canonicalize()
        .context("canonicalizing data dir")?;
    let storage_dir = data_dir.join("storage");
    let database_url = format!("sqlite://{}?mode=rwc", data_dir.join("sonar.db").display());

    tracing::info!("starting sonar server");
    tracing::info!("\taddress: {}", args.address);
    tracing::info!("\tdatabase: {}", database_url);
    tracing::info!("\tstorage: {}", storage_dir.display());

    let storage_backend = sonar::StorageBackend::Filesystem {
        path: data_dir.join("storage"),
    };
    let config = sonar::Config::new(database_url, storage_backend);
    let context = sonar::new(config).await.context("creating sonar context")?;

    let grpc_context = context.clone();
    let f0 = tokio::spawn(async move {
        sonar_grpc::start_server(grpc_context, args.address)
            .await
            .context("starting grpc server")?;
        Ok::<(), eyre::Report>(())
    });

    let opensubsonic_context = context.clone();
    let f1 = tokio::spawn(async move {
        sonar_opensubsonic::start_server(args.address_opensubsonic, opensubsonic_context)
            .await
            .context("starting opensubsonic server")?;
        Ok::<(), eyre::Report>(())
    });

    let (r0, r1) = tokio::try_join!(f0, f1)?;
    r0?;
    r1?;

    Ok(())
}
