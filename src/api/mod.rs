mod prelude {
	pub(crate) use crate::model;
	pub(crate) use actix_web::{web, Responder};
	pub(crate) use sqlx::MySqlPool;
	pub(crate) use futures::stream::StreamExt;
}

mod menu;

//Temporary
pub use menu::get_menu_service;

pub fn get_service(scope: &str) -> actix_web::Scope{
	actix_web::web::scope(scope)
    .service(menu::get_menu_service())
    .route("", actix_web::web::get().to(hello_world))
}

async fn hello_world() -> impl actix_web::Responder {
	"Hello, World"
}
