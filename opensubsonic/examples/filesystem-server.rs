use std::{
    collections::HashMap,
    net::SocketAddr,
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::{Context, Result as AResult};
use lofty::{
    file::{AudioFile as _, TaggedFileExt as _},
    probe::Probe,
    tag::Accessor as _,
};
use opensubsonic::service::prelude::*;
use tokio::net::TcpListener;

struct FilesystemServer {
    pictures: HashMap<String, Picture>,
    tracks: HashMap<String, Child>,
    albums: HashMap<String, AlbumID3>,
    artists: HashMap<String, ArtistID3>,
}

impl FilesystemServer {
    fn new(metadata: Vec<TrackMetadata>) -> Self {
        let mut pictures: HashMap<String, Picture> = Default::default();
        let mut artists: HashMap<String, ArtistID3> = Default::default();
        let mut albums: HashMap<String, AlbumID3> = Default::default();
        let mut tracks: HashMap<String, Child> = Default::default();

        for m in metadata {
            let artist_id = Self::make_artist_id(&m.artist);
            let album_id = Self::make_album_id(&m.artist, &m.album);
            let track_id = Self::make_track_id(&m.artist, &m.album, &m.title);

            if !artists.contains_key(&artist_id) {
                artists.insert(
                    artist_id.clone(),
                    ArtistID3 {
                        id: artist_id.clone(),
                        name: m.artist.clone(),
                        album_count: 0,
                        cover_art: None,
                        artist_image_url: None,
                        starred: None,
                    },
                );
            }

            if !albums.contains_key(&album_id) {
                albums.insert(
                    album_id.clone(),
                    AlbumID3 {
                        id: album_id.clone(),
                        name: m.album.clone(),
                        artist: Some(m.artist.clone()),
                        artist_id: Some(artist_id.clone()),
                        ..Default::default()
                    },
                );
            }

            if !tracks.contains_key(&track_id) {
                tracks.insert(
                    track_id.clone(),
                    Child {
                        id: track_id.clone(),
                        parent: Some(album_id.clone()),
                        is_dir: false,
                        title: m.title.clone(),
                        album: Some(m.album.clone()),
                        artist: Some(m.artist.clone()),
                        genre: m.genre.clone(),
                        cover_art: None,
                        album_id: Some(album_id.clone()),
                        artist_id: Some(artist_id.clone()),
                        path: Some(m.path.to_string_lossy().to_string()),
                        duration: Some(From::from(m.duration)),
                        ..Default::default()
                    },
                );
            }

            if let Some(picture) = m.picture {
                pictures.insert(track_id.clone(), picture);
                if let Some(album) = albums.get_mut(&album_id) {
                    album.cover_art = Some(track_id.clone());
                }
            }
        }

        for track in tracks.values() {
            if let Some(album) = albums.get_mut(&track.id) {
                album.song_count += 1;
                album.duration = From::from(
                    album.duration.to_duration() + track.duration.unwrap_or_default().to_duration(),
                );
            }
        }

        artists
            .keys()
            .chain(albums.keys())
            .chain(tracks.keys())
            .for_each(|id| {
                tracing::debug!("loaded: {}", id);
            });

        Self {
            pictures,
            tracks,
            albums,
            artists,
        }
    }

    fn make_artist_id(artist: &str) -> String {
        format!("artist:{}", artist)
    }
    fn make_album_id(artist: &str, album: &str) -> String {
        format!("album:{}-{}", artist, album)
    }
    fn make_track_id(artist: &str, album: &str, title: &str) -> String {
        format!("track:{}-{}-{}", artist, album, title)
    }
}

#[opensubsonic::async_trait]
impl OpenSubsonicServer for FilesystemServer {
    async fn ping(&self, _request: Request<Ping>) -> Result<()> {
        Ok(())
    }
    async fn scrobble(&self, request: Request<Scrobble>) -> Result<()> {
        tracing::info!("scrobble: {:?}", request.body);
        Ok(())
    }
    async fn get_cover_art(&self, request: Request<GetCoverArt>) -> Result<Image> {
        let track_id = &request.body.id;
        let picture = self
            .pictures
            .get(track_id)
            .unwrap_or_else(|| panic!("picture '{}' not found", track_id));
        Ok(Image {
            mime_type: picture
                .mime_type
                .as_deref()
                .unwrap_or("image/jpeg")
                .to_string(),
            data: From::from(picture.data.clone()),
        })
    }
    async fn get_starred2(&self, _request: Request<GetStarred2>) -> Result<Starred2> {
        Ok(Default::default())
    }
    async fn get_playlists(&self, _request: Request<GetPlaylists>) -> Result<Playlists> {
        Ok(Default::default())
    }
    async fn get_artists(&self, _request: Request<GetArtists>) -> Result<ArtistsID3> {
        let mut artists_by_letter: HashMap<char, Vec<ArtistID3>> = Default::default();
        for artist in self.artists.values() {
            let letter = artist
                .name
                .chars()
                .next()
                .unwrap_or_default()
                .to_ascii_uppercase();
            artists_by_letter
                .entry(letter)
                .or_default()
                .push(artist.clone());
        }

        Ok(ArtistsID3 {
            index: artists_by_letter
                .into_iter()
                .map(|(letter, artists)| IndexID3 {
                    name: letter.to_string(),
                    artist: artists,
                })
                .collect(),
            ignored_articles: Default::default(),
        })
    }
    async fn get_album(&self, request: Request<GetAlbum>) -> Result<AlbumWithSongsID3> {
        let album_id = &request.body.id;
        Ok(AlbumWithSongsID3 {
            album: self
                .albums
                .get(album_id)
                .unwrap_or_else(|| panic!("album '{}' not found", album_id))
                .clone(),
            song: self
                .tracks
                .values()
                .filter(|t| t.album_id.as_ref() == Some(album_id))
                .cloned()
                .collect(),
        })
    }
    async fn get_album_list2(&self, request: Request<GetAlbumList2>) -> Result<AlbumList2> {
        let offset = request.body.offset.unwrap_or_default();
        Ok(AlbumList2 {
            album: self
                .albums
                .values()
                .skip(offset as usize)
                .cloned()
                .collect(),
        })
    }

    // TODO: re-add this method
    // async fn stream(
    //     &self,
    //     request: Request<Stream>,
    //     // TODO: use range
    //     _range: Option<ByteRange>,
    // ) -> Result<ByteStream> {
    //     let track_id = &request.body.id;
    //     let track = self
    //         .tracks
    //         .get(track_id)
    //         .expect(&format!("track '{}' not found", track_id));
    //     let content = std::fs::read(track.path.as_ref().unwrap()).expect("failed to read file");
    //     Ok(ByteStream::new(
    //         "audio/mpeg",
    //         tokio_stream::once(Ok(From::from(content))),
    //     ))
    // }
}

struct Picture {
    mime_type: Option<String>,
    data: Vec<u8>,
}

impl std::fmt::Debug for Picture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Picture")
            .field("mime_type", &self.mime_type)
            .field("data_len", &self.data.len())
            .finish()
    }
}

