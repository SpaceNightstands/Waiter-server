-- Add migration script here
CREATE TABLE products (
	id INT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
	kind ENUM('available', 'orderable', 'beverage') NOT NULL,
	name TINYTEXT NOT NULL,
	price SMALLINT UNSIGNED NOT NULL,
	max_num TINYINT UNSIGNED NOT NULL,
	ingredients TEXT NULL,
	image BLOB(2097152) NOT NULL, -- 2MiB
	CHECK(SUBSTRING(image FROM 1 FOR 8) = 0x89504E470D0A1A0A)
)
