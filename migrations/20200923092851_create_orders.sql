-- Add migration script here
CREATE TABLE orders (
	id INT NOT NULL AUTO_INCREMENT PRIMARY KEY,
	day DATE NOT NULL DEFAULT CURDATE(),
	owner TINYTEXT NOT NULL,
	cart JSON DEFAULT "[]",
	INDEX(owner)
)
