mod prelude {
	pub(crate) use crate::model;
	pub(crate) use actix_web::{web, Responder};
	pub(crate) use sqlx::MySqlPool;
	pub(crate) use futures::stream::StreamExt;
}

mod menu;

use actix_web::web;
pub fn get_menu_service() -> actix_web::Scope{
	use menu::*;
	web::scope("/menu")
    .route("", web::get().to(get_menu))
}
