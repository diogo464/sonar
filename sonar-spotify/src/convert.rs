use std::path::Path;

use sonar::Result;
use tokio::{
    io::{AsyncWriteExt, BufWriter},
    process::Command,
};

#[allow(dead_code)]
pub enum ConvertSamplesSource<'a> {
    Buffer(&'a [i16]),
    Path(&'a Path),
}

pub async fn convert_samples_i16(source: ConvertSamplesSource<'_>, output: &Path) -> Result<()> {
    tracing::info!("converting samples to {:?}", output);
    // https://stackoverflow.com/questions/11986279/can-ffmpeg-convert-audio-from-raw-pcm-to-wav
    // ffmpeg -f s16le -ar 44.1k -ac 2 -i file.pcm file.wav
    let mut cmd = Command::new("ffmpeg");
    cmd.arg("-f")
        .arg("s16le")
        .arg("-ar")
        .arg("44100")
        .arg("-ac")
        .arg("2")
        .arg("-i");

    match source {
        ConvertSamplesSource::Buffer(_) => cmd.arg("-"),
        ConvertSamplesSource::Path(p) => cmd.arg(p),
    };

    cmd.arg("-ar")
        .arg("48000")
        .arg("-ab")
        .arg("320k")
        .arg("-ac")
        .arg("2")
        .arg("-f")
        .arg("mp3")
        .arg("-y")
        .arg(output);

    let child = match source {
        ConvertSamplesSource::Buffer(buf) => {
            let mut child = cmd.stdin(std::process::Stdio::piped()).spawn()?;
            let stdin = child.stdin.as_mut().expect("stdin should be present");
            let mut stdin = BufWriter::new(stdin);
            for sample in buf {
                stdin.write_i16_le(*sample).await?;
            }
            stdin.flush().await?;
            drop(stdin);
            child
        }
        ConvertSamplesSource::Path(_) => cmd.spawn()?,
    };

    child.wait_with_output().await?;
    Ok(())
}
