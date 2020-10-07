-- Add migration script here
CREATE TABLE orders (
	id INT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
	day TIMESTAMP NOT NULL DEFAULT NOW(),
	owner TINYTEXT NOT NULL,
	INDEX(owner)
)
