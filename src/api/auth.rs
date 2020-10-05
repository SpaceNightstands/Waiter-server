use super::prelude::{
	AuthorizationError as AuthError,
	StaticError
};
use actix_web::{
	dev::{
		self,
		Service,
		ServiceRequest,
		ServiceResponse
	},
	http::StatusCode,
	Error as axError
};
use sqlx::types::chrono::{
	self,
	Utc
};
use futures::future;
use std::sync::Arc;
pub type Key = hmac::Hmac<sha2::Sha256>;
type DateTime = chrono::DateTime<Utc>;

#[derive(serde::Deserialize, derive_getters::Getters)]
pub(super) struct AuthToken {
	sub: String, //subject
	#[serde(deserialize_with = "deser_datetime")]
	exp: DateTime, //Expiration Time
	idempotence: String
}

fn deser_datetime<'de, D: serde::Deserializer<'de>>(deser: D) -> Result<DateTime, D::Error> {
	let timestamp = <&str as serde::Deserialize>::deserialize(deser)?;
	Ok(
		chrono::DateTime::parse_from_rfc3339(
			timestamp
		).map_err(
			|err| serde::de::Error::custom(err)
		)?.with_timezone(&Utc)
	)
}

pub(super) struct JWTAuth(pub(super) Arc<Key>);

impl<S, B> dev::Transform<S> for JWTAuth
where
	S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = axError>,
	S::Future: 'static,
	B: 'static
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
				key: self.0.clone()
			}
		)
	}
}

pub struct JWTAuthService<S: Service>{
	service: S,
	key: Arc<Key>
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
		use futures::future::err;
		let header = if let Some(header) = req.headers().get("Authorization") {
			match header.to_str() {
				Ok(header) => header,
				Err(error) => return Box::pin(
					err(
						AuthError::new(error).into()
					)
				)
			}
		} else {
			return Box::pin(
				err(
					StaticError::new(StatusCode::UNAUTHORIZED, "Couldn't find Authorization header").into()
				)
			)
		};

		let claims: Result<AuthToken, jwt::Error> = if let Some(token) = header.trim().strip_prefix("Bearer "){
			use jwt::VerifyWithKey;

			token.verify_with_key(self.key.as_ref())
		} else {
			return Box::pin(
				err(
					StaticError::new(StatusCode::UNAUTHORIZED, "Authorization header doesn't start with \"Bearer \"").into()
				)
			)
		};

		match claims {
			Ok(claims) => {
				if Utc::now() < claims.exp {
					req.head_mut().extensions_mut()
						.insert(claims);
					Box::pin(self.service.call(req))
				} else {
					Box::pin(
						err(
							StaticError::new(StatusCode::UNAUTHORIZED, "JWT Token expired").into()
						)
					)
				}
			},
			Err(error) => Box::pin(err(AuthError::new(error).into()))
		}
	}
}
