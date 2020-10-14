-- Add migration script here
CREATE TABLE carts (
	`order` INT UNSIGNED NOT NULL,
	item INT UNSIGNED NOT NULL,
	quantity INT UNSIGNED NOT NULL,
	FOREIGN KEY(`order`) REFERENCES orders (id) ON DELETE CASCADE,
	FOREIGN KEY(item) REFERENCES products (id)
)