#[derive(Debug)]
struct TrackMetadata {
    path: PathBuf,
    title: String,
    album: String,
    artist: String,
    genre: Option<String>,
    duration: Duration,
    picture: Option<Picture>,
}

#[tokio::main]
async fn main() -> AResult<()> {
    tracing_subscriber::fmt::init();

    let tracks = read_metadata_dir(Path::new("music"))?;
    tracing::info!("Found {} tracks", tracks.len());
    for track in tracks.iter() {
        tracing::debug!("{:?}", track);
    }

    let server = FilesystemServer::new(tracks);
    let service = OpenSubsonicService::new("filesystem-server", "1.0.0", server);
    let router = axum::Router::default()
        .nest_service("/", service)
        .layer(tower_http::trace::TraceLayer::new_for_http());

    axum::serve(
        TcpListener::bind("0.0.0.0:3000".parse::<SocketAddr>().unwrap())
            .await
            .context("failed to bind")?,
        router,
    )
    .await
    .context("failed to serve")?;

    Ok(())
}

fn read_metadata_dir(path: &Path) -> AResult<Vec<TrackMetadata>> {
    let mut tracks = Vec::new();
    let mut queue = vec![path.to_path_buf()];

    tracing::info!("Reading metadata from directory: {:?}", path);
    while let Some(path) = queue.pop() {
        if path.is_dir() {
            std::fs::read_dir(path)?
                .filter_map(|entry| entry.ok())
                .for_each(|entry| queue.push(entry.path()));
        } else {
            let metadata = read_metadata(&path)?;
            tracks.push(metadata);
        }
    }

    Ok(tracks)
}

fn read_metadata(path: &Path) -> AResult<TrackMetadata> {
    tracing::info!("Reading metadata from file: {:?}", path);
    if !path.is_file() {
        anyhow::bail!("path is not a file!");
    }

    let tagged_file = Probe::open(path)
        .context("failed to open file")?
        .read()
        .context("failed to read file")?;

    let tag = match tagged_file.primary_tag() {
        Some(primary_tag) => primary_tag,
        // If the "primary" tag doesn't exist, we just grab the
        // first tag we can find. Realistically, a tag reader would likely
        // iterate through the tags to find a suitable one.
        None => tagged_file.first_tag().expect("ERROR: No tags found!"),
    };

    let picture = tag.pictures().first().map(|p| Picture {
        mime_type: p.mime_type().map(|m| m.to_string()),
        data: p.data().to_vec(),
    });

    let properties = tagged_file.properties();
    let duration = properties.duration();

    Ok(TrackMetadata {
        path: path.to_path_buf(),
        title: tag
            .title()
            .map(|t| t.to_string())
            .unwrap_or("None".to_string()),
        album: tag
            .album()
            .map(|a| a.to_string())
            .unwrap_or("None".to_string()),
        artist: tag
            .artist()
            .map(|a| a.to_string())
            .unwrap_or("None".to_string()),
        genre: tag.genre().map(|g| g.to_string()),
        duration,
        picture,
    })
}
