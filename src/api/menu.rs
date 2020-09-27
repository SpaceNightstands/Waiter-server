use super::prelude::*;
use model::Product;

pub fn get_service() -> actix_web::Scope{
	web::scope("/menu")
    .route("", web::get().to(get_menu))
    .route("", web::put().to(put_menu))
    .route("/{id}", web::delete().to(delete_menu))
}

async fn get_menu(db: web::Data<MySqlPool>) -> impl Responder {
	let products = sqlx::query_as!(
		Product,
		"SELECT * FROM products"
	).fetch(db.get_ref())
	 .filter_map(
		|item| futures::future::ready(item.ok())
	 ).collect::<Vec<Product>>().await;
	web::Json(products)
}

async fn put_menu(db: web::Data<MySqlPool>, prod: web::Json<Product>) -> Result<impl Responder, Error> {
	//To index into the row with get
	use sqlx::prelude::Row;

	log::debug!("Inserting Product named \"{}\" into product list", prod.name);
	let tx = db.get_ref()
		.begin()
		.await
		.map_err(Error::new)?;
	let product = sqlx::query!(
		"INSERT INTO products(kind, name) VALUES (?, ?) RETURNING id, kind, name",
		prod.kind, prod.name	
	).fetch_one(db.get_ref())
	 .await
	 .map(
		 |item| Product{
				id: item.get(0),
				kind: item.get(1),
				name: item.get(2)
			}
	 ).map_err(Error::new)?;
	tx.commit().await.map_err(Error::new)?;
	Ok(web::Json(product))
}

async fn delete_menu(db: web::Data<MySqlPool>, web::Path(id): web::Path<u32>) -> Result<impl Responder, Error> {
	//To index into the row with get
	use sqlx::prelude::Row;

	log::debug!("Deleting Product {} from product list", id);
	let tx = db.get_ref()
		.begin()
		.await
		.map_err(Error::new)?;
	let product = sqlx::query!(
		"DELETE FROM products WHERE id = ? RETURNING id, kind, name",
		id	
	).fetch_one(db.get_ref())
	 .await
	 .map(
		 |item| Product{
				id: item.get(0),
				kind: item.get(1),
				name: item.get(2)
			}
	 ).map_err(Error::new)?;
	tx.commit().await.map_err(Error::new)?;
	Ok(web::Json(product))
}

//TODO: edit product
