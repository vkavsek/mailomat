CREATE TABLE users(
	user_id UUID PRIMARY KEY,
	username TEXT NOT NULL UNIQUE,
	password_hash TEXT NOT NULL
)
