-- the NOT NULL in the primary keys is beacause sqlx was sometimes returning Option<T> if the NOT NULL was no there.

-- TABLE DEFINITIONS

CREATE TABLE user (
	id			INTEGER PRIMARY KEY NOT NULL,
	username		TEXT NOT NULL UNIQUE,
	-- scrypt PHC string
	password_hash		TEXT NOT NULL DEFAULT '',
	avatar			INTEGER REFERENCES image(id),
	created_at		INTEGER NOT NULL DEFAULT (unixepoch())
);
CREATE INDEX user_username ON user(username);

CREATE TABLE property (
	namespace	INTEGER NOT NULL,
	identifier	INTEGER NOT NULL,
	user		INTEGER REFERENCES user(id) DEFAULT NULL,
	key		TEXT NOT NULL,
	value		TEXT NOT NULL,
	PRIMARY KEY (namespace, identifier, key),
	UNIQUE(namespace, identifier, user, key)
);
CREATE INDEX property_key ON property(key);
CREATE INDEX property_user_key ON property(user, key);
CREATE INDEX property_key_value ON property(key, value);
CREATE INDEX property_user_key_value ON property(user, key ,value);
CREATE INDEX property_namespace_identifier ON property(namespace, identifier);
CREATE INDEX property_namespace_identifier_key ON property(namespace, identifier, key);
CREATE INDEX property_namespace_identifier_user_key ON property(namespace, identifier, user, key);

CREATE TABLE genre (
	namespace 	INTEGER NOT NULL,
	identifier 	INTEGER NOT NULL,
	genre		TEXT NOT NULL,
	PRIMARY KEY(namespace, identifier, genre)
);
CREATE INDEX genre_genre ON genre(genre);
CREATE INDEX genre_namespace_identifier ON genre(namespace, identifier);

CREATE TABLE blob (
	id	INTEGER PRIMARY KEY NOT NULL,
	key	TEXT NOT NULL UNIQUE,
	size	INTEGER NOT NULL,
	sha256	TEXT NOT NULL CHECK(LENGTH(sha256) = 64)
);
CREATE INDEX blob_key ON blob(key);
CREATE INDEX blob_sha256 ON blob(sha256);

CREATE TABLE image (
	id		INTEGER PRIMARY KEY NOT NULL,
	blob		INTEGER NOT NULL REFERENCES blob(id),
	mime_type	TEXT NOT NULL,
	created_at	INTEGER NOT NULL DEFAULT (unixepoch())
);

CREATE TABLE audio (
	id			INTEGER PRIMARY KEY NOT NULL,
	bitrate			INTEGER NOT NULL,
	duration_ms		INTEGER NOT NULL,
	num_channels		INTEGER NOT NULL,
	sample_freq		INTEGER NOT NULL,
	mime_type		TEXT NOT NULL,
	blob			INTEGER NOT NULL REFERENCES blob(id),
	filename		TEXT,
	created_at		INTEGER NOT NULL DEFAULT (unixepoch())
);

CREATE TABLE pin (
	namespace	INTEGER NOT NULL,
	identifier	INTEGER NOT NULL,
	user		INTEGER NOT NULL REFERENCES user(id),
	PRIMARY KEY(namespace, identifier, user)
);
CREATE INDEX pin_namespace_identifier ON pin(namespace, identifier);
CREATE INDEX pin_namespace_identifier_user ON pin(namespace, identifier, user);

CREATE TABLE subscription (
	user			INTEGER NOT NULL REFERENCES user(id),
	external_id		TEXT NOT NULL,
	PRIMARY KEY (user, external_id)
);
CREATE INDEX subscription_user ON subscription(user);

CREATE TABLE artist (
	id			INTEGER PRIMARY KEY NOT NULL,
	name 			TEXT NOT NULL,
	listen_count		INTEGER NOT NULL DEFAULT 0,
	cover_art		INTEGER REFERENCES image(id),
	created_at		INTEGER NOT NULL DEFAULT (unixepoch())
);

CREATE TABLE album (
	id			INTEGER PRIMARY KEY NOT NULL,
	name			TEXT NOT NULL,
	artist			INTEGER NOT NULL REFERENCES artist(id),
	listen_count		INTEGER NOT NULL DEFAULT 0,
	cover_art		INTEGER REFERENCES image(id),
	created_at		INTEGER NOT NULL DEFAULT (unixepoch())
);
CREATE INDEX album_artist ON album(artist);

CREATE TABLE track (
	id			INTEGER PRIMARY KEY NOT NULL,
	name			TEXT NOT NULL DEFAULT '',
	album			INTEGER NOT NULL REFERENCES album(id),
	listen_count		INTEGER NOT NULL DEFAULT 0,
	cover_art		INTEGER REFERENCES image(id),
	-- S : Synced lyrics, U : Unsynced lyrics
	lyrics_kind		TEXT CHECK (lyrics_kind IS NULL OR lyrics_kind IN ('S', 'U')),
	created_at		INTEGER NOT NULL DEFAULT (unixepoch())
);
CREATE INDEX track_album ON track(album);

CREATE TABLE track_lyrics_line (
	track		INTEGER NOT NULL REFERENCES track(id),
	offset		INTEGER NOT NULL, -- milliseconds since start
	duration	INTEGER NOT NULL, -- milliseconds
	text		TEXT NOT NULL
);
CREATE INDEX track_lyrics_line_track ON track_lyrics_line(track);

CREATE TABLE track_audio (
	track		INTEGER NOT NULL REFERENCES track(id),
	audio		INTEGER NOT NULL REFERENCES audio(id),
	preferred	BOOLEAN CHECK(preferred IS NULL or preferred IS TRUE),
	UNIQUE(track, preferred),
	UNIQUE(track, audio)
);
CREATE INDEX track_audio_track ON track_audio(track);
CREATE INDEX track_audio_audio ON track_audio(audio);
CREATE INDEX track_audio_track_audio ON track_audio(track, audio);

