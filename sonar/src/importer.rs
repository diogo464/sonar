use crate::{
    album, artist, audio,
    blob::BlobStorage,
    bytestream::{self, ByteStream},
    db::Db,
    extractor::SonarExtractor,
    track, AlbumCreate, AlbumId, ArtistCreate, ArtistId, AudioCreate, Error, ErrorKind, Result,
    Track, TrackCreate,
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

pub async fn import(
    importer: &Importer,
    db: &Db,
    storage: &dyn BlobStorage,
    extractors: &[SonarExtractor],
    import: Import,
) -> Result<Track> {
    // acquire permit
    let _permit = importer.semaphore.acquire().await;

    // write to temporary file
    let filename = import
        .filepath
        .as_ref()
        .and_then(|x| x.split('/').last())
        .unwrap_or("input");
    let tmp_dir = tempfile::tempdir()?;
    let tmp_filepath = tmp_dir.path().join(filename);
    bytestream::to_file(import.stream, &tmp_filepath).await?;

    // check file size
    // TODO: we should have a wrapper stream that fails afetr x bytes are read so we are not
    // dumping the whole thing on disk before checking.
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

    let mut metadatas = Vec::with_capacity(handles.len());
    for handle in handles {
        if let Ok(Ok(metadata)) = handle.await {
            metadatas.push(metadata);
        }
    }

    let track_name = match metadatas
        .iter()
        .find_map(|metadata| metadata.title.as_deref())
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
            properties: Default::default(),
        };

        // TODO: add _tx methods
        let mut conn = db.acquire().await?;
        artist::find_or_create_by_name(&mut conn, artist_create)
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
            properties: Default::default(),
        };

        // TODO: add _tx methods
        let mut conn = db.acquire().await?;
        album::find_or_create_by_name(&mut conn, album_create)
            .await?
            .id
    };

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
        properties: Default::default(),
    };
    let track = track::create(&mut conn, track_create).await?;
    conn.commit().await?;
    Ok(track)
}
