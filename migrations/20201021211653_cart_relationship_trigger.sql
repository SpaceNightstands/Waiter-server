-- Add migration script here
DELIMITER $$
CREATE TRIGGER cap_products
	AFTER INSERT ON carts
	FOR EACH ROW
BEGIN
    IF (SELECT SUM(carts.quantity) <= products.max_num FROM carts JOIN products ON carts.item = products.id WHERE carts.item = NEW.item) = 0 THEN
        SIGNAL SQLSTATE '45001' SET MESSAGE_TEXT='Too many items ordered';
    END IF;
END$$
