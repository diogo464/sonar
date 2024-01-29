//"title": "Your Time",
//"artist": "Savoy feat. KIELY",
//"track": "2/4",
//"album": "1000 Years EP",
//"disc": "1/1",
//"date": "2015-12-02",
//"genre": "complextro",
//"TBPM": "0",
//"compilation": "0",
//"language": "eng",
//"lyrics-XXX": "",
//"album_artist": "Savoy",
//"TLEN": "276486",
//"TIPL": "arranger",
//"TDOR": "2015-12-02",
//"publisher": "Monstercat",
//"Script": "Latn",
//"TSRC": "CA6D21500408",
//"TMED": "Digital Media",
//"encoder": "Lavf60.16.100",
//"artist-sort": "Savoy feat. KIELY",
//"ALBUMARTISTSORT": "Savoy",
//"CATALOGNUMBER": "MCEP086",
//"Album Artist Credit": "Savoy",
//"MusicBrainz Album Type": "e",
//"Artist Credit": "Savoy feat. KIELY",
//"MusicBrainz Album Status": "Official",
//"MusicBrainz Album Release Country": "XW",
//"spotify_album_id": "4frUzLfeOhJxIGZVG5n1iK",
//"spotify_track_id": "496lkFmrm4eXHCXifwqOGW",
//"spotify_artist_id": "25vU5DYwHIHhg1ViWV3SJq",
//"MusicBrainz Album Id": "2ce0cb4b-7958-455e-b6f9-e3d500fd1a99",
//"MusicBrainz Artist Id": "89d03474-2f5f-45fe-839e-209a2728dc9c",
//"MusicBrainz Album Artist Id": "89d03474-2f5f-45fe-839e-209a2728dc9c",
//"MusicBrainz Release Group Id": "836b2d7f-189a-4e99-8c19-22c1545ea7ef",
//"MusicBrainz Release Track Id": "c70dbc1a-6dea-4090-b998-4de2a06c0700"

use crate::{Genre, Genres, Properties};

#[derive(Debug, Clone)]
pub struct ArtistMetadata {
    pub name: Option<String>,
    pub genres: Genres,
    pub properties: Properties,
}

pub struct AlbumMetadata {
    pub name: Option<String>,
    pub genres: Genres,
    pub properties: Properties,
}

pub struct TrackMetadata {
    pub name: Option<String>,
    pub genres: Genres,
    pub properties: Properties,
}
