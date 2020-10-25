use crate::{
	model::*,
	middleware::*,
	api::*
};
use actix_web::{
	test,
	dev::Service
};
use sqlx::types::chrono;
use hmac::NewMac;
use jwt::SignWithKey;
use std::sync::Arc;

pub(super) async fn integration_test(database: &sqlx::MySqlPool) {
	let key = Arc::new(
		auth::Key::new_varkey(
			dotenv_codegen::dotenv!("JWT_SECRET").as_bytes()
		).unwrap()
	);
	let cache = Arc::new(dashmap::DashSet::new());
	let filter: Arc<[String]> = vec!["admin".to_string()].into();

	let mut service = test::init_service(
		actix_web::App::new()
			.data(database.clone())
			.wrap(cache::IdempotencyCache(cache))
			.wrap(auth::JWTAuth(key.clone()))
			.wrap(actix_web::middleware::Logger::default())
			.service(menu::get_service(filter))
			.service(order::get_service())
	).await;

	//Common JWT exp attribute
	let common_expiry = chrono::Utc::today()
		.succ()
		.succ()
		.and_hms(0, 0, 0)
		.with_timezone(&chrono::FixedOffset::east(0));

	//Testing idempotency token cache
	{
		let auth = auth::AuthToken {
			sub: "test".to_string(),
			exp: common_expiry.clone(),
			idempotency: "test0".to_string()
		}.sign_with_key(&*key).unwrap();
		let auth = format!("Bearer {}", auth);
		let req = test::TestRequest::get()
			.uri("/order")
			.header(
				actix_web::http::header::AUTHORIZATION,
				auth.clone()
			).to_request();
		let resp: Vec<Order> = test::read_body_json(
			service.call(req).await.unwrap()
		).await;
		assert_eq!(resp.len(), 1);

		let req = test::TestRequest::get()
			.uri("/order")
			.header(
				actix_web::http::header::AUTHORIZATION,
				auth
			).to_request();
		let resp = service.call(req).await
			.err()
			.unwrap();
		resp.as_error::<crate::error::Error>()
			.unwrap();
		//TODO: maybe add an assertion?
	}

	//Testing subject authentication
	{
		let auth = auth::AuthToken {
			sub: "admin".to_string(),
			exp: common_expiry.clone(),
			idempotency: "test1".to_string()
		}.sign_with_key(&*key).unwrap();
		let req = test::TestRequest::get()
			.uri("/order")
			.header(
				actix_web::http::header::AUTHORIZATION,
				format!("Bearer {}", auth)
			).to_request();
		let resp: Vec<Order> = test::read_body_json(
			service.call(req).await.unwrap()
		).await;
		assert_eq!(resp.len(), 0);

		let auth = auth::AuthToken {
			sub: "test".to_string(),
			exp: common_expiry.clone(),
			idempotency: "test2".to_string()
		}.sign_with_key(&*key).unwrap();
		let req = test::TestRequest::get()
			.uri("/order")
			.header(
				actix_web::http::header::AUTHORIZATION,
				format!("Bearer {}", auth)
			).to_request();
		let resp: Vec<Order> = test::read_body_json(
			service.call(req).await.unwrap()
		).await;
		assert_eq!(resp.len(), 1);
	}

	//Testing subject filter
	{
		let prod = Product {
			id: 3,
			kind: ProductKind::Available,
			name: String::from("Auth Test"),
			price: 100,
			max_num: 3,
			ingredients: None,
			image: vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]
		};

		let auth = auth::AuthToken {
			sub: "test".to_string(),
			exp: common_expiry.clone(),
			idempotency: "test3".to_string()
		}.sign_with_key(&*key).unwrap();
		let req = test::TestRequest::put()
			.uri("/menu")
			.header(
				actix_web::http::header::AUTHORIZATION,
				format!("Bearer {}", auth)
			).set_json(&prod)
			.to_request();
		let resp = service.call(req).await
			.err()
			.unwrap();
		println!("{:?}", resp);

		let auth = auth::AuthToken {
			sub: "admin".to_string(),
			exp: common_expiry.clone(),
			idempotency: "test4".to_string()
		}.sign_with_key(&*key).unwrap();
		let req = test::TestRequest::put()
			.uri("/menu")
			.header(
				actix_web::http::header::AUTHORIZATION,
				format!("Bearer {}", auth)
			).set_json(&prod)
			.to_request();
		let resp: Product = test::read_body_json(
			service.call(req).await.unwrap()
		).await;
		assert_eq!(resp, prod);
	}
}
