use super::prelude::*;
use model::Order;

pub fn get_service() -> actix_web::Scope{
	web::scope("/order")
    .route("", web::get().to(get_orders))
}

async fn get_orders(db: web::Data<MySqlPool>) -> impl Responder {
	let orders = sqlx::query_as!(
		Order,
		"SELECT * FROM orders"
	).fetch(db.get_ref())
	 .filter_map(
		|item| futures::future::ready(result_ok_log(item))
	 ).collect::<Vec<_>>().await;
	web::Json(orders)
}

//TODO: add order
//TODO: delete order
