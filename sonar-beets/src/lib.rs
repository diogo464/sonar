use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use lofty::{
    config::WriteOptions,
    prelude::*,
    probe::Probe,
    tag::{Tag, TagType},
};
use serde::Deserialize;
use sonar::{bytes::Bytes, metadata_prelude::*};
use tokio::process::Command;

const BEETS_CONFIG_TEMPLATE: &str = include_str!("config.yaml");
const MARKER_LIBRARY_PATH: &str = "{{LIBRARY_PATH}}";
const MARKER_DIRECTORY_PATH: &str = "{{DIRECTORY_PATH}}";
const MARKER_PLUGINS: &str = "{{PLUGINS}}";
const PLUGIN_COVERART: &str = "fetchart";

const TAG_TITLE: &str = "title";
// const TAG_ARTIST: &str = "artist";
// const TAG_ALBUM: &str = "album";
// track number in format "<track>/<total>"
const TAG_TRACK_NUMBER: &str = "track";
// disc number in format "<disc>/<total>"
const TAG_DISC_NUMBER: &str = "disc";
// const TAG_GENRE: &str = "genre";
// release date in format "YYYY-MM-DD"
// const TAG_RELEASE_DATE: &str = "date";
// const TAG_PUBLISHER: &str = "publisher";
/*
    "MusicBrainz Work Id": "fe2eab1d-646f-4836-8651-6d8ce0f8205d",
    "MusicBrainz Album Id": "37e4a79b-723f-4501-94aa-775c609b7fdf",
    "MusicBrainz Artist Id": "24e1b53c-3085-4581-8472-0b0088d2508c",
    "MusicBrainz Album Artist Id": "24e1b53c-3085-4581-8472-0b0088d2508c",
    "MusicBrainz Release Group Id": "fe4373ed-5e89-46b3-b4c0-31433ce217df",
    "MusicBrainz Release Track Id": "cc59647c-3435-3c96-a62a-56334aa27ebd"
*/
// const TAG_MUSICBRAINZ_WORK_ID: &str = "MusicBrainz Work Id";
// const TAG_MUSICBRAINZ_ALBUM_ID: &str = "MusicBrainz Album Id";
// const TAG_MUSICBRAINZ_ARTIST_ID: &str = "MusicBrainz Artist Id";
// const TAG_MUSICBRAINZ_ALBUM_ARTIST_ID: &str = "MusicBrainz Album Artist Id";
// const TAG_MUSICBRAINZ_RELEASE_GROUP_ID: &str = "MusicBrainz Release Group Id";
// const TAG_MUSICBRAINZ_RELEASE_TRACK_ID: &str = "MusicBrainz Release Track Id";

#[derive(Debug, Default)]
pub struct BeetsMetadataProvider;

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
impl MetadataProvider for BeetsMetadataProvider {
    fn supports(&self, kind: MetadataRequestKind) -> bool {
        tracing::debug!("checking if provider supports kind: {:?}", kind);
        std::matches!(
            kind,
            MetadataRequestKind::Album | MetadataRequestKind::AlbumTracks
        )
    }
    #[tracing::instrument(skip(self, context, request), fields(album = request.album.id.to_string()))]
    async fn album_metadata(
        &self,
        context: &sonar::Context,
        request: &AlbumMetadataRequest,
    ) -> Result<AlbumMetadata> {
        tracing::info!("fetching album metadata for {:#?}", request.album);
        tracing::debug!("creating temporary directory");
        let dir = tempfile::tempdir()?;
        tracing::debug!("fetching album tracks");
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

        let cover_path_jpg = prepared.album_directory.join("cover.jpg");
        let cover_path_png = prepared.album_directory.join("cover.png");
        let cover_path = if cover_path_jpg.exists() {
            Some(cover_path_jpg)
        } else if cover_path_png.exists() {
            Some(cover_path_png)
        } else {
            tracing::warn!("no cover art found for album");
            None
        };

        let cover = match cover_path {
            Some(cover_path) => {
                tracing::info!("reading album cover art from {}", cover_path.display());
                let cover = tokio::fs::read(&cover_path).await?;
                Some(Bytes::from(cover))
            }
            None => None,
        };

        Ok(AlbumMetadata {
            cover,
            ..Default::default()
        })
    }

