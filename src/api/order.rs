use super::prelude::*;

pub fn get_service() -> actix_web::Scope{
	web::scope("/order")
    .route("", web::get().to(get_menu))
}

async fn get_orders(db: web::Data<MySqlPool>) -> impl Responder {
	let products = sqlx::query_as!(
		model::Product,
		"SELECT * FROM products"
	).fetch(db.get_ref())
	 .filter_map(
		 |item| futures::future::ready(
				match item {
					Ok(item) => Some(format!("{:?}", item)),
					Err(_) => None
				}
		 )
	 ).collect::<Vec<String>>().await;
	web::Json(products)
}

//TODO: add order
//TODO: delete order