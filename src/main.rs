mod model;
mod api;
mod error;

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
		&*std::env::var("DATABASE_URL")
				.expect("Environment variable DATABASE_URL not set")
	).await
	 .expect("Couldn't connect to database");

	use actix_web::{HttpServer, App};
	HttpServer::new(move ||
		App::new()
				.data(conn.clone())
				.wrap(actix_web::middleware::Logger::default())
				.service(api::get_service("/"))
	).bind("0.0.0.0:8080")?
	 .run()
	 .await
}

use sqlx::MySqlPool;
async fn get_database(db_url: &str)->Result<MySqlPool, sqlx::Error> {
	let conn = MySqlPool::connect(db_url).await?;
	/*Check last order list addition,
		truncate if older than a day*/
	Ok(conn)
}
