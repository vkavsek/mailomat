-- Add migration script here
ALTER TABLE subscriptions ADD COLUMN status TEXT NULL;

-- CREATE TABLE subscription_tokens (
-- 	token UUID NOT NULL PRIMARY KEY,
-- 	id UUID NOT NULL,
-- )
