-- Add migration script here
CREATE TABLE products (
	id INT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
	kind ENUM('available', 'orderable', 'beverage') NOT NULL,
	name TINYTEXT NOT NULL,
	price SMALLINT UNSIGNED NOT NULL,
	max_num TINYINT UNSIGNED NOT NULL,
	ingredients TEXT NULL
)
