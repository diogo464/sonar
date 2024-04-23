DROP TABLE subscription;
CREATE TABLE subscription (
	id		INTEGER PRIMARY KEY NOT NULL,
	user		INTEGER NOT NULL REFERENCES user(id),
	created_at	INTEGER NOT NULL DEFAULT (unixepoch()),
	last_submitted	INTEGER, -- unix timestamp in seconds
	interval_sec	INTEGER,
	description	TEXT,
	artist		TEXT,
	album		TEXT,
	track		TEXT,
	playlist	TEXT,
	external_id	TEXT,
	media_type	TEXT CHECK (media_type IS NULL OR media_type IN ('artist', 'album', 'track', 'playlist'))
);
CREATE INDEX subscription_user ON subscription(user);
