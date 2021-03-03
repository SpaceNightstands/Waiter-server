use super::{
	auth::AuthToken,
	prelude::*
};
use futures::{
	future::FutureExt,
	channel::oneshot
};

//use SharedPointer
type Cache = dashmap::DashSet<String>;
pub type CachePointer = Pin<SharedPointer<Cache>>;

pub async fn make_impedency_cache() -> (Box<Cache>, oneshot::Sender<()>) {
	//use Box instead of Arc and use SharedPointer as reference
	let cache = Box::new(Cache::new());
	//Wipe the idempotency cache everyday
	//TODO: truncate the capacity to a sensible amount
	let (routine_stopper, recv) = oneshot::channel::<()>();
	{
		let cache = unsafe {
			SharedPointer::new(&*cache)
		};
		actix_rt::spawn(
			async move {
				log::debug!("Scheduled Cache clearer");
				let mut recv = recv.fuse();
				loop {
					futures::select_biased! {
						_ = recv => break,
						is_past_midnight = crate::until_midnight() => if is_past_midnight {
							cache.clear();
						}
					}
				}
			}
		)
	};
	(cache, routine_stopper)
}

pub struct IdempotencyCache(pub CachePointer);

impl<S, B> dev::Transform<S> for IdempotencyCache
where
	S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = AxError>,
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
	cache: CachePointer
}

impl<S, B> Service for IdempotencyCacheService<S>
where
	S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = AxError>,
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
		if req.method().is_safe() || !req.method().is_idempotent() {
			return Box::pin(self.service.call(req))
		}

		let ext = req.head().extensions();
		//Get the idempotency token from the JWT
		let idempotency = ext.get::<AuthToken>()
			.map(
				|token| token.idempotency()
			);
		match idempotency {
			Some(idempotency) => {
				/*If the idempotency token has been used already, 
				 *return an error*/
				log::debug!("Idemp Token: {:?}", idempotency);
				let cache_contains_idempotency = self.cache.contains(idempotency);
				log::debug!("Is token in cache?: {:?}", cache_contains_idempotency);
				if cache_contains_idempotency {
					Box::pin(
						future::err(
							idemp_error("Invalid idemp token").into()
						)
					)
				} else {
					/*Otherwise, create a future that will add the token
					 *to the cache after having responded to the request*/
					let cache = self.cache.clone();
					let idempotency = idempotency.clone();
					std::mem::drop(ext);
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
					idemp_error("Invalid JWT").into()
				)
			)
		}
	}
}

const fn idemp_error(message: &'static str) -> Error {
	Error::Static {
		status: StatusCode::UNAUTHORIZED,
		reason: "IdempotencyCache",
		message
	}
}
