mod menu;
mod order;
mod auth;

mod prelude {
	pub(crate) use crate::model;
	pub(crate) use crate::error::Error;
	pub(crate) use actix_web::{web, Responder};
	pub(crate) use sqlx::MySqlPool;
	pub(crate) use futures::stream::StreamExt;
//	pub(crate) use super::auth::AuthToken;

	//Utils
	pub(crate) fn result_ok_log<T, E: std::fmt::Display>(res: Result<T, E>) -> Option<T> {
		match res {
			Ok(item) => Some(item),
			Err(error) => {
				use log::{
					log_enabled, error, Level::Error
				};
				if log_enabled!(Error) {
					error!("{}", error)
				};
				None
			}
		}
	}
}

//TODO: Add auth guard
pub fn get_service(scope: &str) -> actix_web::Scope{
	actix_web::web::scope(scope)
    .service(menu::get_service().guard(auth::jwt_guard))
    .service(order::get_service().guard(auth::jwt_guard))
}

