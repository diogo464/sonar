use lofty::{Accessor, AudioFile, TaggedFileExt};

#[derive(Debug, Default)]
pub struct LoftyExtractor;

impl sonar::metadata::Extractor for LoftyExtractor {
    fn extract(
        &self,
        path: &std::path::Path,
    ) -> std::io::Result<sonar::metadata::ExtractedMetadata> {
        let file = lofty::read_from_path(path)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        let properties = file.properties();
        let tag = match file.primary_tag().or_else(|| file.first_tag()) {
            Some(tag) => tag,
            None => {
                tracing::info!("no tags found for {}", path.display());
                return Ok(Default::default());
            }
        };

        let title = tag.title().map(|x| x.to_string());
        let album = tag.album().map(|x| x.to_string());
        let artist = tag.artist().map(|x| x.to_string());
        let track_number = tag.track();
        let disc_number = tag.disk();
        let duration = properties.duration();
        let genres = tag
            .genre()
            .and_then(|g| g.parse::<sonar::Genre>().ok())
            .map(sonar::Genres::from)
            .unwrap_or_default();

        let cover_art = tag
            .pictures()
            .iter()
            .next()
            .map(|p| sonar::metadata::ExtractedImage {
                mime_type: p
                    .mime_type()
                    .map(|x| x.to_string())
                    .unwrap_or_else(|| "image/jpeg".to_string()),
                data: p.data().to_vec(),
            });

        Ok(sonar::metadata::ExtractedMetadata {
            title,
            album,
            artist,
            track_number,
            disc_number,
            duration: Some(duration),
            release_date: None,
            cover_art,
            genres,
        })
    }
}
