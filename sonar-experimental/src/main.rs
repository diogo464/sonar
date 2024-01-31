use std::path::PathBuf;

use clap::Parser;
use eyre::Result;
use lofty::AudioFile;

#[derive(Debug, Parser)]
struct Args {
    filepath: PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    let filepath = args.filepath;
    let file = lofty::read_from_path(filepath)?;
    let props = file.properties();
    println!("{:#?}", props);

    Ok(())
}