    #[tracing::instrument(skip(self, context, request), fields(album = request.album.id.to_string()))]
    async fn album_tracks_metadata(
        &self,
        context: &sonar::Context,
        request: &AlbumTracksMetadataRequest,
    ) -> Result<AlbumTracksMetadata> {
        tracing::info!("fetching album tracks metadata for {:#?}", request.album);
        tracing::debug!("creating temporary directory");
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
            tracing::debug!("ffprobe output for ({track_id}, {track_path:?}):\n{ffprobe_output}");
            let ffprobe_output: FFprobeOutput =
                serde_json::from_str(&ffprobe_output).map_err(Error::wrap)?;
            let tags = ffprobe_output.format.tags;
            tracing::debug!("tags: {:#?}", tags);

            let name = tags.get(TAG_TITLE).map(|s| s.to_string());
            let track_number = tags
                .get(TAG_TRACK_NUMBER)
                .map(|s| s.as_str())
                .and_then(parse_track_or_disc_number);
            let disc_number = tags
                .get(TAG_DISC_NUMBER)
                .map(|s| s.as_str())
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

#[tracing::instrument(skip_all)]
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
    tracing::info!("creating beets config file with plugins: {}", plugins);
    let config_path = temp_dir.join("config.yaml");
    let library_path = temp_dir.join("library.db");
    let import_path = temp_dir.join("data");

    tracing::info!("creating beets config file");
    let config_content = BEETS_CONFIG_TEMPLATE
        .replace(MARKER_LIBRARY_PATH, &library_path.display().to_string())
        .replace(MARKER_DIRECTORY_PATH, &import_path.display().to_string())
        .replace(MARKER_PLUGINS, plugins);
    tracing::debug!("beets config file content:\n{}", config_content);
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
        // TODO: we are just assuming mp3 but we should somehow request this explicitly or use the
        // mime type from the track
        let track_path = album_directory.join(track_name).with_extension("mp3");
        let download = sonar::track_download(context, track.id, Default::default()).await?;
        tracing::debug!("downloading track {} to {}", track.id, track_path.display());
        sonar::bytestream::to_file(download.stream, &track_path).await?;

        // NOTE: beets seems to import better when these tags are set
        tracing::debug!("tagging track {} with artist, album and title", track.id);
        let _ = tokio::task::spawn_blocking({
            let artist_name = artist_name.clone();
            let album_name = album_name.clone();
            let track_name = track_name.clone();
            let track_path = track_path.clone();
            move || {
                let mut tagged_file = Probe::open(&track_path)
                    .expect("ERROR: Bad path provided!")
                    .read()
                    .expect("ERROR: Failed to read file!");
                let tag = match tagged_file.first_tag_mut() {
                    Some(tag) => tag,
                    None => {
                        tagged_file.insert_tag(Tag::new(TagType::Id3v2));
                        tagged_file.primary_tag_mut().unwrap()
                    }
                };
                tag.set_artist(artist_name);
                tag.set_album(album_name);
                tag.set_title(track_name);
                tag.save_to_path(&track_path, WriteOptions::default())
                    .expect("ERROR: Failed to save tags!");
            }
        })
        .await;

        tracks_paths.push((track.id, track_path));
    }

    // NOTE: this is just for debugging purposes
    tracing::debug!("running find on target before running beets directory");
    let _ = Command::new("find").arg(&album_directory).status().await;

    tracing::info!("running beet import");
    let status = Command::new("beet")
        .arg("-v")
        .arg("-c")
        .arg(&config_path)
        .arg("import")
        .arg(&import_path)
        .status()
        .await?;

    tracing::debug!("running find on target after running beets directory");
    let _ = Command::new("find").arg(&album_directory).status().await;

    if !status.success() {
        return Err(Error::new(ErrorKind::Internal, "beet import failed"));
    }

    Ok(PreparedDirectory {
        album_directory,
        tracks: tracks_paths,
    })
}

fn parse_track_or_disc_number(s: &str) -> Option<u32> {
    let mut parts = s.split('/');
    let track_number = parts.next()?.parse::<u32>().ok()?;
    parts.next()?;
    Some(track_number)
}
