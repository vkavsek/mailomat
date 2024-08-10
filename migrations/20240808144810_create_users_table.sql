CREATE TABLE users(
	user_id UUID PRIMARY KEY,
	username TEXT NOT NULL UNIQUE,
	pwd_salt UUID NOT NULL,
	password TEXT NOT NULL
)
