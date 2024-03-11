use std::path::PathBuf;

use eyre::Result;

struct Player {}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt::init();


    Ok(())
}
