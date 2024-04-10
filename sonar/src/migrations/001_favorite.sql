CREATE TABLE favorite (
	user		INTEGER NOT NULL REFERENCES user(id),
	namespace	INTEGER NOT NULL,
	identifier	INTEGER NOT NULL,
	created_at	INTEGER NOT NULL DEFAULT(unixepoch()),
	PRIMARY KEY(user, namespace, identifier)
);
CREATE INDEX favorite_user ON favorite(user);