CREATE TABLE playlist (
	id		INTEGER PRIMARY KEY NOT NULL,
	owner		INTEGER NOT NULL REFERENCES user(id),
	name		TEXT NOT NULL DEFAULT '',
	created_at	INTEGER NOT NULL DEFAULT (unixepoch()),
	UNIQUE(owner, name)
);
CREATE INDEX playlist_owner ON playlist(owner);

CREATE TABLE playlist_track (
	playlist	INTEGER NOT NULL REFERENCES playlist(id),
	track		INTEGER NOT NULL REFERENCES track(id),
	created_at	INTEGER NOT NULL DEFAULT (unixepoch()),
	PRIMARY KEY(playlist, track)
);
CREATE INDEX playlist_track_playlist ON playlist_track(playlist);

CREATE TABLE scrobble (
	id			INTEGER PRIMARY KEY NOT NULL,
	user			INTEGER NOT NULL REFERENCES user(id),
	track			INTEGER NOT NULL REFERENCES track(id),
	listen_at		INTEGER NOT NULL,
	listen_secs		INTEGER NOT NULL,
	listen_device		TEXT NOT NULL DEFAULT '',
	created_at		INTEGER NOT NULL DEFAULT (unixepoch())
);

CREATE TABLE scrobble_submission (
	scrobble	INTEGER NOT NULL REFERENCES scrobble(id),
	scrobbler	TEXT NOT NULL,
	created_at	INTEGER NOT NULL DEFAULT (unixepoch()),
	UNIQUE(scrobble, scrobbler)
);
CREATE INDEX scrobble_submission_scrobbler ON scrobble_submission(scrobbler);
CREATE INDEX scrobble_submission_scrobble_scrobbler ON scrobble_submission(scrobble, scrobbler);

-- AUXILIARY VIEWS

CREATE VIEW view_artist_extra (
	id, album_count
) AS
	SELECT artist.id, COUNT(album.id)
	FROM artist
	LEFT JOIN album ON artist.id = album.artist
	GROUP BY artist.id;

CREATE VIEW view_track_extra (
    id,
    duration_ms,
    audio
) AS
	SELECT
		track.id,
		COALESCE(CAST(SUM(CASE WHEN track_audio.preferred = TRUE THEN audio.duration_ms ELSE 0 END) AS INTEGER), 0),
		COALESCE(CAST(MAX(CASE WHEN track_audio.preferred = TRUE THEN audio.id END) AS INTEGER), NULL)
	FROM track
	LEFT JOIN track_audio ON track_audio.track = track.id
	LEFT JOIN audio ON audio.id = track_audio.audio
	GROUP BY track.id;

CREATE VIEW view_album_extra (
	id, track_count, duration_ms
) AS
	SELECT album.id, CAST(COALESCE(COUNT(track.id), 0) AS INTEGER), CAST(COALESCE(SUM(view_track_extra.duration_ms), 0) AS INTEGER)
	FROM album
	LEFT JOIN track ON track.album = album.id
	LEFT JOIN view_track_extra ON view_track_extra.id = track.id
	GROUP BY album.id;

CREATE VIEW view_playlist_extra (
	id, track_count, duration_ms
) AS
	SELECT playlist.id, COALESCE(COUNT(playlist_track.track), 0), COALESCE(CAST(SUM(view_track_extra.duration_ms) AS INTEGER), 0)
	FROM playlist
	LEFT JOIN playlist_track ON playlist_track.playlist = playlist.id
	LEFT JOIN view_track_extra ON view_track_extra.id = playlist_track.track
	GROUP BY playlist.id;

-- SQLX VIEWS

CREATE VIEW sqlx_image (
	id, mime_type, blob_key, blob_size
) AS
	SELECT image.id, image.mime_type, blob.key, blob.size
	FROM image
	INNER JOIN blob ON blob.id = image.blob;

CREATE VIEW sqlx_audio (
	id, bitrate, duration_ms, num_channels, sample_freq, mime_type, filename, blob_key, blob_size
) AS
	SELECT audio.id, bitrate, duration_ms, num_channels, sample_freq, mime_type, filename, blob.key, blob.size
	FROM audio
	INNER JOIN blob ON blob.id = audio.blob;

CREATE VIEW sqlx_artist (
	id, name, listen_count, cover_art, album_count, created_at
) AS
	SELECT artist.id, name, listen_count, cover_art, album_count, created_at
	FROM artist
	INNER JOIN view_artist_extra ON view_artist_extra.id = artist.id;

CREATE VIEW sqlx_album (
	id, name, duration_ms, artist, listen_count, cover_art, track_count, created_at
) AS
	SELECT album.id, name, duration_ms, artist, listen_count, cover_art, track_count, created_at
	FROM album
	INNER JOIN view_album_extra ON view_album_extra.id = album.id;

CREATE VIEW sqlx_track (
	id, name, artist, album, duration_ms, audio, listen_count, cover_art, created_at
) AS
	SELECT track.id, track.name, album.artist, track.album, view_track_extra.duration_ms, view_track_extra.audio, track.listen_count, track.cover_art, track.created_at
	FROM track
	INNER JOIN album ON album.id = track.album
	INNER JOIN view_track_extra ON view_track_extra.id = track.id;

CREATE VIEW sqlx_playlist (
	id, name, owner, track_count, duration_ms, created_at
) AS
	SELECT playlist.id, name, owner, track_count, duration_ms, created_at
	FROM playlist
	INNER JOIN view_playlist_extra ON view_playlist_extra.id = playlist.id;
