pub mod menu;
pub mod order;

mod prelude {
	pub(super) use crate::auth::AuthToken;
	pub(super) use crate::error::Error;
	pub(super) use crate::middleware::filter;
	pub(super) use crate::model;
	pub(super) use crate::Pool;
	pub(super) use actix_web::{web, Responder};
	pub(super) use futures::stream::StreamExt;
}
