-- Add migration script here
ALTER TABLE orders ADD COLUMN owner_name TEXT NOT NULL
