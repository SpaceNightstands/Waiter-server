pub mod menu;
pub mod order;

mod prelude {
	pub(super) use crate::model;
	pub(super) use crate::error::Error;
	pub(super) use actix_web::{web, Responder};
	pub(super) use sqlx::MySqlPool;
	pub(super) use futures::stream::StreamExt;
	pub(super) use crate::auth::AuthToken;
	pub(super) use crate::middleware::filter;
}
