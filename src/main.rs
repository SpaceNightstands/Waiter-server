mod model;
mod api;
mod middleware;
mod error;
mod database;
mod pointer;
mod signals;

#[cfg(test)]
mod test;

use std::env::var as env_var;
use api::*;
use middleware::*;
use futures::future::FutureExt;
use pointer::SharedPointer;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
	//Parse .env if possible
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

	//Create Database Connection and get cleaning routine future
	log::debug!("Connecting to Database...");
	let (conn, database_stopper) = database::get_database(
		&*env_var("DATABASE_URL")
			.expect("Environment variable DATABASE_URL not set"),
	).await
		.expect("Couldn't connect to database");

	//Create Cache and get clearing routine future
	let (cache, cache_stopper) = cache::make_impedency_cache().await;

	//For host guard
	/*let host = env_var("HOST")
    .expect("Environment variable HOST not set");*/

	//Create JWT Key
	use hmac::NewMac;
	let key = auth::Key::new_varkey(
		env_var("JWT_SECRET")
			.expect("Environment variable DATABASE_URL not set")
			.as_bytes()
	).unwrap();

	//Admins are the only google accounts able to edit the menu
	let admins = env_var("ADMINS")
    .map(
			|string| string.split(',')
				.map(String::from)
				.collect::<std::collections::HashSet<String>>()
		).ok();

	use actix_web::{
		HttpServer,
		App,
		middleware
	};
	let server = {
		let (key_ref, admins_ref) = unsafe {
			(
				SharedPointer::new(&key),
				admins.as_ref().map(
					|admins| SharedPointer::new(admins)
				)
			)
		};
		HttpServer::new(move || {
			let cache = cache.clone();
			//Middleware is executed in reverse registration order
			App::new()
				.app_data(
					actix_web::web::JsonConfig::default()
						.limit(2621440) //2.5 MiB
						.error_handler(
							|err, _| error::Error::passthrough(
								actix_web::http::StatusCode::BAD_REQUEST,
								"JsonExtractor",
								&err
							).into()
						)
				).data(conn.clone())
				.wrap(cache::IdempotencyCache(cache))
				.wrap(auth::JWTAuth(key_ref))
				.wrap(actix_cors::Cors::permissive())
				.wrap(middleware::Logger::default())
				.service(menu::get_service(admins_ref))
				.service(order::get_service(admins_ref))
		})
	}.bind(
		format!(
			"{}:{}",
			env_var("SERVER_ADDRESS").unwrap_or("0.0.0.0".to_string()),
			env_var("SERVER_PORT").unwrap_or("8080".to_string()),
		)
	)?.disable_signals()
		.run();

	signals::handle_kill_signals(server.clone(), database_stopper, cache_stopper);

	let return_value = server.await;

	println!("Cleanup");
	drop(key);
	drop(admins);

	return_value
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

