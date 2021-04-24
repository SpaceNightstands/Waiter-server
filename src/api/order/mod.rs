use super::prelude::{
	*,
	model::Order
};

pub(crate) fn get_service(filter: Option<filter::SubList>) -> actix_web::Scope{
	let scope = web::scope("/order")
    .route("", web::get().to(get_orders))
    .route("", web::put().to(put_orders));

	let all_orders_service = web::resource("/all").route(web::get().to(get_all_orders));

	if let Some(filter) = filter {
		scope.service(all_orders_service.wrap(filter::SubjectFilter(filter)))
	} else {
		scope.service(all_orders_service)
	}
}

//TODO: Turn into function/use stream adapters
macro_rules! stream_to_vec {
	($query: ident, $orders: ident) => {
		//Iterate through all (id, owner, owner_name, cartItem[n].id, cartItem[n].quantity) tuples
		while let Some(item) = $query.next().await {
			//If the read throws, return the error
			let item = item?;
			//If the order array has at least one item, get a mutable reference
			if let Some(order) = $orders.last_mut() {
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
			$orders.push(
				Order {
					id: item.id,
					owner: item.owner,
					owner_name: item.owner_name,
					cart: vec![(item.item, item.quantity)]
				}
			);
		}
	}
}

async fn get_orders(db: web::Data<Pool>, req: web::HttpRequest) -> Result<impl Responder, Error> {
	let owner = req.extensions();
	let owner = get_auth_token(&owner)?;

	let mut query = sqlx::query!(
		"SELECT o.id,o.owner,o.owner_name,c.item,c.quantity
		 FROM orders AS o INNER JOIN carts AS c
		 ON o.id=c.order
		 WHERE o.owner=?
		 ORDER BY id",
		owner.sub()
	).fetch(db.get_ref());

	let mut orders = Vec::<Order>::new();
	stream_to_vec!(query, orders);
	Ok(web::Json(orders))
}

async fn get_all_orders(db: web::Data<Pool>) -> Result<impl Responder, Error> {
	let mut query = sqlx::query!(
		"SELECT o.id,o.owner,o.owner_name,c.item,c.quantity
		 FROM orders AS o INNER JOIN carts AS c
		 ON o.id=c.order
		 ORDER BY id",
	).fetch(db.get_ref());

	let mut orders = Vec::<Order>::new();
	stream_to_vec!(query, orders);
	Ok(web::Json(orders))
}

#[derive(serde::Deserialize, serde::Serialize)]
struct PutOrder {
	owner_name: String,
	//cart is an array of (item id, quantity) tuples
	cart: Vec<(u32, u32)>
}

async fn put_orders(db: web::Data<Pool>, put_order: web::Json<PutOrder>, req: web::HttpRequest) -> Result<impl Responder, Error> {
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

	let insert_id = sqlx::query!(
		"INSERT INTO orders(owner, owner_name) VALUES (?, ?)",
		owner.sub(), put_order.owner_name
	).execute(&mut tx).await
    .map_err(Error::from)?
		.last_insert_id();

	for (item, quantity) in put_order.cart.iter() {
		sqlx::query!(
			"INSERT INTO carts VALUES (?, ?, ?)",
			insert_id, item, quantity
		).execute(&mut tx)
		 .await
		 .map_err(Error::from)?;
	}

	tx.commit().await.map_err(Error::from)?;

	let put_order = put_order.into_inner();
	let order = Order {
		id: insert_id as u32,
		//TODO: Find a better way to pass the owner id
		owner: owner.sub().clone(),
		owner_name: put_order.owner_name,
		cart: put_order.cart
	};

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

#[cfg(test)]
mod test;
