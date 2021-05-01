use super::prelude::{model::Order, *};

pub(crate) fn get_service(filter: Option<filter::SubList>) -> actix_web::Scope {
	let scope = web::scope("/order")
		.route("", web::get().to(get_orders))
		.route("", web::put().to(put_orders));

	let admin_routes = web::scope("")
		.route("/all", web::get().to(get_all_orders))
		.route("/{id}", web::delete().to(set_order_as_done));

	if let Some(filter) = filter {
		scope.service(admin_routes.wrap(filter::SubjectFilter(filter)))
	} else {
		scope.service(admin_routes)
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
					order.cart_mut().push((item.item, item.quantity));
					continue;
				}
			}
			/*If the order array is empty or if the ids are different, create a
				*new order object*/
			$orders.push(Order {
				id: item.id,
				owner: item.owner,
				owner_name: item.owner_name,
				first_term: item.first_term,
				cart: vec![(item.item, item.quantity)],
			});
		}
	};
}

async fn get_orders(db: web::Data<Pool>, req: web::HttpRequest) -> Result<impl Responder, Error> {
	let owner = req.extensions();
	let owner = get_auth_token(&owner)?;

	let mut query = sqlx::query!(
		"SELECT o.id,o.owner,o.owner_name,o.first_term as `first_term: bool`,c.item,c.quantity
		 FROM orders AS o INNER JOIN carts AS c
		 ON o.id=c.order
		 WHERE o.owner=? AND o.done = false
		 ORDER BY id",
		owner.sub()
	)
	.fetch(db.get_ref());

	let mut orders = Vec::<Order>::new();
	stream_to_vec!(query, orders);
	Ok(web::Json(orders))
}

async fn get_all_orders(db: web::Data<Pool>) -> Result<impl Responder, Error> {
	let mut query = sqlx::query!(
		"SELECT o.id,o.owner,o.owner_name,o.first_term as `first_term: bool`,c.item,c.quantity
		 FROM orders AS o INNER JOIN carts AS c
		 ON o.id=c.order
		 WHERE o.done = false
		 ORDER BY id",
	)
	.fetch(db.get_ref());

	let mut orders = Vec::<Order>::new();
	stream_to_vec!(query, orders);
	Ok(web::Json(orders))
}

async fn put_orders(
	db: web::Data<Pool>, order: web::Json<Order>, req: web::HttpRequest,
) -> Result<impl Responder, Error> {
	let owner = req.extensions();
	let owner = get_auth_token(&owner)?;
	if order.cart.len() <= 0 {
		return Err(Error::Static {
			status: actix_web::http::StatusCode::BAD_REQUEST,
			reason: "Request",
			message: "The cart is empty",
		}
		.into());
	}

	log::debug!("Inserting Order");
	let mut tx = db.get_ref().begin().await.map_err(Error::from)?;

	let insert_id = sqlx::query!(
		"INSERT INTO orders(owner, owner_name, first_term) VALUES (?, ?, ?)",
		owner.sub(),
		order.owner_name,
		order.first_term
	)
	.execute(&mut tx)
	.await
	.map_err(Error::from)?
	.last_insert_id();

	for (item, quantity) in order.cart.iter() {
		sqlx::query!(
			"INSERT INTO carts VALUES (?, ?, ?)",
			insert_id,
			item,
			quantity
		)
		.execute(&mut tx)
		.await
		.map_err(Error::from)?;
	}

	tx.commit().await.map_err(Error::from)?;

	let mut order = order.into_inner();
	order.id = insert_id as u32;

	Ok(web::Json(order))
}

async fn set_order_as_done(
	db: web::Data<Pool>, id: web::Path<u32>,
) -> Result<impl Responder, Error> {
	let id = id.into_inner();

	let mut tx = db.get_ref().begin().await.map_err(Error::from)?;

	sqlx::query!("UPDATE orders SET done=true WHERE id=?", id)
		.execute(&mut tx)
		.await
		.map_err(Error::from)?;

	let mut order_cart_stream = sqlx::query!(
		"SELECT o.id,o.owner,o.owner_name,o.first_term as `first_term: bool`,c.item,c.quantity
		 FROM orders AS o INNER JOIN carts AS c
		 ON o.id=c.order
		 WHERE o.id=?",
		id
	)
	.fetch(&mut tx);

	let mut order = {
		let first = order_cart_stream.next().await.ok_or(Error::Static {
			status: actix_web::http::StatusCode::BAD_REQUEST,
			reason: "Database",
			message: "The requested resource does not exist",
		})??;

		Order {
			id: first.id,
			owner: first.owner,
			owner_name: first.owner_name,
			first_term: first.first_term,
			cart: vec![(first.item, first.quantity)],
		}
	};

	//Code edited from stream_to_vec!
	while let Some(item) = order_cart_stream.next().await {
		let item = item?;
		order.cart_mut().push((item.item, item.quantity));
	}

	/*We drop the stream that holds a reference to the transaction
	otherwise, we wouldn't be able to commit*/
	drop(order_cart_stream);
	tx.commit().await.map_err(Error::from)?;

	Ok(web::Json(order))
}

//Utils:
#[inline]
fn get_auth_token<'r>(
	req: &'r std::cell::Ref<'_, actix_web::dev::Extensions>,
) -> Result<&'r AuthToken, Error> {
	req.get::<AuthToken>().ok_or(AuthError())
}

const fn AuthError() -> Error {
	Error::Static {
		status: actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
		reason: "Authentication",
		message: "JWT Token wasn't correctly validated",
	}
}
