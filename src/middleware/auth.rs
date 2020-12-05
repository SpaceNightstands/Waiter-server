use super::prelude::*;
use actix_web::{
	dev::{
		self,
		Service,
		ServiceRequest,
		ServiceResponse,
	},
	http::{
		self,
		StatusCode
	},
	Error as axError
};
use sqlx::types::chrono;
use futures::future::{self, err};
use std::sync::Arc;

pub type Key = hmac::Hmac<sha2::Sha256>;
type DateTime = chrono::DateTime<chrono::FixedOffset>;

#[derive(serde::Serialize, serde::Deserialize, derive_getters::Getters, Clone)]
pub struct AuthToken {
	pub(crate) sub: String, //subject
	#[serde(with = "datetime")]
	pub(crate) exp: DateTime, //Expiration Time
	pub(crate) idempotency: String //idempotency token
}

mod datetime {
	use super::DateTime;

	pub(super) fn serialize<S: serde::Serializer>(tstamp: &DateTime, ser: S) -> Result<S::Ok, S::Error> {
		ser.serialize_str(&*tstamp.to_rfc3339())
	}

	pub(super) fn deserialize<'de, D: serde::Deserializer<'de>>(deser: D) -> Result<DateTime, D::Error> {
		let timestamp = <&str as serde::Deserialize>::deserialize(deser)?;
		DateTime::parse_from_rfc3339(timestamp)
			.map_err(serde::de::Error::custom)
	}
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

	type Transform = JWTAuthService<S>;

	type Future = future::Ready<Result<Self::Transform, Self::InitError>>;

	fn new_transform(&self, service: S) -> Self::Future {
		future::ok(
			JWTAuthService{
				service,
				key: self.0.clone(),
			}
		)
	}
}

pub struct JWTAuthService<S: Service>{
	pub(super) service: S,
	pub(super) key: Arc<Key>,
}

impl<S, B> Service for JWTAuthService<S>
where
	S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = axError>,
	S::Future: 'static,
	B: 'static
{
	type Request = S::Request;

	type Response = S::Response;

	type Error = S::Error;

	//TODO: Make a more specific type
	type Future = future::LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

	fn poll_ready(&mut self, ctx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
		self.service.poll_ready(ctx)
	}

	fn call(&mut self, mut req: Self::Request) -> Self::Future {
		//Authorization: Bearer <token>
		//Check that the header exists
		let header = if let Some(header) = req.headers().get(http::header::AUTHORIZATION) {
			match header.to_str() {
				Ok(header) => header,
				Err(error) => return Box::pin(
					err(
						Error::from(error).into()
					)
				)
			}
		} else {
			return Box::pin(
				err(
					auth_error(
						"Couldn't find Authorization header"
					).into()
				)
			)
		};

		//Check that the value starts with "Bearer ". If so, verify the jwt that comes after
		let claims: Result<AuthToken, jwt::Error> = if let Some(token) = header.trim().strip_prefix("Bearer "){
			use jwt::VerifyWithKey;
			token.verify_with_key(self.key.as_ref())
		} else {
			return Box::pin(
				err(
					auth_error(
						"Authorization header doesn't start with \"Bearer \""
					).into()
				)
			)
		};

		//Check that the jwt hasn't expired
		match claims {
			Ok(claims) => {
				// Add checksum check
				if chrono::Utc::now() < claims.exp {
					//If everything's ok, add the AuthToken to the request extensions
					req.head_mut().extensions_mut()
						.insert(claims);
					Box::pin(self.service.call(req))
				} else {
					Box::pin(
						err(
							auth_error("JWT Token expired").into()
						)
					)
				}
			},
			Err(error) => Box::pin(err(Error::from(error).into()))
		}
	}
}

#[inline]
const fn auth_error(message: &'static str) -> Error {
	Error::Static {
		status: StatusCode::UNAUTHORIZED,
		reason: "Authorization",
		message
	}
}
