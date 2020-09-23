-- Add migration script here
CREATE TABLE orders (
	id INT NOT NULL AUTO_INCREMENT PRIMARY KEY,
	owner TINYTEXT NOT NULL,
	cart JSON DEFAULT "{}"
)
