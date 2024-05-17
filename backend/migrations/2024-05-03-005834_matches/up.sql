-- Your SQL goes here
CREATE TABLE matches (
	
	user1 TEXT NOT NULL,
	user2 TEXT NOT NULL,
	
	state INT NOT NULL CHECK (state IN (
		0, /* dead */
		1, /* active */
		2, /* 1 liked 2 */
		3  /* 2 liked 1 */
	)),
	
	PRIMARY KEY (user1, user2),
	FOREIGN KEY (user1) REFERENCES users(id) ON DELETE CASCADE,
	FOREIGN KEY (user2) REFERENCES users(id) ON DELETE CASCADE,
	
	CHECK (user1 < user2)
	
);