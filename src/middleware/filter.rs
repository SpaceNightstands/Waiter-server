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
use super::auth::AuthToken;

type SubList<'s> = std::sync::Arc<[&'s str]>;

pub struct SubjectFilter<'s>(SubList<'s>);

impl<'s, S, B> dev::Transform<S> for SubjectFilter<'s>
where
	S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = axError>,
	S::Future: 'static,
	B: 'static
{
	type Request = S::Request;

	type Response = S::Response;

	type Error = S::Error;

	type InitError = ();

	type Transform = SubjectFilterService<'s, S>;

	type Future = future::Ready<Result<Self::Transform, Self::InitError>>;

	fn new_transform(&self, service: S) -> Self::Future {
		future::ok(
			SubjectFilterService{
				service,
				authorized: self.0.clone()
			}
		)
	}
}

pub struct SubjectFilterService<'s, S: Service>{
	service: S,
	authorized: SubList<'s>
}

impl<'s, S, B> Service for SubjectFilterService<'s, S>
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

	fn call(&mut self, req: Self::Request) -> Self::Future {
		let ext = req.head().extensions();
		let subject = ext.get::<AuthToken>()
			.map(
				|token| &**token.sub()
			);
		match subject {
			Some(ref sub) => {
				if self.authorized.contains(sub) {
					std::mem::drop(ext);
					return Box::pin(self.service.call(req))
				} else {
					Box::pin(
						future::err(
							StaticError("Account is not authorized").into()
						)
					)
				}
			},
			None => Box::pin(
				future::err(
					StaticError("Invalid JWT").into()
				)
			)
		}
	}
}

#[inline]
fn StaticError(err: &'static str) -> crate::error::StaticError {
	crate::error::StaticError::new(StatusCode::UNAUTHORIZED, err)
}
