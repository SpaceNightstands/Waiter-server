use super::prelude::{
	*,
	model::Product
};
use crate::middleware::filter;

pub fn get_service<T: Into<Option<filter::SubList>>>(filter: T) -> actix_web::Scope{
	let filter = filter.into();
	let scope = web::scope("/menu")
    .route("", web::get().to(get_menu));
	if let Some(filter) = filter {
    scope.service(
			web::scope("")
				.wrap(filter::SubjectFilter(filter))
				.route("", web::put().to(put_menu))
				.route("/{id}", web::delete().to(delete_menu))
		)
	} else {
		scope.route("", web::put().to(put_menu))
			.route("/{id}", web::delete().to(delete_menu))
	}
}

async fn get_menu(db: web::Data<MySqlPool>) -> impl Responder {
	let products = sqlx::query_as(
		"SELECT * FROM products"
	).fetch(db.get_ref())
	 .filter_map(|item| futures::future::ready(result_ok_log(item)))
	 .collect::<Vec<Product>>().await;
	web::Json(products)
}

async fn put_menu(db: web::Data<MySqlPool>, prod: web::Json<Product>) -> Result<impl Responder, Error> {
	log::debug!("Inserting Product named \"{}\" into product list", prod.name());
	let mut tx = db.get_ref()
		.begin()
		.await
		.map_err(Error::from)?;
	let product = sqlx::query!(
		"INSERT INTO products(kind, name, price, max_num, ingredients, image) VALUES (?, ?, ?, ?, ?, ?) RETURNING *",
		prod.kind(), prod.name(),
		prod.price(), prod.max_num(),
		prod.ingredients(), prod.image()
	).fetch_one(&mut tx)
	 .await
	 .map(make_product_from_row)
	 .map_err(Error::from)?;
	tx.commit().await.map_err(Error::from)?;
	Ok(web::Json(product))
}

async fn delete_menu(db: web::Data<MySqlPool>, web::Path(id): web::Path<u32>) -> Result<impl Responder, Error> {
	log::debug!("Deleting Product {} from product list", id);
	let mut tx = db.get_ref()
		.begin()
		.await
		.map_err(Error::from)?;
	let product = sqlx::query!(
		"DELETE FROM products WHERE id = ? RETURNING *",
		id	
	).fetch_one(&mut tx)
	 .await
	 .map(make_product_from_row)
	 .map_err(Error::from)?;
	tx.commit().await.map_err(Error::from)?;
	Ok(web::Json(product))
}

//TODO: edit product
//Utils: 
#[inline]
fn make_product_from_row(item: sqlx::mysql::MySqlRow) -> Product {
	//To index into the row with get
	use sqlx::Row;
	Product{
		id: item.get(0), //"id"
		kind: item.get(1), // "kind"
		name: item.get(2), // "name"
		price: item.get(3), // "price"
		max_num: item.get(4), // "max_num"
		ingredients: item.get(5), // "ingredients"
		image: item.get(6) // "image"
	}
}
