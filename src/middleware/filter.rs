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

//TODO: Move to a HashSet, or generalize over some "Searchable" trait
pub type SubList = std::sync::Arc<[String]>;

pub struct SubjectFilter(pub SubList);

impl<S, B> dev::Transform<S> for SubjectFilter
where
	S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = axError>,
	S::Future: 'static,
	B: 'static
{
	type Request = S::Request;

	type Response = S::Response;

	type Error = S::Error;

	type InitError = ();

	type Transform = SubjectFilterService<S>;

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

pub struct SubjectFilterService<S: Service>{
	service: S,
	authorized: SubList
}

impl<S, B> Service for SubjectFilterService<S>
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
				|token| token.sub()
			);
		match subject {
			Some(sub) => {
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
