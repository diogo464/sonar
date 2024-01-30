use std::{collections::HashSet, net::SocketAddr, path::PathBuf, sync::OnceLock};

use clap::Parser;
use eyre::{Context, Result};
use sonar::UserId;
use tokio_stream::StreamExt;

const IMPORT_FILETYPES: &[&str] = &["flac", "mp3", "ogg", "opus", "wav"];
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
    User(UserArgs),
    Artist(ArtistArgs),
    Album(AlbumArgs),
    //Track(TrackArgs),
    Import(ImportArgs),
    Metadata(MetadataArgs),
    Server(ServerArgs),
    Extractor(ExtractorArgs),
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    color_eyre::install()?;

    let args = Args::parse();

    SERVER_ENDPOINT.set(args.server.clone()).unwrap();

    match args.command {
        Command::User(cargs) => match cargs.command {
            UserCommand::List(cargs) => cmd_user_list(cargs).await?,
            UserCommand::Create(cargs) => cmd_user_create(cargs).await?,
            UserCommand::Update(cargs) => cmd_user_update(cargs).await?,
            UserCommand::Delete(cargs) => cmd_user_delete(cargs).await?,
        },
        Command::Artist(cargs) => match cargs.command {
            ArtistCommand::List(cargs) => cmd_artist_list(cargs).await?,
            ArtistCommand::Create(cargs) => cmd_artist_create(cargs).await?,
            ArtistCommand::Update(cargs) => cmd_artist_update(cargs).await?,
            ArtistCommand::Delete(cargs) => cmd_artist_delete(cargs).await?,
        },
        Command::Album(cargs) => match cargs.command {
            AlbumCommand::List(cargs) => cmd_album_list(cargs).await?,
            AlbumCommand::Create(cargs) => cmd_album_create(cargs).await?,
            AlbumCommand::Update(cargs) => cmd_album_update(cargs).await?,
            AlbumCommand::Delete(cargs) => cmd_album_delete(cargs).await?,
        },
        Command::Import(cargs) => cmd_import(cargs).await?,
        Command::Metadata(margs) => match margs.command {
            MetadataCommand::Album(cargs) => cmd_metadata_album(cargs, margs.view).await?,
            MetadataCommand::AlbumTracks(cargs) => {
                cmd_metadata_album_tracks(cargs, margs.view).await?
            }
        },
        Command::Server(cargs) => cmd_server(cargs).await?,
        Command::Extractor(cargs) => cmd_extractor(cargs).await?,
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
struct UserArgs {
    #[clap(subcommand)]
    command: UserCommand,
}

#[derive(Debug, Parser)]
enum UserCommand {
    List(UserListArgs),
    Create(UserCreateArgs),
    Update(UserUpdateArgs),
    Delete(UserDeleteArgs),
}

#[derive(Debug, Parser)]
struct UserListArgs {
    #[clap(flatten)]
    params: ListParams,
}

async fn cmd_user_list(args: UserListArgs) -> Result<()> {
    let mut client = create_client().await?;
    let response = client
        .user_list(sonar_grpc::UserListRequest {
            offset: args.params.offset,
            count: args.params.limit,
        })
        .await?;
    for user in response.into_inner().users {
        println!("{:?}", user);
    }
    Ok(())
}

#[derive(Debug, Parser)]
struct UserCreateArgs {
    username: String,

    password: String,

    #[clap(long)]
    avatar: Option<PathBuf>,
}

async fn cmd_user_create(args: UserCreateArgs) -> Result<()> {
    let mut client = create_client().await?;
    let image_id = match args.avatar {
        Some(path) => {
            tracing::info!("uploading avatar from {}", path.display());
            let content = tokio::fs::read(path).await.context("reading avatar")?;
            let response = client
                .image_create(sonar_grpc::ImageCreateRequest { content })
                .await
                .context("uploading image")?;
            Some(response.into_inner().image_id)
        }
        None => None,
    };

    let response = client
        .user_create(sonar_grpc::UserCreateRequest {
            username: args.username,
            password: args.password,
            avatar: image_id,
        })
        .await?;
    println!("{:?}", response.into_inner());

    Ok(())
}

#[derive(Debug, Parser)]
struct UserUpdateArgs {}

async fn cmd_user_update(_args: UserUpdateArgs) -> Result<()> {
    todo!()
}

#[derive(Debug, Parser)]
struct UserDeleteArgs {
    id: UserId,
}

async fn cmd_user_delete(args: UserDeleteArgs) -> Result<()> {
    let mut client = create_client().await?;
    let response = client
        .user_delete(sonar_grpc::UserDeleteRequest {
            user_id: From::from(args.id),
        })
        .await?;
    println!("{:?}", response.into_inner());
    Ok(())
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
struct AlbumArgs {
    #[clap(subcommand)]
    command: AlbumCommand,
}

#[derive(Debug, Parser)]
enum AlbumCommand {
    List(AlbumListArgs),
    Create(AlbumCreateArgs),
    Update(AlbumUpdateArgs),
    Delete(AlbumDeleteArgs),
}

#[derive(Debug, Parser)]
struct AlbumListArgs {
    #[clap(flatten)]
    params: ListParams,
}

async fn cmd_album_list(args: AlbumListArgs) -> Result<()> {
    let mut client = create_client().await?;
    let response = client
        .album_list(sonar_grpc::AlbumListRequest {
            offset: args.params.offset,
            count: args.params.limit,
        })
        .await?;
    for album in response.into_inner().albums {
        let album_id = sonar::AlbumId::try_from(album.id)?;
        let album_name = album.name;
        println!("{}\t{}", album_id, album_name);
    }
    Ok(())
}

#[derive(Debug, Parser)]
struct AlbumCreateArgs {
    artist_id: sonar::ArtistId,

    name: String,

    #[clap(long)]
    cover_art: Option<PathBuf>,

    #[clap(long, default_value = "")]
    genres: sonar::Genres,
}

async fn cmd_album_create(args: AlbumCreateArgs) -> Result<()> {
    todo!()
}

#[derive(Debug, Parser)]
struct AlbumUpdateArgs {}

async fn cmd_album_update(args: AlbumUpdateArgs) -> Result<()> {
    todo!()
}

#[derive(Debug, Parser)]
struct AlbumDeleteArgs {}

async fn cmd_album_delete(args: AlbumDeleteArgs) -> Result<()> {
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

#[derive(Debug, Parser)]
struct ImportArgs {
    paths: Vec<PathBuf>,
}

async fn cmd_import(args: ImportArgs) -> Result<()> {
    let mut client = create_client().await?;

    tracing::info!("scanning for files");
    let files = list_files_in_directories(args.paths)
        .await?
        .into_iter()
        .filter(|path| {
            IMPORT_FILETYPES
                .iter()
                .any(|filetype| path.extension().map(|x| x == *filetype).unwrap_or(false))
        });

    for filepath in files {
        tracing::info!("importing {}", filepath.display());
        let file = tokio::fs::File::open(&filepath).await?;
        let reader = tokio::io::BufReader::new(file);
        let stream =
            tokio_util::io::ReaderStream::new(reader).map(move |x| sonar_grpc::ImportRequest {
                chunk: x.unwrap().to_vec(),
                filepath: Some(filepath.display().to_string()),
                artist_id: None,
                album_id: None,
            });
        let response = client.import(stream).await?;
        let track = response.into_inner();
        println!("{:?}", track);
    }

    Ok(())
}

#[derive(Debug, Parser)]
struct MetadataArgs {
    #[clap(long)]
    view: bool,

    #[clap(subcommand)]
    command: MetadataCommand,
}

#[derive(Debug, Parser)]
enum MetadataCommand {
    Album(MetadataAlbumArgs),
    AlbumTracks(MetadataAlbumTracksArgs),
}

#[derive(Debug, Parser)]
struct MetadataAlbumArgs {
    album_id: sonar::AlbumId,
}

async fn cmd_metadata_album(args: MetadataAlbumArgs, view: bool) -> Result<()> {
    let mut client = create_client().await?;
    let _response = client
        .metadata_fetch(sonar_grpc::MetadataFetchRequest {
            kind: sonar_grpc::MetadataFetchKind::Album as i32,
            item_id: From::from(args.album_id),
        })
        .await?;
    Ok(())
}

#[derive(Debug, Parser)]
struct MetadataAlbumTracksArgs {
    album_id: sonar::AlbumId,
}

async fn cmd_metadata_album_tracks(args: MetadataAlbumTracksArgs, view: bool) -> Result<()> {
    let mut client = create_client().await?;
    if view {
        let response = client
            .metadata_album_tracks(sonar_grpc::MetadataAlbumTracksRequest {
                album_id: From::from(args.album_id),
            })
            .await?;
        for track in response.into_inner().tracks {
            let track_id = sonar::TrackId::try_from(track.0)?;
            let metadata = track.1;
            println!("Track: {}", track_id);
            println!("\tName: {:?}", metadata.name);
            for property in metadata.properties {
                println!("\t{}: {}", property.key, property.value);
            }
            if let Some(cover) = metadata.cover {
                println!("\tCover: {} bytes", cover.len());
            }
        }
    } else {
        let _response = client
            .metadata_fetch(sonar_grpc::MetadataFetchRequest {
                kind: sonar_grpc::MetadataFetchKind::Albumtracks as i32,
                item_id: From::from(args.album_id),
            })
            .await?;
    }
    Ok(())
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
    let mut config = sonar::Config::new(database_url, storage_backend);
    config
        .register_extractor("lofty", sonar_extractor_lofty::LoftyExtractor::default())
        .context("registering lofty extractor")?;
    config
        .register_provider("beets", sonar_beets::BeetsMetadataImporter)
        .context("registering beets metadata importer")?;
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

#[derive(Debug, Parser)]
struct ExtractorArgs {
    files: Vec<PathBuf>,
}

async fn cmd_extractor(args: ExtractorArgs) -> Result<()> {
    use sonar::extractor::Extractor;

    let extractors = vec![(
        "lofty",
        Box::new(sonar_extractor_lofty::LoftyExtractor::default()),
    )];

    for file in args.files {
        for extractor in extractors.iter() {
            tracing::info!(
                "extracting metadata from {} using {}",
                file.display(),
                extractor.0
            );
            let extracted = match extractor.1.extract(&file) {
                Ok(extracted) => extracted,
                Err(err) => {
                    tracing::warn!(
                        "failed to extract metadata from {}: {}",
                        file.display(),
                        err
                    );
                    continue;
                }
            };

            tracing::info!("extracted metadata:\n{:#?}", extracted);
        }
    }

    Ok(())
}

async fn list_files_in_directories(paths: Vec<PathBuf>) -> Result<Vec<PathBuf>> {
    let mut queue = paths;
    let mut files = vec![];
    let mut inserted = HashSet::<PathBuf>::new();
    while let Some(p) = queue.pop() {
        let metadata = tokio::fs::metadata(&p).await?;
        if metadata.is_dir() {
            let mut entries = tokio::fs::read_dir(&p).await?;
            while let Some(entry) = entries.next_entry().await? {
                queue.push(entry.path());
            }
        } else if metadata.is_file() {
            match p.canonicalize() {
                Ok(p) => {
                    if inserted.contains(&p) {
                        continue;
                    }
                    inserted.insert(p.clone());
                    files.push(p)
                }
                Err(err) => {
                    tracing::warn!("failed to canonicalize {}: {}", p.display(), err);
                }
            }
        }
    }
    Ok(files)
}
