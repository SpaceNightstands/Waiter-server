mod integration;
mod menu;
mod orders;

pub(super) type Pool = sqlx::SqlitePool;

use crate::middleware::auth::Key;
use crate::model::*;
use hmac::NewMac;

static EXAMPLE_PRODUCT: Product = Product {
	id: 1,
	kind: ProductKind::Available,
	name: String::from("Test"),
	price: 100,
	max_num: 3,
	ingredients: None,
	image: vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A],
};

static JWT_KEY: Key = crate::middleware::auth::Key::new_varkey(b"Test").unwrap();

async fn get_database() -> Pool {
	Pool::connect("sqlite::memory:").await.unwrap()
}
