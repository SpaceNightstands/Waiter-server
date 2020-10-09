mod service;

use actix_web::{
	dev::{
		self,
		Service,
		ServiceRequest,
		ServiceResponse
	},
	Error as axError
};
use sqlx::types::chrono;
use futures::future;
use std::sync::Arc;

pub type Key = hmac::Hmac<sha2::Sha256>;
type DateTime = chrono::DateTime<chrono::FixedOffset>;

#[derive(serde::Deserialize, derive_getters::Getters)]
pub(crate) struct AuthToken {
	sub: String, //subject
	#[serde(deserialize_with = "deser_datetime")]
	exp: DateTime, //Expiration Time
	idempotency: String //idempotency token
}

fn deser_datetime<'de, D: serde::Deserializer<'de>>(deser: D) -> Result<DateTime, D::Error> {
	let timestamp = <&str as serde::Deserialize>::deserialize(deser)?;
	DateTime::parse_from_rfc3339(
		timestamp
	).map_err(
		|err| serde::de::Error::custom(err)
	)
}

pub struct JWTAuth(pub Arc<Key>);

impl<S, B> dev::Transform<S> for JWTAuth
where
	S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = axError>,
	S::Future: 'static,
	B: 'static,
{
	type Request = S::Request;

	type Response = S::Response;

	type Error = S::Error;

	type InitError = ();

	type Transform = service::JWTAuthService<S>;

	type Future = future::Ready<Result<Self::Transform, Self::InitError>>;

	fn new_transform(&self, service: S) -> Self::Future {
		future::ok(
			service::JWTAuthService{
				service,
				key: self.0.clone(),
				authorizer: None
			}
		)
	}
}

pub struct SelectiveJWTAuth(pub Arc<Key>, pub Arc<[String]>);

impl<S, B> dev::Transform<S> for SelectiveJWTAuth
where
	S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = axError>,
	S::Future: 'static,
	B: 'static
{
	type Request = S::Request;

	type Response = S::Response;

	type Error = S::Error;

	type InitError = ();

	type Transform = service::JWTAuthService<S>;

	type Future = future::Ready<Result<Self::Transform, Self::InitError>>;

	fn new_transform(&self, service: S) -> Self::Future {
		future::ok(
			service::JWTAuthService{
				service,
				key: self.0.clone(),
				authorizer: Some(self.1.clone())
			}
		)
	}
}
