use super::{auth::AuthToken, prelude::*};

pub(crate) type SubList = Pin<SharedPointer<std::collections::HashSet<String>>>;

pub(crate) struct SubjectFilter(pub SubList);

impl<S, B> dev::Transform<S, ServiceRequest> for SubjectFilter
where
	S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = AxError>,
	S::Future: 'static,
	B: 'static,
{
	type Response = S::Response;

	type Error = S::Error;

	type InitError = ();

	type Transform = SubjectFilterService<S>;

	type Future = future::Ready<Result<Self::Transform, Self::InitError>>;

	fn new_transform(&self, service: S) -> Self::Future {
		future::ok(SubjectFilterService {
			service,
			authorized: self.0,
		})
	}
}

pub struct SubjectFilterService<S: Service<ServiceRequest>> {
	service: S,
	authorized: SubList,
}

impl<S, B> Service<ServiceRequest> for SubjectFilterService<S>
where
	S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = AxError>,
	S::Future: 'static,
	B: 'static,
{
	type Response = S::Response;

	type Error = S::Error;

	//TODO: Make a more specific type
	type Future = future::LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

	fn poll_ready(
		&self, ctx: &mut std::task::Context<'_>,
	) -> std::task::Poll<Result<(), Self::Error>> {
		self.service.poll_ready(ctx)
	}

	fn call(&self, req: ServiceRequest) -> Self::Future {
		let ext = req.head().extensions();
		//Get the AuthToken
		let subject = ext.get::<AuthToken>().map(|token| token.sub());
		match subject {
			Some(sub) => {
				//If the id is authorized, pass the call
				log::debug!("[SubFilter] Subject: {}", sub);
				if self.authorized.contains(sub) {
					std::mem::drop(ext);
					return Box::pin(self.service.call(req));
				} else {
					//Otherwise, return an error
					Box::pin(future::err(
						filter_error("Account is not authorized").into(),
					))
				}
			}
			None => Box::pin(future::err(filter_error("Invalid JWT").into())),
		}
	}
}

#[inline]
const fn filter_error(message: &'static str) -> Error {
	Error::Static {
		status: StatusCode::UNAUTHORIZED,
		reason: "SubFilter",
		message,
	}
}
