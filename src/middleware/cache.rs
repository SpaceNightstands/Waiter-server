use super::{auth::AuthToken, prelude::*};
use actix_web::rt;
use futures::future::FutureExt;

//use SharedPointer
type Cache = dashmap::DashSet<String>;
pub type CachePointer = Pin<SharedPointer<Cache>>;

pub async fn make_impedency_cache() -> (Box<Cache>, rt::task::JoinHandle<()>) {
	//use Box instead of Arc and use SharedPointer as reference
	let cache = Box::new(Cache::new());
	//Wipe the idempotency cache everyday
	let cache_handle = {
		let cache = unsafe { SharedPointer::new(&*cache) };
		rt::spawn(async move {
			log::debug!("Scheduled Cache clearer");
			loop {
				if crate::until_midnight().await {
					cache.clear();
					cache.shrink_to_fit();
				}
			}
		})
	};
	(cache, cache_handle)
}

pub struct IdempotencyCache(pub CachePointer);

impl<S> dev::Transform<S, ServiceRequest> for IdempotencyCache
where
	S: Service<ServiceRequest, Response = ServiceResponse<dev::Body>, Error = AxError>,
	S::Future: 'static,
{
	type Response = S::Response;

	type Error = S::Error;

	type InitError = ();

	type Transform = IdempotencyCacheService<S>;

	type Future = future::Ready<Result<Self::Transform, Self::InitError>>;

	fn new_transform(&self, service: S) -> Self::Future {
		future::ok(IdempotencyCacheService {
			service,
			cache: self.0.clone(),
		})
	}
}

pub struct IdempotencyCacheService<S: Service<ServiceRequest>> {
	service: S,
	cache: CachePointer,
}

impl<S> Service<ServiceRequest> for IdempotencyCacheService<S>
where
	S: Service<ServiceRequest, Response = ServiceResponse<dev::Body>, Error = AxError>,
	S::Future: 'static,
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
		if req.method().is_safe() || !req.method().is_idempotent() {
			return Box::pin(self.service.call(req));
		}

		let ext = req.head().extensions();
		//Get the idempotency token from the JWT
		let idempotency = ext.get::<AuthToken>().map(|token| token.idempotency());
		match idempotency {
			Some(idempotency) => {
				/*If the idempotency token has been used already,
				 *return an error*/
				log::debug!("Idemp Token: {:?}", idempotency);
				let cache_contains_idempotency = self.cache.contains(idempotency);
				log::debug!("Is token in cache?: {:?}", cache_contains_idempotency);
				if cache_contains_idempotency {
					/*ext and idempotency contain references to req, so
					 *we have to drop the references before using into_parts*/
					drop(idempotency);
					drop(ext);
					Box::pin(future::err(
						Error::Static {
							status: StatusCode::OK,
							reason: "IdempotencyCache",
							message: "Request already responded",
						}
						.into(),
					))
				} else {
					/*Otherwise, create a future that will add the token
					 *to the cache after having responded to the request*/
					let cache = self.cache.clone();
					let idempotency = idempotency.clone();
					std::mem::drop(ext);
					//TODO: Read Body and cache it
					Box::pin(self.service.call(req).map(move |result| {
						cache.insert(idempotency);
						result
					}))
				}
			}
			None => Box::pin(future::err(idemp_error("Invalid JWT").into())),
		}
	}
}

const fn idemp_error(message: &'static str) -> Error {
	Error::Static {
		status: StatusCode::UNAUTHORIZED,
		reason: "IdempotencyCache",
		message,
	}
}
