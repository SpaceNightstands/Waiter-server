use super::prelude::{model::Product, *};

pub(crate) fn get_service(filter: Option<filter::SubList>) -> actix_web::Scope {
	let scope = web::scope("/menu").route("", web::get().to(get_menu));

	let protectable_routes = web::scope("")
		.route("", web::put().to(put_menu))
		.route("/{id}", web::delete().to(delete_menu));

	if let Some(filter) = filter {
		scope.service(protectable_routes.wrap(filter::SubjectFilter(filter)))
	} else {
		scope.service(protectable_routes)
	}
}

async fn get_menu(db: web::Data<Pool>) -> Result<impl Responder, Error> {
	let mut tx = db.get_ref().begin().await.map_err(Error::from)?;
	//Get count of products in the database table
	let product_count = sqlx::query!("SELECT COUNT(*) as count FROM products",)
		.fetch_one(&mut tx)
		.await?
		.count;

	//Prealloc the space
	let mut products = Vec::with_capacity(product_count as usize);
	let mut prod_stream =
		sqlx::query_as_unchecked!(Product, "SELECT * FROM products").fetch(&mut tx);

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

async fn put_menu(db: web::Data<Pool>, prod: web::Json<Product>) -> Result<impl Responder, Error> {
	log::debug!(
		"Inserting Product named \"{}\" into product list",
		prod.name()
	);
	let mut tx = db.get_ref().begin().await.map_err(Error::from)?;

	let insert_id = sqlx::query!(
		"INSERT INTO products(kind, name, price, max_num, ingredients, image) VALUES (?, ?, ?, ?, ?, ?)",
		prod.kind(), prod.name(),
		prod.price(), prod.max_num(),
		prod.ingredients(), prod.image()
	)
	.execute(&mut tx)
	.await
	.map_err(Error::from)?
	.last_insert_id();

	tx.commit().await.map_err(Error::from)?;

	let mut product = prod.into_inner();
	product.id = insert_id as u32;
	Ok(web::Json(product))
}

async fn delete_menu(
	db: web::Data<Pool>,
	web::Path(id): web::Path<u32>,
) -> Result<impl Responder, Error> {
	log::debug!("Deleting Product {} from product list", id);
	let mut tx = db.get_ref().begin().await.map_err(Error::from)?;

	let product = sqlx::query_as!(
		Product,
		"SELECT
			id, kind as `kind: crate::model::ProductKind`,
			name, price, max_num, ingredients, image
		FROM products WHERE id = ?",
		id
	)
	.fetch_one(&mut tx)
	.await
	.map_err(Error::from)?;

	sqlx::query!("DELETE FROM products WHERE id = ?", id)
		.execute(&mut tx)
		.await
		.map_err(Error::from)?;

	tx.commit().await.map_err(Error::from)?;
	Ok(web::Json(product))
}

//TODO: edit product
