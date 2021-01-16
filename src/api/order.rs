use super::prelude::{
	*,
	model::Order
};
use sqlx::Row;

pub(crate) fn get_service() -> actix_web::Scope{
	web::scope("/order")
    .route("", web::get().to(get_orders))
    .route("", web::put().to(put_orders))
	//.route("/{id}", web::delete().to(delete_orders))
}

async fn get_orders(db: web::Data<MySqlPool>, req: web::HttpRequest) -> Result<impl Responder, Error> {
	let owner = req.extensions();
	let owner = get_auth_token(&owner)?;
	let mut orders = Vec::<Order>::new();

	let mut query = sqlx::query!(
		"SELECT o.id,o.owner,o.owner_name,c.item,c.quantity
		 FROM orders AS o INNER JOIN carts AS c
		 ON o.id=c.order
		 WHERE o.owner=?
		 ORDER BY id",
		owner.sub()
	).fetch(db.get_ref());

	//Iterate through all (id, owner, owner_name, cartItem[n].id, cartItem[n].quantity) tuples
	while let Some(item) = query.next().await {
		//If the read throws, return the error
		let item = item?;
		//If the order array has at least one item, get a mutable reference
		if let Some(order) = orders.last_mut() {
			/*If the last array element has the same id as the query,
			 *add the (cartItem[n].id, cartItem[n].quantity) tuple to the cart*/
			if order.id() == &item.id {
				order.cart_mut().push(
					(item.item, item.quantity)
				);
				continue;
			}
		}
		/*If the order array is empty or if the ids are different, create a
		 *new order object*/
		orders.push(
			Order {
				id: item.id,
				owner: item.owner,
				owner_name: item.owner_name,
				cart: vec![(item.item, item.quantity)]
			}
		);
	}
	Ok(web::Json(orders))
}

#[derive(serde::Deserialize, serde::Serialize)]
pub(crate) struct PutOrder {
	pub(crate) owner_name: String,
	//cart is an array of (item id, quantity) tuples
	pub(crate) cart: Vec<(u32, u32)>
}

async fn put_orders(db: web::Data<MySqlPool>, mut put_order: web::Json<PutOrder>, req: web::HttpRequest) -> Result<impl Responder, Error> {
	let owner = req.extensions();
	let owner = get_auth_token(&owner)?;
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
		"INSERT INTO orders(owner, owner_name) VALUES (?, ?) RETURNING id, owner, owner_name",
		owner.sub(), put_order.owner_name
	).fetch_one(&mut tx)
		.await
		.map_err(Error::from)
		//Map the returning order into an object
    .map(
			|row| Order {
				id: row.get(0),
				owner: row.get(1),
				owner_name: row.get(2),
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
				/*Map the returning cart item into a tuple
				 *and then push it in the "cart" vector*/
				|row| order.cart_mut().push(
					(row.get(0), row.get(1))
				)
		 )
		 .map_err(Error::from)?;
	}

	tx.commit().await.map_err(Error::from)?;
	Ok(web::Json(order))
}

//Utils: 
#[inline]
fn get_auth_token<'r>(req: &'r std::cell::Ref<'_, actix_web::dev::Extensions>) -> Result<&'r AuthToken, Error>{
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

