mod menu;
mod order;

mod auth;
mod cache;

mod prelude {
	pub(super) use crate::model;
	pub(super) use crate::error::*;
	pub(super) use actix_web::{web, Responder};
	pub(super) use sqlx::MySqlPool;
	pub(super) use futures::stream::StreamExt;
	pub(super) use super::auth::AuthToken;

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

use std::sync::Arc;
pub use auth::Key;

use actix_web::{
	dev::{
		ServiceRequest,
		ServiceResponse
	},
	Error as axError
};
//TODO: Add host guard
pub fn get_service(scope: &str, key: Arc<Key>) -> actix_web::Scope<impl actix_service::ServiceFactory<Config = (), Request=ServiceRequest, Response=ServiceResponse, Error=axError, InitError= ()>> {
	actix_web::web::scope(scope)
    .wrap(
			auth::JWTAuth(key)
		).wrap(
			cache::IdempotencyCache(
				Arc::new(
					dashmap::DashSet::<String>::new()
				)
			)
		).service(
			menu::get_service()
		).service(
			order::get_service()
		)
}

