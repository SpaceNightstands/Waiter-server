use super::prelude::*;

#[actix_web::get("/menu")]
async fn get_menu(db: web::Data<MySqlPool>) -> impl Responder {
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
