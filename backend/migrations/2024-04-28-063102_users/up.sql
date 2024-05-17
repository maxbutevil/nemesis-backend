-- Your SQL goes here

CREATE TABLE users (
	
	id TEXT PRIMARY KEY NOT NULL,
	
	/* PRIVATE */
	latitude REAL,
	longitude REAL,
	
	birth_date TEXT,
	
	/* VITALS */
	name TEXT,
	gender_identity TEXT,
	pronouns TEXT,
	
	/* PROFILE */
	bio TEXT,
	looking_for TEXT,
	
	interests TEXT,
	
	photos TEXT
	
);
