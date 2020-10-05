#![allow(non_snake_case)]
mod model;
mod api;
mod error;

use std::env::var as env_var;
#[actix_web::main]
async fn main() -> std::io::Result<()> {
	//Parse .env (should make this optional)
	dotenv::dotenv().expect(".env not found or not parsable");
	//Enable Logging
	simple_logger::SimpleLogger::new()
    .with_level(log::LevelFilter::Debug)
		.init()
		.expect("Couldn't set logger");

	let conn = get_database(
		&*env_var("DATABASE_URL")
				.expect("Environment variable DATABASE_URL not set")
	).await
	 .expect("Couldn't connect to database");

	use actix_web::{HttpServer, App};
	let folder = env_var("SERVER_DIRECTORY")
    .unwrap_or("/".to_string());

	use hmac::NewMac;
	let key = std::sync::Arc::new(
		api::Key::new_varkey(
				env_var("JWT_SECRET")
					.expect("Environment variable DATABASE_URL not set")
					.as_bytes()
			).unwrap()
	);
	HttpServer::new(move || {
			let key = key.clone();
			App::new()
					.data(conn.clone())
					.wrap(actix_web::middleware::Logger::default())
					.service(api::get_service(&*folder, key))
		}
	).bind(
		format!(
			"{}:{}",
			env_var("SERVER_ADDRESS").unwrap_or("0.0.0.0".to_string()),
			env_var("SERVER_PORT").unwrap_or("8080".to_string()),
		)
	)?.run()
	 .await
}

use sqlx::MySqlPool;
async fn get_database(db_url: &str)->Result<MySqlPool, sqlx::Error> {
	let conn = MySqlPool::connect(db_url).await?;
	/*Check last order list addition,
		truncate if older than a day*/
	Ok(conn)
}
