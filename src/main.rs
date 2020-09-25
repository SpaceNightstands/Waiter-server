mod model;
use actix_web::{
	Responder,
	web
};
use sqlx::MySqlPool;
use futures::stream::StreamExt;

//TODO: move services in an api module
#[actix_web::get("/auth")]
async fn get_jwt() -> impl Responder {
	"Hello, world"
}

#[actix_web::get("/menu")]
async fn get_menu(db: web::Data<MySqlPool>) -> impl Responder {
	let products = sqlx::query_as!(
		model::Product,
		"SELECT * FROM products"
	).fetch(db.get_ref())
	 .filter_map(
		 |item| futures::future::ready(
				match item {
					Ok(item) => Some(format!("{:?}", item)),
					Err(_) => None
				}
		 )
	 ).collect::<Vec<String>>().await;
	web::Json(products)
}

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
				.service(get_jwt)
				.service(get_menu)
	).bind("0.0.0.0:8080")?
	 .run()
	 .await
}

async fn get_database(db_url: &str)->Result<MySqlPool, sqlx::Error> {
	let conn = MySqlPool::connect(db_url).await?;
	/*Check last order list addition,
		truncate if older than a day*/
	Ok(conn)
}
