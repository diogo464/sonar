use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use serde::Deserialize;
use sonar::metadata::prelude::*;
use tokio::process::Command;

const BEETS_CONFIG_TEMPLATE: &'static str = include_str!("config.yaml");
const MARKER_LIBRARY_PATH: &'static str = "{{LIBRARY_PATH}}";
const MARKER_DIRECTORY_PATH: &'static str = "{{DIRECTORY_PATH}}";
const MARKER_PLUGINS: &'static str = "{{PLUGINS}}";
const PLUGIN_COVERART: &'static str = "fetchart";

const TAG_TITLE: &'static str = "title";
const TAG_ARTIST: &'static str = "artist";
const TAG_ALBUM: &'static str = "album";
// track number in format "<track>/<total>"
const TAG_TRACK_NUMBER: &'static str = "track";
// disc number in format "<disc>/<total>"
const TAG_DISC_NUMBER: &'static str = "disc";
const TAG_GENRE: &'static str = "genre";
// release date in format "YYYY-MM-DD"
const TAG_RELEASE_DATE: &'static str = "date";
const TAG_PUBLISHER: &'static str = "publisher";
/*
    "MusicBrainz Work Id": "fe2eab1d-646f-4836-8651-6d8ce0f8205d",
    "MusicBrainz Album Id": "37e4a79b-723f-4501-94aa-775c609b7fdf",
    "MusicBrainz Artist Id": "24e1b53c-3085-4581-8472-0b0088d2508c",
    "MusicBrainz Album Artist Id": "24e1b53c-3085-4581-8472-0b0088d2508c",
    "MusicBrainz Release Group Id": "fe4373ed-5e89-46b3-b4c0-31433ce217df",
    "MusicBrainz Release Track Id": "cc59647c-3435-3c96-a62a-56334aa27ebd"
*/
const TAG_MUSICBRAINZ_WORK_ID: &'static str = "MusicBrainz Work Id";
const TAG_MUSICBRAINZ_ALBUM_ID: &'static str = "MusicBrainz Album Id";
const TAG_MUSICBRAINZ_ARTIST_ID: &'static str = "MusicBrainz Artist Id";
const TAG_MUSICBRAINZ_ALBUM_ARTIST_ID: &'static str = "MusicBrainz Album Artist Id";
const TAG_MUSICBRAINZ_RELEASE_GROUP_ID: &'static str = "MusicBrainz Release Group Id";
const TAG_MUSICBRAINZ_RELEASE_TRACK_ID: &'static str = "MusicBrainz Release Track Id";

#[derive(Debug, Default)]
pub struct BeetsMetadataImporter;

#[derive(Debug, Deserialize)]
struct FFprobeOutput {
    format: FFprobeFormat,
}

#[derive(Debug, Deserialize)]
struct FFprobeFormat {
    tags: HashMap<String, String>,
}

struct PreparedDirectory {
    album_directory: PathBuf,
    tracks: Vec<(TrackId, PathBuf)>,
}

