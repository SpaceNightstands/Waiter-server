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
use futures::future;
use std::sync::Arc;
pub type Key = hmac::Hmac<sha2::Sha256>;


#[derive(derive_getters::Getters)]
pub(super) struct AuthToken {
	account_id: String,
	idempotence: String
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

	//TODO: Change with a more specific type
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

		let claims = if let Some(token) = header.trim().strip_prefix("Bearer "){
			use jwt::{VerifyWithKey, Error};
			use std::collections::HashMap;

			let claims: Result<HashMap<String, String>, Error> = token.verify_with_key(self.key.as_ref());
			claims
		} else {
			return Box::pin(
				err(
					StaticError::new(StatusCode::UNAUTHORIZED, "Authorization header doesn't start with \"Bearer \"").into()
				)
			)
		};

		match claims {
			Ok(mut claims) => {
				if let (Some(acc_id), Some(idempotence)) = (claims.remove("sub"), claims.remove("idemp")) {
					req.head_mut().extensions_mut().insert(
						AuthToken {
							account_id: acc_id,
							idempotence
						}
					);
					Box::pin(self.service.call(req))
				} else {
					Box::pin(
						err(
							StaticError::new(StatusCode::UNAUTHORIZED, "Couldn't find sub(ject) in JWT").into()
						)
					)
				}
			},
			Err(error) => Box::pin(err(AuthError::new(error).into()))
		}
	}
}
