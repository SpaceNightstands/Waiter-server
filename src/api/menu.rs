use super::prelude::*;

pub fn get_service() -> actix_web::Scope{
	web::scope("/menu")
    .route("", web::get().to(get_menu))
    .route("", web::put().to(put_menu))
}

async fn get_menu(db: web::Data<MySqlPool>) -> impl Responder {
	let products = sqlx::query_as!(
		model::Product,
		"SELECT * FROM products"
	).fetch(db.get_ref())
	 .filter_map(
		 |item| futures::future::ready(
				match item {
					Ok(item) => Some(format!("{:?}", item)),
					Err(_) => None
				}
		 )
	 ).collect::<Vec<String>>().await;
	web::Json(products)
}

async fn put_menu(db: web::Data<MySqlPool>, name: String) -> impl Responder {
	use sqlx::prelude::Row;

	log::debug!("Inserting Product {} into product list", name);
	// TODO: avoid unwrap
	let tx = db.get_ref().begin().await.unwrap();
	let product = sqlx::query!(
		"INSERT INTO products(name) VALUES (?) RETURNING id, name",
		name	
	).fetch_one(db.get_ref())
	 .await
	 .map(
		 |item| model::Product{
				id: item.get(0),
				name: item.get(1)
			}
	 ).unwrap();
	tx.commit().await.unwrap();
	web::Json(product)
}

//TODO: delete product
//TODO: edit product
