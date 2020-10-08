//TODO: Refactor modules
#![allow(non_snake_case)]
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
	//Parse .env (should make this optional)
	dotenv::dotenv().expect(".env not found or not parsable");
	//Enable Logging
	simple_logger::SimpleLogger::new()
    .with_level(log::LevelFilter::Debug)
		.init()
		.expect("Couldn't set logger");

	//Create JWT Key
	use hmac::NewMac;
	let key = std::sync::Arc::new(
		auth::Key::new_varkey(
				env_var("JWT_SECRET")
					.expect("Environment variable DATABASE_URL not set")
					.as_bytes()
			).unwrap()
	);

	//Create Cache and get clearing routine future
	let (cache, cache_stopper, cache_clearing_routine) = cache::make_impedency_cache().await;

	//Create Database Connection and get cleaning routine future
	let (conn, database_stopper, database_cleanse_routine) = database::get_database(
		&*env_var("DATABASE_URL")
			.expect("Environment variable DATABASE_URL not set"),
	).await
	 .expect("Couldn't connect to database");

	use actix_web::{HttpServer, App};
	let server = HttpServer::new(move || {
		let key = key.clone();
		let cache = cache.clone();
		//TODO: Add host guard
		//Middleware is executed in reverse registration order
		App::new()
				.data(conn.clone())
				.wrap(cache::IdempotencyCache(cache))
				.wrap(auth::JWTAuth(key))
				.wrap(actix_web::middleware::Logger::default())
				.service(menu::get_service())
				.service(order::get_service())
	}).bind(
		format!(
			"{}:{}",
			env_var("SERVER_ADDRESS").unwrap_or("0.0.0.0".to_string()),
			env_var("SERVER_PORT").unwrap_or("8080".to_string()),
		)
	)?.disable_signals()
		.run();

	let sigHandler = {
		let server = server.clone();
		#[cfg(not(unix))]
		{
			actix_rt::signal::ctrl_c()
				.then(
					|_| async move {
						log::debug!("Received Ctrl-C");
						stopper(server, database_stopper, cache_stopper).await
					}
				)
		}
		#[cfg(unix)]
		{
			actix_rt::signal::ctrl_c()
				.then(
					|_| async move {
						log::debug!("Received Ctrl-C");
						stopper(server, database_stopper, cache_stopper).await
					}
				)
		}
	};
	futures::join![sigHandler, server, database_cleanse_routine, cache_clearing_routine].1
}

fn wait_until_midnight() -> futures::future::Fuse<impl std::future::Future<Output = bool>> {
	use sqlx::types::chrono::Local;
	async {
		//Use and_hms_opt instead, handle errors
		let before_waiting = Local::today();
		let time_until_midnight = before_waiting.succ()
			.and_hms(0, 0, 0)
			.signed_duration_since(Local::now())
			.to_std()
			.unwrap();
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
