use std::str::FromStr;

use crate::{
    album, artist, audio,
    blob::BlobStorage,
    bytestream::{self, ByteStream},
    db::Db,
    extractor::SonarExtractor,
    track, AlbumCreate, AlbumId, ArtistCreate, ArtistId, AudioCreate, Error, ErrorKind, Properties,
    PropertyValue, Result, Track, TrackCreate,
};

#[derive(Debug)]
pub struct Config {
    pub max_import_size: usize,
    pub max_concurrent_imports: usize,
}

pub struct Import {
    pub artist: Option<ArtistId>,
    pub album: Option<AlbumId>,
    pub filepath: Option<String>,
    pub stream: ByteStream,
}

impl std::fmt::Debug for Import {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Import")
            .field("artist", &self.artist)
            .field("album", &self.album)
            .field("filename", &self.filepath)
            .finish()
    }
}

#[derive(Debug)]
pub struct Importer {
    config: Config,
    semaphore: tokio::sync::Semaphore,
}

pub fn new(config: Config) -> Importer {
    let semaphore = tokio::sync::Semaphore::new(config.max_concurrent_imports);
    Importer { config, semaphore }
}

#[tracing::instrument(skip(importer, db, storage, extractors))]
pub async fn import(
    importer: &Importer,
    db: &Db,
    storage: &dyn BlobStorage,
    extractors: &[SonarExtractor],
    import: Import,
) -> Result<Track> {
    // acquire permit
    tracing::info!("acquiring import permit for file: {:?}", import.filepath);
    let _permit = importer.semaphore.acquire().await.unwrap();

    tracing::info!("importing file: {:?}", import.filepath);
    // write to temporary file
    let filename = import
        .filepath
        .as_ref()
        .and_then(|x| x.split('/').last())
        .unwrap_or("input");
    let tmp_dir = tempfile::tempdir()?;
    let tmp_filepath = tmp_dir.path().join(filename);
    tracing::debug!("writing import to temporary file: {:?}", tmp_filepath);
    bytestream::to_file(import.stream, &tmp_filepath).await?;

    // check file size
    // TODO: we should have a wrapper stream that fails afetr x bytes are read so we are not
    // dumping the whole thing on disk before checking.
    tracing::debug!("checking file size: {:?}", import.filepath);
    if tokio::fs::metadata(&tmp_filepath).await?.len() as usize > importer.config.max_import_size {
        return Err(Error::new(
            ErrorKind::Invalid,
            format!(
                "file size exceeds maximum import size: {:?}",
                import.filepath
            ),
        ));
    }

    // run metadata extractors
    let mut handles = Vec::with_capacity(extractors.len());
    for extractor in extractors.iter() {
        let extractor = extractor.clone();
        let tmp_filepath = tmp_filepath.clone();
        let handle = tokio::task::spawn_blocking(move || match extractor.extract(&tmp_filepath) {
            Ok(metadata) => {
                tracing::info!("extracted metadata using {}", extractor.name());
                Ok(metadata)
            }
            Err(err) => {
                tracing::warn!(
                    "failed to extract metadata using {}: {}",
                    extractor.name(),
                    err
                );
                Err(err)
            }
        });
        handles.push(handle);
    }

    let path_components = import
        .filepath
        .as_ref()
        .map(|x| x.split('/').collect::<Vec<_>>())
        .unwrap_or_default();

    let mut path_name = None;
    let mut path_album = None;
    let mut path_artist = None;
    if path_components.len() >= 3 {
        path_name = path_components
            .last()
            .map(|x| x.split_once('.').map(|(name, _)| name).unwrap_or(x))
            .map(|x| x.to_string());
        path_album = path_components
            .get(path_components.len() - 2)
            .map(|x| x.to_string());
        path_artist = path_components
            .get(path_components.len() - 3)
            .map(|x| x.to_string());
    }

    let mut metadatas = Vec::with_capacity(handles.len());
    for handle in handles {
        if let Ok(Ok(metadata)) = handle.await {
            metadatas.push(metadata);
        }
    }

    let track_name = match metadatas
        .iter()
        .find_map(|metadata| metadata.title.as_deref())
        .or(path_name.as_deref())
    {
        Some(track_name) => track_name,
        None => {
            return Err(Error::new(
                ErrorKind::Invalid,
                format!("unable to find track name for file: {:?}", import.filepath),
            ))
        }
    };

    // find or create matching artist
    let artist_id = if let Some(artist_id) = import.artist {
        artist_id
    } else {
        let artist_name = match metadatas
            .iter()
            .find_map(|metadata| metadata.artist.as_deref())
            .or(path_artist.as_deref())
        {
            Some(artist_name) => artist_name,
            None => {
                return Err(Error::new(
                    ErrorKind::Invalid,
                    format!("unable to find artist name for file: {:?}", import.filepath),
                ))
            }
        };

        let artist_create = ArtistCreate {
            name: artist_name.to_owned(),
            cover_art: Default::default(),
            genres: Default::default(),
            properties: Default::default(),
        };

        artist::find_or_create_by_name_tx(db, artist_create)
            .await?
            .id
    };

    // find or create matching album
    let album_id = if let Some(album_id) = import.album {
        album_id
    } else {
        let album_name = match metadatas
            .iter()
            .find_map(|metadata| metadata.album.as_deref())
            .or(path_album.as_deref())
        {
            Some(album_name) => album_name,
            None => {
                return Err(Error::new(
                    ErrorKind::Invalid,
                    format!("unable to find album name for file: {:?}", import.filepath),
                ))
            }
        };

        let album_create = AlbumCreate {
            name: album_name.to_owned(),
            artist: artist_id,
            cover_art: Default::default(),
            genres: Default::default(),
            properties: Default::default(),
        };

        album::find_or_create_by_name_tx(db, album_create).await?.id
    };

    let mut properties = Properties::default();
    if let Some(disc_number) = metadatas.iter().find_map(|m| m.disc_number) {
        properties.insert(
            crate::prop::DISC_NUMBER,
            PropertyValue::from_str(&disc_number.to_string()).unwrap(),
        );
    }
    if let Some(track_number) = metadatas.iter().find_map(|m| m.track_number) {
        properties.insert(
            crate::prop::TRACK_NUMBER,
            PropertyValue::from_str(&track_number.to_string()).unwrap(),
        );
    }

    let mut conn = db.begin().await?;
    let audio_stream = bytestream::from_file(&tmp_filepath).await?;
    let audio = audio::create(
        &mut conn,
        storage,
        AudioCreate {
            stream: audio_stream,
            filename: import.filepath,
        },
    )
    .await?;

    // create track
    let track_create = TrackCreate {
        name: track_name.to_owned(),
        album: album_id,
        cover_art: None, // TODO: extract cover art
        lyrics: None,    // TODO: extract lyrics
        audio: Some(audio.id),
        properties,
    };
    let track = track::create(&mut conn, track_create).await?;
    conn.commit().await?;
    Ok(track)
}
