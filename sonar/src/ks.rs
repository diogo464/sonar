use std::path::Path;

use tokio::io::AsyncReadExt;

pub async fn sha256_file(path: &Path) -> std::io::Result<String> {
    use sha2::{Digest, Sha256};
    // workaround for https://github.com/rust-lang/rust-analyzer/issues/15242
    let mut hasher = <Sha256 as Digest>::new();
    let file = tokio::fs::File::open(path).await?;
    let mut reader = tokio::io::BufReader::new(file);
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let n = reader.read(&mut buffer).await?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }
    let hash = hasher.finalize();
    Ok(hex::encode(hash))
}
