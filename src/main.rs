#[actix_web::main]
async fn main() -> std::io::Result<()> {
	//Parse .env (should make this optional)
	dotenv::dotenv().expect(".env not found or not parsable");
	//Enable Logging
	simple_logger::SimpleLogger::new().init().expect("Couldn't start logger");

	let conn = sqlx::MySqlPool::connect(
		&*std::env::var("DATABASE_URL").expect("environment variable DATABASE_URL not defined")
	).await
	 .expect("Couldn't connect to database");

	use actix_web::{HttpServer, App};
	HttpServer::new(move ||
		App::new()
				.data(conn.clone())
				.wrap(actix_web::middleware::Logger::default())
	).bind("0.0.0.0:8080")?
	 .run()
	 .await
}
