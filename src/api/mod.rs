pub mod menu;
pub mod order;

mod prelude {
	pub(super) use crate::model;
	pub(super) use crate::error::Error;
	pub(super) use actix_web::{web, Responder};
	pub(super) use sqlx::MySqlPool;
	pub(super) use futures::stream::StreamExt;
	pub(super) use crate::auth::AuthToken;

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
