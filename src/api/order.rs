use super::prelude::{
	*,
	DatabaseError as DBError
};
use model::Order;

pub fn get_service() -> actix_web::Scope{
	web::scope("/order")
    .route("", web::get().to(get_orders))
    .route("", web::put().to(put_orders))
    .route("/{id}", web::delete().to(delete_orders))
}

async fn get_orders(db: web::Data<MySqlPool>) -> impl Responder {
	let orders = sqlx::query_as!(
		Order,
		"SELECT id,owner,cart FROM orders"
	).fetch(db.get_ref())
	 .filter_map(
		|item| futures::future::ready(result_ok_log(item))
	 ).collect::<Vec<_>>().await;
	web::Json(orders)
}

#[derive(serde::Deserialize)]
struct InsertableOrder {
	idempotency: String,
	cart: Vec<u32>
}

async fn put_orders(db: web::Data<MySqlPool>, order: web::Json<InsertableOrder>, req: web::HttpRequest) -> Result<impl Responder, DBError> {
	log::debug!("Inserting Order");
	let tx = db.get_ref()
		.begin()
		.await
		.map_err(DBError::from)?;
	let extensions = req.extensions();
	let orders = sqlx::query!(
		"INSERT INTO orders(owner, cart) VALUES (?, ?) RETURNING id, owner, cart",
		extensions.get::<AuthToken>()
			.unwrap()
			.account_id(),
		serde_json::to_string(&order.cart).unwrap()
	).fetch_one(db.get_ref())
	 .await
	 .map(make_order_from_row)
	 .map_err(DBError::from)?;
	tx.commit().await.map_err(DBError::from)?;
	Ok(web::Json(orders))
}

async fn delete_orders(db: web::Data<MySqlPool>, web::Path(id): web::Path<u32>) -> Result<impl Responder, DBError> {
	log::debug!("Deleting Order {} from order list", id);
	let tx = db.get_ref()
		.begin()
		.await
		.map_err(DBError::from)?;
	let product = sqlx::query!(
		"DELETE FROM orders WHERE id = ? RETURNING id, owner, cart",
		id	
	).fetch_one(db.get_ref())
	 .await
	 .map(make_order_from_row)
	 .map_err(DBError::from)?;
	tx.commit().await.map_err(DBError::from)?;
	Ok(web::Json(product))
}

//Utils: 
fn make_order_from_row(item: sqlx::mysql::MySqlRow) -> Order {
	//To index into the row with get
	use sqlx::prelude::Row;
	Order{
		id: item.get("id"),
		owner: item.get("owner"),
		cart: item.get("cart")
	}
}