#[sonar::async_trait]
impl sonar::metadata::MetadataProvider for BeetsMetadataImporter {
    fn supports(&self, kind: MetadataRequestKind) -> bool {
        match kind {
            MetadataRequestKind::Album => true,
            MetadataRequestKind::AlbumTracks => true,
            _ => false,
        }
    }
    async fn album_metadata(
        &self,
        context: &sonar::Context,
        request: &AlbumMetadataRequest,
    ) -> Result<AlbumMetadata> {
        let dir = tempfile::tempdir()?;
        let tracks =
            sonar::track_list_by_album(context, request.album.id, Default::default()).await?;
        let prepared = prepare_directory(
            context,
            request.artist.clone(),
            request.album.clone(),
            tracks.clone(),
            dir.path(),
            true,
        )
        .await?;

        tracing::info!("reading album cover art");
        let cover_path = prepared.album_directory.join("cover.jpg");
        let cover = tokio::fs::read(&cover_path).await?;

        Ok(AlbumMetadata {
            cover: Some(From::from(cover)),
            ..Default::default()
        })
    }
    async fn album_tracks_metadata(
        &self,
        context: &sonar::Context,
        request: &AlbumTracksMetadataRequest,
    ) -> Result<AlbumTracksMetadata> {
        let dir = tempfile::tempdir()?;
        let prepared = prepare_directory(
            context,
            request.artist.clone(),
            request.album.clone(),
            request.tracks.clone(),
            dir.path(),
            false,
        )
        .await?;
        let tracks = prepared.tracks;
        let mut track_metadata: HashMap<TrackId, TrackMetadata> = HashMap::new();

        tracing::info!("fetching track metadata");
        //ffprobe -v 0 -show_format -print_format json "$argv[1]" | jq
        for (track_id, track_path) in tracks {
            tracing::debug!("fetching metadata for track {}", track_id);
            let ffprobe_output = Command::new("ffprobe")
                .arg("-v")
                .arg("0")
                .arg("-show_format")
                .arg("-print_format")
                .arg("json")
                .arg(&track_path)
                .output()
                .await?;

            let ffprobe_output = String::from_utf8(ffprobe_output.stdout).map_err(Error::wrap)?;
            let ffprobe_output: FFprobeOutput =
                serde_json::from_str(&ffprobe_output).map_err(Error::wrap)?;
            let tags = ffprobe_output.format.tags;
            tracing::debug!("tags: {:#?}", tags);

            let name = tags.get(TAG_TITLE).map(|s| s.to_string());
            let track_number = tags
                .get(TAG_TRACK_NUMBER)
                .and_then(parse_track_or_disc_number);
            let disc_number = tags
                .get(TAG_DISC_NUMBER)
                .and_then(parse_track_or_disc_number);

            let mut properties = Properties::new();
            if let Some(track_number) = track_number {
                properties.insert(sonar::prop::TRACK_NUMBER, track_number.into());
            }
            if let Some(disc_number) = disc_number {
                properties.insert(sonar::prop::DISC_NUMBER, disc_number.into());
            }

            track_metadata.insert(
                track_id,
                TrackMetadata {
                    name,
                    properties,
                    cover: None,
                },
            );
        }

        Ok(AlbumTracksMetadata {
            tracks: track_metadata,
        })
    }
}

async fn prepare_directory(
    context: &sonar::Context,
    artist: Artist,
    album: Album,
    tracks: Vec<Track>,
    temp_dir: &Path,
    enable_coverart: bool,
) -> Result<PreparedDirectory> {
    let plugins = if enable_coverart {
        PLUGIN_COVERART
    } else {
        "''"
    };
    let config_path = temp_dir.join("config.yaml");
    let library_path = temp_dir.join("library.db");
    let import_path = temp_dir.join("data");

    tracing::info!("creating beets config file");
    let config_content = BEETS_CONFIG_TEMPLATE
        .replace(MARKER_LIBRARY_PATH, &library_path.display().to_string())
        .replace(MARKER_DIRECTORY_PATH, &import_path.display().to_string())
        .replace(MARKER_PLUGINS, plugins);
    tracing::debug!("beets config file content: {}", config_content);
    tokio::fs::write(&config_path, config_content).await?;

    tracing::info!("creating album directory");
    let artist_name = &artist.name;
    let album_name = &album.name;
    let album_directory = import_path.join(artist_name).join(album_name);
    let mut tracks_paths = Vec::new();
    tokio::fs::create_dir_all(&album_directory).await?;

    tracing::info!("downloading tracks");
    for track in tracks.iter() {
        let track_name = &track.name;
        let track_path = album_directory.join(track_name);
        let stream = sonar::track_download(context, track.id, Default::default()).await?;
        tracing::debug!("downloading track {} to {}", track.id, track_path.display());
        sonar::bytestream::to_file(stream, &track_path).await?;
        tracks_paths.push((track.id, track_path));
    }

    tracing::info!("running beet import");
    let status = Command::new("beet")
        .arg("-c")
        .arg(&config_path)
        .arg("import")
        .arg(&import_path)
        .status()
        .await?;

    if !status.success() {
        return Err(Error::new(ErrorKind::Internal, "beet import failed"));
    }

    Ok(PreparedDirectory {
        album_directory,
        tracks: tracks_paths,
    })
}

fn parse_track_or_disc_number(s: &String) -> Option<u32> {
    let mut parts = s.split('/');
    let track_number = parts.next()?.parse::<u32>().ok()?;
    parts.next()?;
    Some(track_number)
}
