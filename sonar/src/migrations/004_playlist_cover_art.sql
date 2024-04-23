ALTER TABLE playlist ADD cover_art INTEGER REFERENCES image(id);

DROP VIEW sqlx_playlist;
CREATE VIEW sqlx_playlist (
	id, name, owner, track_count, duration_ms, cover_art, created_at
) AS
	SELECT playlist.id, name, owner, track_count, duration_ms, cover_art, created_at
	FROM playlist
	INNER JOIN view_playlist_extra ON view_playlist_extra.id = playlist.id;
