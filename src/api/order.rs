use super::prelude::{
	*,
	model::Order
};
use sqlx::Row;

pub fn get_service() -> actix_web::Scope{
	web::scope("/order")
    .route("", web::get().to(get_orders))
    .route("", web::put().to(put_orders))
	//.route("/{id}", web::delete().to(delete_orders))
}

async fn get_orders(db: web::Data<MySqlPool>, req: web::HttpRequest) -> Result<impl Responder, Error> {
	let owner = req.extensions();
	let owner = get_auth_token(owner)?;
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
					order.cart_mut().push(
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

#[derive(serde::Deserialize)]
struct PutOrder {
	owner_name: String,
	//cart is an array of (item id, quantity) tuples
	cart: Vec<(u32, u32)>
}

async fn put_orders(db: web::Data<MySqlPool>, mut put_order: web::Json<PutOrder>, req: web::HttpRequest) -> Result<impl Responder, Error> {
	let owner = req.extensions();
	let owner = get_auth_token(owner)?;
	if put_order.cart.len() <= 0 {
		return Err(
			Error::Static{
				status: actix_web::http::StatusCode::BAD_REQUEST,
				reason: "Request",
				message: "The cart is empty"
			}.into()
		)
	}

	log::debug!("Inserting Order");
	let mut tx = db.get_ref()
		.begin()
		.await
		.map_err(Error::from)?;

	let mut order = sqlx::query!(
		"INSERT INTO orders(owner, owner_name) VALUES (?, ?) RETURNING id, owner",
		owner.sub(), put_order.owner_name
	).fetch_one(&mut tx)
		.await
		.map_err(Error::from)
    .map(
			|row| Order {
				id: row.get(0),
				owner: row.get(1),
				cart: Vec::new()
			}
		)?;
	log::debug!("{:?}", order);
	for (item, quantity) in put_order.cart.drain(..) {
		sqlx::query!(
			"INSERT INTO carts VALUES (?, ?, ?) RETURNING item, quantity",
			order.id(), item, quantity
		).fetch_one(&mut tx)
		 .await
		 .map(
			 |row| order.cart_mut().push(
				 (row.get(0), row.get(1))
			 )
		 )
		 .map_err(Error::from)?;
	}

	tx.commit().await.map_err(Error::from)?;
	Ok(web::Json(order))
}

/*async fn delete_orders(db: web::Data<MySqlPool>, web::Path(id): web::Path<u32>, req: web::HttpRequest) -> Result<impl Responder, Error> {
	let owner = req.extensions();
	let owner = get_auth_token(&owner)?;

	log::debug!("Deleting Order {} from order list", id);
	let mut tx = db.get_ref()
		.begin()
		.await
		.map_err(DBError::from)?;
	let product = sqlx::query!(
		"DELETE orders FROM orders WHERE orders.id=? AND orders.owner=?",
		id, owner.sub()
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
}*/

//Utils: 
#[inline]
fn get_auth_token<'r>(req: std::cell::Ref<'r, actix_web::dev::Extensions>) -> Result<&'r AuthToken, Error>{
	req.get::<AuthToken>()
    .ok_or(AuthError())
}

const fn AuthError() -> Error {
	Error::Static{
		status: actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
		reason: "Authentication",
		message: "JWT Token wasn't correctly validated"
	}
}

