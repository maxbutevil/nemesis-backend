-- Your SQL goes here
CREATE TABLE impressions (
	
	user_id TEXT NOT NULL,
	profile_id TEXT NOT NULL,
	
	liked INT NOT NULL,
	
	PRIMARY KEY (user_id, profile_id),
	FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
	FOREIGN KEY (profile_id) REFERENCES users(id) ON DELETE CASCADE,
	
	CHECK(user_id != profile_id)
	
) STRICT;