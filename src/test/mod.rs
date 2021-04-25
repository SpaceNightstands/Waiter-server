mod menu;
mod orders;
mod integration;

pub(super) type Pool = sqlx::SqlitePool;

use crate::model::*;

static EXAMPLE_PRODUCT: Product = Product {
	id: 1,
	kind: ProductKind::Available,
	name: String::from("Test"),
	price: 100,
	max_num: 3,
	ingredients: None,
	image: vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]
};

async fn get_database() -> Pool {
	Pool::connect("sqlite::memory:").await.unwrap()
}

