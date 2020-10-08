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
use futures::{
	future::FutureExt,
	channel::oneshot
};

pub type Cache = std::sync::Arc<dashmap::DashSet<String>>;

pub async fn make_impedency_cache() -> (Cache, oneshot::Sender<()>, impl std::future::Future) {
	let cache = Cache::new(dashmap::DashSet::new());
	let (routine_stopper, recv) = oneshot::channel::<()>();
	let cleaning_routine = {
		let cache = cache.clone();
		async move {
			log::debug!("Scheduled Cache clearer");
			let mut recv = recv.fuse();
			loop {
				futures::select_biased! {
					_ = recv => break,
					is_past_midnight = crate::wait_until_midnight() => if is_past_midnight {
						cache.clear();
					}
				}
			}
		}
	};
	(cache, routine_stopper, cleaning_routine)
}

pub struct IdempotencyCache(pub Cache);

impl<S, B> dev::Transform<S> for IdempotencyCache
where
	S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = axError>,
	S::Future: 'static,
	B: 'static
{
	type Request = S::Request;

	type Response = S::Response;

	type Error = S::Error;

	type InitError = ();

	type Transform = IdempotencyCacheService<S>;

	type Future = future::Ready<Result<Self::Transform, Self::InitError>>;

	fn new_transform(&self, service: S) -> Self::Future {
		future::ok(
			IdempotencyCacheService{
				service,
				cache: self.0.clone()
			}
		)
	}
}

pub struct IdempotencyCacheService<S: Service>{
	service: S,
	cache: Cache
}

impl<S, B> Service for IdempotencyCacheService<S>
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
		let idempotency = req.head().extensions().get::<AuthToken>()
			.map(
				|token| token.idempotency().clone()
			);
		match idempotency {
			Some(idempotency) => {
				if self.cache.contains(&idempotency) {
					//TODO: Return some "already replied" response
					Box::pin(
						future::err(
							crate::error::StaticError::new(
								StatusCode::BAD_REQUEST,
								"Already replied to a request with that idemp token"
							).into()
						)
					)
				} else {
					let cache = self.cache.clone();
					Box::pin(
						self.service.call(req)
							.map(
								move |result| {
									cache.insert(
										idempotency
									);
									result
								}
							)
					)
				}
			},
			None => Box::pin(
				future::err(
					StaticError("The JWT wasn't correctly verified").into()
				)
			)
		}
	}
}

#[inline]
fn StaticError(err: &'static str) -> crate::error::StaticError {
	crate::error::StaticError::new(StatusCode::INTERNAL_SERVER_ERROR, err)
}
