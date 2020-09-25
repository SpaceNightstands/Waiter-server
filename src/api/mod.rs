mod prelude {
	pub(crate) use crate::model;
	pub(crate) use actix_web::{web, Responder};
	pub(crate) use sqlx::MySqlPool;
	pub(crate) use futures::stream::StreamExt;
}

mod menu;
mod order;

//TODO: Add auth guard
pub fn get_service(scope: &str) -> actix_web::Scope{
	actix_web::web::scope(scope)
    .service(menu::get_service())
    .service(order::get_service())
}

