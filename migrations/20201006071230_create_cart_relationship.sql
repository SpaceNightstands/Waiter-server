-- Add migration script here
CREATE TABLE carts (
	`order` INT UNSIGNED NOT NULL,
	item INT UNSIGNED NOT NULL,
	quantity INT UNSIGNED NOT NULL,
	INDEX(`order`)
)
