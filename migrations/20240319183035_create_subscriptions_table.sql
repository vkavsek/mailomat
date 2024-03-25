-- Add migration script here
CREATE TABLE subscriptions (
	id UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
	email TEXT NOT NULL UNIQUE,
	name TEXT NOT NULL,
	subscribed_at TIMESTAMPTZ NOT NULL
)
