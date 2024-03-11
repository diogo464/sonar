use async_trait::async_trait;
use sonar::{Error, TrackId};

use crate::{Audio, AudioChunk, AudioClient, Result};

#[derive(Debug, Clone)]
pub struct AudioClientGrpc {
    client: sonar_grpc::Client,
}

impl AudioClientGrpc {
    pub fn new(client: sonar_grpc::Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl AudioClient for AudioClientGrpc {
    async fn stat(&self, track: TrackId) -> Result<u64> {
        let mut client = self.client.clone();
        let response = client
            .track_stat(sonar_grpc::TrackStatRequest {
                track_id: track.to_string(),
            })
            .await
            .map_err(Error::wrap)?;
        Ok(response.into_inner().size as u64)
    }
    async fn chunk(&self, track: TrackId, offset: u64, length: u64) -> Result<AudioChunk> {
        let mut client = self.client.clone();
        let response = client
            .track_download_chunk(sonar_grpc::TrackDownloadChunkRequest {
                track_id: track.to_string(),
                offset: offset as u32,
                size: length as u32,
            })
            .await
            .map_err(Error::wrap)?;
        Ok(AudioChunk {
            data: response.into_inner().data.into(),
            offset,
        })
    }
    async fn download(&self, track: TrackId) -> Result<Audio> {
        todo!()
    }
}
