use crate::{
	model::*,
	middleware::auth
};
use actix_web::{
	test,
	dev::Service
};
use sqlx::types::chrono;
use hmac::NewMac;
use jwt::SignWithKey;
/*
async fn put_orders(db: web::Data<MySqlPool>, mut cart: web::Json<Vec<(u32, u32)>>, req: web::HttpRequest) -> Result<impl Responder, Error>
*/

pub(super) async fn orders_test(database: &sqlx::MySqlPool) {
	let key = std::sync::Arc::new(
		auth::Key::new_varkey(
			dotenv_codegen::dotenv!("JWT_SECRET").as_bytes()
		).unwrap()
	);

	let mut service = test::init_service(
		actix_web::App::new()
			.data(database.clone())
			.wrap(auth::JWTAuth(key.clone()))
			.service(crate::api::order::get_service())
			.service(crate::api::menu::get_service())
	).await;

	//JSON Auth Token
	let auth: String = {
		let auth = auth::AuthToken {
			sub: "test".to_string(),
			exp: chrono::Utc::today()
				.succ()
				.succ()
				.and_hms(0, 0, 0)
				.with_timezone(&chrono::FixedOffset::east(0)),
			idempotency: "test".to_string(),
		};
		auth.sign_with_key(&*key).unwrap()
	};

	let prod = {
		let req = test::TestRequest::put()
			.uri("/menu")
			.header(
				actix_web::http::header::AUTHORIZATION,
				format!("Bearer {}", auth)
			).set_json(
				&Product {
					id: 0,
					kind: ProductKind::Available,
					name: String::from("Test"),
					price: 100,
					max_num: 3,
					ingredients: None,
					image: vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]
				}
			).to_request();
		let resp: Product = test::read_body_json(
			service.call(req).await.unwrap()
		).await;
		*resp.id()
	};

	//Put
	{
		let req = test::TestRequest::put()
			.uri("/order")
			.header(
				actix_web::http::header::AUTHORIZATION,
				format!("Bearer {}", auth)
			).set_json(
				&vec![(prod, 1), (prod, 2)]
			).to_request();
		let resp = test::read_body(
			service.call(req).await.unwrap()
		).await;
		println!("{:?}", resp);
		//assert_eq!(resp[0], prod, "Sample: {:?}\n\nResponse: {:?}", prod, resp);
	}
	//Get
	{
		let req = test::TestRequest::get()
			.uri("/order")
			.header(
				actix_web::http::header::AUTHORIZATION,
				format!("Bearer {}", auth)
			).to_request();
		let resp = test::read_body(
			service.call(req).await.unwrap()
		).await;
		println!("{:?}", resp);
		//assert_eq!(resp[0], prod, "Sample: {:?}\n\nResponse: {:?}", prod, resp);
	}
}
