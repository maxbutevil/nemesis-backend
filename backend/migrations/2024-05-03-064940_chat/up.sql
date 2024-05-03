-- Your SQL goes here
CREATE TABLE messages (
	
	id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL, /* TEMPORARY */
	user1 TEXT NOT NULL,
	user2 TEXT NOT NULL,
	sender INT NOT NULL,
	
	time TEXT DEFAULT CURRENT_TIMESTAMP NOT NULL,
	content TEXT NOT NULL,
	
	/* PRIMARY KEY (user1, user2, id), */
	FOREIGN KEY (user1) REFERENCES users(id) ON DELETE CASCADE,
	FOREIGN KEY (user2) REFERENCES users(id) ON DELETE CASCADE,
	CHECK (user1 < user2)
	
) STRICT;

CREATE INDEX messages_idx ON messages(user1, user2, time);

