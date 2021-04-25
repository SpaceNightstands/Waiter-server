-- Add migration script here
ALTER TABLE orders ADD COLUMN done BOOLEAN NOT NULL DEFAULT false

