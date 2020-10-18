use super::prelude::{
	*,
	error::DatabaseError as DBError,
	model::Product
};

pub fn get_service() -> actix_web::Scope{
	web::scope("/menu")
    .route("", web::get().to(get_menu))
    .route("", web::put().to(put_menu))
    .route("/{id}", web::delete().to(delete_menu))
}

async fn get_menu(db: web::Data<MySqlPool>) -> impl Responder {
	let products = sqlx::query_as(
		"SELECT * FROM products"
	).fetch(db.get_ref())
	 .filter_map(|item| futures::future::ready(result_ok_log(item)))
	 .collect::<Vec<Product>>().await;
	web::Json(products)
}

async fn put_menu(db: web::Data<MySqlPool>, prod: web::Json<Product>) -> Result<impl Responder, DBError> {
	log::debug!("Inserting Product named \"{}\" into product list", prod.name());
	let mut tx = db.get_ref()
		.begin()
		.await
		.map_err(DBError::from)?;
	let product = sqlx::query!(
		"INSERT INTO products(kind, name, price, max_num, ingredients, image) VALUES (?, ?, ?, ?, ?, ?) RETURNING *",
		prod.kind(), prod.name(),
		prod.price(), prod.max_num(),
		prod.ingredients(), prod.image()
	).fetch_one(&mut tx)
	 .await
	 .map(make_product_from_row)
	 .map_err(DBError::from)?;
	tx.commit().await.map_err(DBError::from)?;
	Ok(web::Json(product))
}

async fn delete_menu(db: web::Data<MySqlPool>, web::Path(id): web::Path<u32>) -> Result<impl Responder, DBError> {
	log::debug!("Deleting Product {} from product list", id);
	let mut tx = db.get_ref()
		.begin()
		.await
		.map_err(DBError::from)?;
	let product = sqlx::query!(
		"DELETE FROM products WHERE id = ? RETURNING *",
		id	
	).fetch_one(&mut tx)
	 .await
	 .map(make_product_from_row)
	 .map_err(DBError::from)?;
	tx.commit().await.map_err(DBError::from)?;
	Ok(web::Json(product))
}

//TODO: edit product
//Utils: 
#[inline]
fn make_product_from_row(item: sqlx::mysql::MySqlRow) -> Product {
	//To index into the row with get
	use sqlx::prelude::Row;
	Product{
		id: item.get("id"),
		kind: item.get("kind"),
		name: item.get("name"),
		price: item.get("price"),
		max_num: item.get("max_num"),
		ingredients: item.get("ingredients"),
		image: item.get("image")
	}
}
