mod model;
mod api;
mod middleware;
mod error;
mod database;

#[cfg(test)]
mod test;

use std::env::var as env_var;
use api::*;
use middleware::*;
use futures::{
	future::FutureExt,
	channel::oneshot::Sender
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
	//Parse .env
	#[allow(unused_must_use)]
	{
		dotenv::dotenv();
	}
	//Enable Logging
	//TODO: Improve logging
	simple_logger::SimpleLogger::new()
    .with_level(log::LevelFilter::Debug)
		.init()
		.expect("Couldn't set logger");

	//For host guard
	/*let host = env_var("HOST")
    .expect("Environment variable HOST not set");*/

	//Create JWT Key
	use hmac::NewMac;
	//TODO: Test NonNull pointer
	let key = std::sync::Arc::new(
		auth::Key::new_varkey(
			env_var("JWT_SECRET")
				.expect("Environment variable DATABASE_URL not set")
				.as_bytes()
		).unwrap()
	);

	//Admins are the only google accounts able to edit the menu
	let admins = env_var("ADMINS")
    .map(
			|string| filter::SubList::new(
				string.split(',')
					.map(String::from)
					.collect::<std::collections::HashSet<String>>()
			)
		).ok();

	//Create Cache and get clearing routine future
	let (cache, cache_stopper) = cache::make_impedency_cache().await;

	//Create Database Connection and get cleaning routine future
	let (conn, database_stopper) = database::get_database(
		&*env_var("DATABASE_URL")
			.expect("Environment variable DATABASE_URL not set"),
	).await
		.expect("Couldn't connect to database");


	use actix_web::{
		HttpServer,
		App,
		middleware
	};
	let server = HttpServer::new(move || {
		let key = key.clone();
		let cache = cache.clone();
		let admins = admins.clone();
		//TODO: Add host guard
		//Middleware is executed in reverse registration order
		App::new()
			.data(conn.clone())
			.wrap(cache::IdempotencyCache(cache))
			.wrap(auth::JWTAuth(key))
			.wrap(middleware::Logger::default())
			.service(menu::get_service(admins))
			.service(order::get_service())
	}).bind(
		format!(
			"{}:{}",
			env_var("SERVER_ADDRESS").unwrap_or("0.0.0.0".to_string()),
			env_var("SERVER_PORT").unwrap_or("8080".to_string()),
		)
	)?.disable_signals()
		.run();

	{
		let server = server.clone();
		//On non-unix platforms, use the simple ctrl+c handler
		#[cfg(not(unix))]
		actix_rt::spawn(
			actix_rt::signal::ctrl_c()
				.then(
					|_| async move {
						log::debug!("Received Ctrl-C");
						stopper(server, database_stopper, cache_stopper).await
					}
				)
		);
		/*On *nix register a listener for every terminating
		 *signal*/
		#[cfg(unix)]
		{
			use actix_rt::signal::unix::{
				self,
				SignalKind
			};
			let mut signals = Vec::new();
			const SIGNAL_LIST: [SignalKind; 4] = [
				SignalKind::interrupt(),
				SignalKind::hangup(),
				SignalKind::terminate(),
				SignalKind::quit(),
			];
			for kind in SIGNAL_LIST.iter() {
				match unix::signal(*kind) {
					Ok(stream) => signals.push(stream),
					Err(e) => if log::log_enabled!(log::Level::Error) {
						log::error!(
							"Cannot initialize stream handler for {:?} err: {}",
							kind, e
						)
					}
				}
			}

			//Poll every stream and stop everything if any signal is received
			use std::task::Poll;
			actix_rt::spawn(
				futures::future::poll_fn(
					move |ctx|{
						for sig in signals.iter_mut() {
							if let Poll::Ready(Some(())) = sig.poll_recv(ctx) {
								return Poll::Ready(())
							}
						}
						Poll::Pending
					}
				).then(
					|_| async move {
						stopper(server, database_stopper, cache_stopper).await
					}
				)
			)
		}
	};
	server.await
}

fn until_midnight() -> futures::future::Fuse<impl std::future::Future<Output = bool>> {
	use sqlx::types::chrono::Local;
	async {
		//Get today
		let before_waiting = Local::today();
		//Get tomorrow and add 0:0:0 as hours, minutes and seconds respectively
		let time_until_midnight = before_waiting.succ()
			//TODO: Use and_hms_opt instead, handle errors
			.and_hms(0, 0, 0)
			//Get the duration between midnight and now
			.signed_duration_since(Local::now())
			.to_std()
			//TODO: Handle error better
			.unwrap();
		//Tell the runtime that this future has to wait until midnight
		actix_rt::time::delay_for(time_until_midnight).await;
		//Return if it's actually past midnight
		Local::today() > before_waiting
	}.fuse()
}

#[inline]
async fn stopper(server: actix_web::dev::Server, database: Sender<()>, cache: Sender<()>) {
	server.stop(true).await;
	database.send(()).unwrap();
	cache.send(()).unwrap();
}
