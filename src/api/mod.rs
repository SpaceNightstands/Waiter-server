mod prelude {
	pub(crate) use crate::model;
	pub(crate) use actix_web::{web, Responder};
	pub(crate) use sqlx::MySqlPool;
	pub(crate) use futures::stream::StreamExt;
}

mod menu;

pub fn get_menu_service() -> actix_web::Scope{
	actix_web::web::scope("/menu")
    .service(menu::get_menu)
}
