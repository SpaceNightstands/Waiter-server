use super::prelude::{
	*,
	model::Product
};
use crate::middleware::filter;

pub(crate) fn get_service(filter: Option<filter::SubList>) -> actix_web::Scope{
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

async fn get_menu(db: web::Data<MySqlPool>) -> Result<impl Responder, Error> {
	let mut tx = db.get_ref()
		.begin().await
		.map_err(Error::from)?;
	//Get count of products in the database table
	let product_count = sqlx::query!(
		"SELECT COUNT(*) as count FROM products",
	).fetch_one(&mut tx).await?
		.count;

	//Prealloc the space
	let mut products = Vec::with_capacity(product_count as usize);
	let mut prod_stream = sqlx::query_as(
		"SELECT * FROM products"
	).fetch(&mut tx);
	while let Some(prod) = prod_stream.next().await {
		let prod: Product = prod?;
		products.push(prod);
	}
	/*Drop the stream to drop the transaction reference.
	 *If this isn't done, commit won't be able to move the 
	 *object out of the binding*/
	drop(prod_stream);

	tx.commit().await.map_err(Error::from)?;
	Ok(web::Json(products))
}

async fn put_menu(db: web::Data<MySqlPool>, prod: web::Json<Product>) -> Result<impl Responder, Error> {
	log::debug!("Inserting Product named \"{}\" into product list", prod.name());
	let mut tx = db.get_ref()
		.begin().await
		.map_err(Error::from)?;
	let product = sqlx::query!(
		"INSERT INTO products(kind, name, price, max_num, ingredients, image) VALUES (?, ?, ?, ?, ?, ?) RETURNING *",
		prod.kind(), prod.name(),
		prod.price(), prod.max_num(),
		prod.ingredients(), prod.image()
	).fetch_one(&mut tx).await
	 .map(make_product_from_row)
	 .map_err(Error::from)?;
	tx.commit().await.map_err(Error::from)?;
	Ok(web::Json(product))
}

async fn delete_menu(db: web::Data<MySqlPool>, web::Path(id): web::Path<u32>) -> Result<impl Responder, Error> {
	log::debug!("Deleting Product {} from product list", id);
	let mut tx = db.get_ref()
		.begin().await
		.map_err(Error::from)?;
	let product = sqlx::query!(
		"DELETE FROM products WHERE id = ? RETURNING *",
		id	
	).fetch_one(&mut tx).await
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
	//TODO: use column names instead of positions
	Product {
		id: item.get(0),
		kind: item.get(1),
		name: item.get(2),
		price: item.get(3),
		max_num: item.get(4),
		ingredients: item.get(5),
		image: item.get(6)
	}
}
