use super::prelude::{
	*,
	error::DatabaseError as DBError,
	model::Order
};
use actix_web::Error;

pub fn get_service() -> actix_web::Scope{
	web::scope("/order")
    .route("", web::get().to(get_orders))
    .route("", web::put().to(put_orders))
    .route("/{id}", web::delete().to(delete_orders))
}

async fn get_orders(db: web::Data<MySqlPool>, req: web::HttpRequest) -> Result<impl Responder, Error> {
	let owner = req.extensions();
	let owner = get_auth_token(&owner)?;
	let mut orders = Vec::<Order>::new();

	let mut query = sqlx::query!(
		"SELECT o.id,o.owner,c.item,c.quantity
		 FROM orders AS o INNER JOIN carts AS c
		 ON o.id=c.order
		 WHERE o.owner=?
		 ORDER BY id",
		owner.sub()
	).fetch(db.get_ref());
	while let Some(item) = query.next().await {
		if let Some(item) = result_ok_log(item) {
			if let Some(order) = orders.last_mut() {
				if order.id() == &item.id {
					order.cart.push(
						(item.item, item.quantity)
					);
					continue;
				}
			}
			orders.push(
				Order {
					id: item.id,
					owner: item.owner,
					cart: vec![(item.item, item.quantity)]
				}
			);
		}
	}
	Ok(web::Json(orders))
}

//Order is an array of (item id, quantity) tuples
async fn put_orders(db: web::Data<MySqlPool>, mut cart: web::Json<Vec<(u32, u32)>>, req: web::HttpRequest) -> Result<impl Responder, Error> {
	let owner = req.extensions();
	let owner = get_auth_token(&owner)?;
	if cart.len() <= 0 {
		return Err(
			error::StaticError::new(
				actix_web::http::StatusCode::BAD_REQUEST,
				"The cart is empty"
			).into()
		)
	}

	log::debug!("Inserting Order");
	let mut tx = db.get_ref()
		.begin()
		.await
		.map_err(DBError::from)?;

	let mut order = sqlx::query!(
		"INSERT INTO orders(owner) VALUES (?) RETURNING id, owner",
		owner.sub()
	).fetch_one(&mut tx)
		.await
		.map_err(DBError::from)
    .map(
			|row| Order {
				id: row.get("id"),
				owner: row.get("owner"),
				cart: Vec::new()
			}
		)?;
	for (item, quantity) in cart.drain(..) {
		sqlx::query!(
			"INSERT INTO carts VALUES (?, ?, ?) RETURNING item, quantity",
			order.id(), item, quantity
		).fetch_one(&mut tx)
		 .await
		 .map(
			 |row| order.cart.push(
				 (row.get("item"), row.get("quantity"))
			 )
		 )
		 .map_err(DBError::from)?;
	}

	tx.commit().await.map_err(DBError::from)?;
	Ok(web::Json(order))
}

async fn delete_orders(db: web::Data<MySqlPool>, web::Path(id): web::Path<u32>, req: web::HttpRequest) -> Result<impl Responder, Error> {
	let owner = req.extensions();
	let owner = get_auth_token(&owner)?;

	log::debug!("Deleting Order {} from order list", id);
	let mut tx = db.get_ref()
		.begin()
		.await
		.map_err(DBError::from)?;
	let product = sqlx::query!(
		"DELETE orders, carts FROM orders, carts WHERE orders.id=? AND orders.owner=? AND carts.order=?",
		id, owner.sub(), id
	).fetch_one(&mut tx)
	 .await
	 .map(
		 |row| Order {
			 id: row.get("id"),
			 owner: row.get("owner"),
			 cart: Vec::new(),
		 }
	 ).map_err(DBError::from)?;
	tx.commit().await.map_err(DBError::from)?;

	Ok(web::Json(product))
}

//Utils: 
#[inline]
fn get_auth_token<'r>(req: &'r std::cell::Ref<'_, actix_web::dev::Extensions>) -> Result<&'r AuthToken, error::StaticError>{
	req.get::<AuthToken>()
    .ok_or(AuthError())
}

const fn AuthError() -> error::StaticError {
	error::StaticError::new(
		actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
		"JWT Token wasn't correctly validated"
	)
}

