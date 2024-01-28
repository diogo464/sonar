CREATE TABLE property (
	namespace 	INTEGER NOT NULL,
	id			INTEGER NOT NULL,
	key			TEXT NOT NULL,
	value		TEXT NOT NULL,
	created_at	INTEGER NOT NULL DEFAULT (unixepoch()),
	PRIMARY KEY (namespace, id, key)
);

CREATE TABLE genre (
	namespace	INTEGER NOT NULL,
	id			INTEGER NOT NULL,
	genre		TEXT NOT NULL,
	created_at	INTEGER NOT NULL DEFAULT (unixepoch()),
	PRIMARY KEY (namespace, id, genre)
);

CREATE TABLE image (
	id			INTEGER PRIMARY KEY,
	mime_type	TEXT NOT NULL,
	blob_key	TEXT NOT NULL,
	created_at	INTEGER NOT NULL DEFAULT (unixepoch())
);

CREATE TABLE user (
	id				INTEGER PRIMARY KEY,
	username		TEXT NOT NULL UNIQUE,
	-- scrypt PHC string
	password_hash	TEXT NOT NULL DEFAULT '',
	avatar			INTEGER REFERENCES image(id),
	created_at		INTEGER NOT NULL DEFAULT (unixepoch())
);

CREATE TABLE artist (
	id				INTEGER PRIMARY KEY,
	name 			TEXT NOT NULL,
	listen_count	INTEGER NOT NULL DEFAULT 0,
	cover_art		INTEGER REFERENCES image(id),
	created_at		INTEGER NOT NULL DEFAULT (unixepoch())
);

CREATE TABLE album (
	id				INTEGER PRIMARY KEY,
	name			TEXT NOT NULL,
	artist			INTEGER NOT NULL REFERENCES artist(id),
	release_date	TEXT NOT NULL,
	listen_count	INTEGER NOT NULL DEFAULT 0,
	cover_art		INTEGER REFERENCES image(id),
	created_at		INTEGER NOT NULL DEFAULT (unixepoch())
);

CREATE TABLE track (
	id				INTEGER PRIMARY KEY,
	name			TEXT NOT NULL DEFAULT '',
	album			INTEGER NOT NULL REFERENCES album(id),
	disc_number		INTEGER NOT NULL,
	track_number	INTEGER NOT NULL,
	duration_ms		INTEGER NOT NULL,
	listen_count	INTEGER NOT NULL DEFAULT 0,
	cover_art		INTEGER REFERENCES image(id),
	-- S : Synced lyrics
	-- U : Unsynced lyrics
	lyrics_kind		TEXT CHECK (lyrics_kind IS NULL OR lyrics_kind IN ('S', 'U')),
	audio_blob_key	TEXT NOT NULL,
	audio_filename	TEXT NOT NULL DEFAULT '',
	created_at		INTEGER NOT NULL DEFAULT (unixepoch())
);

CREATE TABLE track_lyrics_line (
	track		INTEGER NOT NULL REFERENCES track(id),
	offset		INTEGER NOT NULL,
	text		TEXT NOT NULL
);

CREATE TABLE playlist (
	id			INTEGER PRIMARY KEY,
	owner		INTEGER NOT NULL REFERENCES user(id),
	name		TEXT NOT NULL DEFAULT '',
	created_at	INTEGER NOT NULL DEFAULT (unixepoch())
);

CREATE TABLE playlist_track (
	playlist	INTEGER NOT NULL REFERENCES playlist(id),
	track		INTEGER NOT NULL REFERENCES track(id),
	created_at	INTEGER NOT NULL DEFAULT (unixepoch()),
	PRIMARY KEY(playlist, track)
);

CREATE TABLE scrobble (
	id				INTEGER PRIMARY KEY,
	user			INTEGER NOT NULL REFERENCES user(id),
	track			INTEGER NOT NULL REFERENCES track(id),
	listen_at		INTEGER NOT NULL,
	listen_secs		INTEGER NOT NULL,
	listen_device	TEXT NOT NULL DEFAULT '',
	created_at		INTEGER NOT NULL DEFAULT (unixepoch())
);

CREATE VIEW artist_album_count (
	id, album_count	
) AS
	SELECT artist.id, COUNT(album.id)
	FROM artist
	LEFT JOIN album ON artist.id = album.artist
	GROUP BY artist.id;

CREATE VIEW artist_view (
	id, name, listen_count, cover_art, album_count
) AS
	SELECT artist.id, name, listen_count, cover_art, album_count
	FROM artist
	INNER JOIN artist_album_count ON artist.id = artist_album_count.id;

CREATE VIEW album_track_count (
	id, track_count	
) AS
	SELECT album.id, COUNT(track.id)
	FROM album
	LEFT JOIN track ON album.id = track.album
	GROUP BY album.id;

CREATE VIEW album_view (
	id, name, artist, release_date, listen_count, cover_art, track_count
) AS
	SELECT album.id, name, artist, release_date, listen_count, cover_art, track_count
	FROM album
	INNER JOIN album_track_count ON album.id = album_track_count.id;

CREATE VIEW track_artist (
	id, artist
) AS
	SELECT track.id id, album.artist
	FROM track
	INNER JOIN album ON track.album = album.id;

CREATE VIEW track_view (
	id, name, artist, album, disc_number, track_number, duration_ms, listen_count, cover_art
) AS
	SELECT track.id, name, artist, album, disc_number, track_number, duration_ms, listen_count, cover_art
	FROM track
	INNER JOIN track_artist ON track.id = track_artist.id;

CREATE VIEW playlist_track_count (
	id, track_count
) AS
	SELECT playlist.id, COUNT(playlist_track.track)
	FROM playlist
	LEFT JOIN playlist_track ON playlist_track.playlist = playlist.id
	GROUP BY playlist.id;

CREATE VIEW playlist_view (
	id, name, owner, track_count
) AS
	SELECT playlist.id, name, owner, track_count
	FROM playlist
	INNER JOIN playlist_track_count ON playlist_track_count.id = playlist.id;
