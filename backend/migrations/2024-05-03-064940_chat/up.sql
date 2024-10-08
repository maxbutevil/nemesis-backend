-- Your SQL goes here
CREATE TABLE messages (
	
	/* id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL, *//* TEMPORARY */
	id TEXT NOT NULL PRIMARY KEY,
	user1 TEXT NOT NULL, /* REFERENCES users(id) ON DELETE CASCADE, */
	user2 TEXT NOT NULL, /* REFERENCES users(id) ON DELETE CASCADE, */
	sender INT NOT NULL,
	
	timestamp TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP NOT NULL,
	content TEXT NOT NULL,
	
	/* PRIMARY KEY (user1, user2, id), */
	FOREIGN KEY (user1) REFERENCES users(id) ON DELETE CASCADE,
	FOREIGN KEY (user2) REFERENCES users(id) ON DELETE CASCADE,
	CHECK (user1 < user2)
	
);

CREATE INDEX messages_idx ON messages(user1, user2, timestamp);

