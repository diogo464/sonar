use lofty::{
    file::{AudioFile, TaggedFileExt},
    prelude::ItemKey,
    tag::Accessor,
};

#[derive(Debug, Default)]
pub struct LoftyExtractor;

impl sonar::Extractor for LoftyExtractor {
    #[tracing::instrument(skip(self))]
    fn extract(&self, path: &std::path::Path) -> std::io::Result<sonar::ExtractedMetadata> {
        tracing::info!("extracting metadata from {} using lofty", path.display());
        let file = lofty::read_from_path(path)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        let properties = file.properties();
        tracing::debug!("properties: {:#?}", properties);
        let tag = match file.primary_tag().or_else(|| file.first_tag()) {
            Some(tag) => tag,
            None => {
                tracing::info!("no tags found for {}", path.display());
                return Ok(Default::default());
            }
        };

        let title = tag.title().map(|x| x.to_string());
        let album = tag.album().map(|x| x.to_string());
        let artist = match tag.get_string(&ItemKey::AlbumArtist) {
            Some(x) => Some(x.to_string()),
            None => tag.artist().map(|x| x.to_string()),
        };
        let track_number = tag.track();
        let disc_number = tag.disk();
        let duration = properties.duration();
        let genres = tag
            .genre()
            .and_then(|g| g.parse::<sonar::Genre>().ok())
            .map(sonar::Genres::from)
            .unwrap_or_default();

        let cover_art = tag.pictures().iter().next().map(|p| sonar::ExtractedImage {
            mime_type: p
                .mime_type()
                .map(|x| x.to_string())
                .unwrap_or_else(|| "image/jpeg".to_string()),
            data: p.data().to_vec(),
        });

        let metadata = sonar::ExtractedMetadata {
            title,
            album,
            artist,
            track_number,
            disc_number,
            duration: Some(duration),
            release_date: None,
            cover_art,
            genres,
        };
        tracing::debug!("metadata: {:#?}", metadata);
        Ok(metadata)
    }
}
