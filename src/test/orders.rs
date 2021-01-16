use crate::{
	model::*,
	middleware::auth,
	api::order::PutOrder
};
use actix_web::{
	test,
	dev::Service,
	http
};
use sqlx::types::chrono;
use hmac::NewMac;
use jwt::SignWithKey;

pub(super) async fn orders_test(database: &sqlx::MySqlPool) {
	let key = auth::Key::new_varkey(
		dotenv_codegen::dotenv!("JWT_SECRET").as_bytes()
	).unwrap();

	let mut service = test::init_service(
		actix_web::App::new()
			.data(database.clone())
			.wrap(
				auth::JWTAuth(
					unsafe {
						crate::pointer::SharedPointer::new(&key)
					}
				)
			).service(crate::api::order::get_service())
			.service(crate::api::menu::get_service(None))
	).await;

	//Common JSON Auth Token
	let auth: String = {
		let auth = auth::AuthToken {
			sub: String::from("test"),
			exp: chrono::Utc::today()
				.succ()
				.succ()
				.and_hms(0, 0, 0)
				.with_timezone(&chrono::FixedOffset::east(0)),
			idempotency: String::from("test")
		};
		auth.sign_with_key(&key).unwrap()
	};

	//Common Product
	let prod = {
		let req = test::TestRequest::put()
			.uri("/menu")
			.header(
				http::header::AUTHORIZATION,
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

	//Wrong PUT's
	{
		let req = test::TestRequest::put()
			.uri("/order")
			.header(
				http::header::AUTHORIZATION,
				format!("Bearer {}", auth)
			).set_json(
				&PutOrder{
					owner_name: String::from("Test"),
					cart: vec![(prod, 4)] //4 is over max_num for the prod product
				}
			).to_request();
		let resp = service.call(req).await.unwrap();
		assert_eq!(
			resp.status(),
			http::StatusCode::INTERNAL_SERVER_ERROR,
			"\nExpected: {:?}\nResponse: {:?}",
			http::StatusCode::INTERNAL_SERVER_ERROR, resp
		);
	}
	{
		let req = test::TestRequest::put()
			.uri("/order")
			.header(
				http::header::AUTHORIZATION,
				format!("Bearer {}", auth)
			).set_json(
				&PutOrder{
					owner_name: String::from("Test"),
					cart: Vec::new() //Empty carts aren't accepted
				}
			).to_request();
		let resp = service.call(req).await.unwrap();
		assert_eq!(
			resp.status(),
			http::StatusCode::BAD_REQUEST,
			"\nExpected: {:?}\nResponse: {:?}",
			http::StatusCode::BAD_REQUEST, resp
		);
	}

	//Expected return from both the PUT and the GET
	let expected = Order {
		id: 2,
		owner: String::from("test"),
		owner_name: String::from("Test"),
		cart: vec![(prod, 3)]
	};

	//Correct PUT
	{
		let req = test::TestRequest::put()
			.uri("/order")
			.header(
				http::header::AUTHORIZATION,
				format!("Bearer {}", auth)
			).set_json(
				&PutOrder{
					owner_name: String::from("Test"),
					cart: vec![(prod, 3)]
				}
			).to_request();
		let resp: Order = test::read_body_json(
			service.call(req).await.unwrap()
		).await;
		assert_eq!(resp, expected);
	}
	//GET
	{
		let req = test::TestRequest::get()
			.uri("/order")
			.header(
				actix_web::http::header::AUTHORIZATION,
				format!("Bearer {}", auth)
			).to_request();
		let resp: Vec<Order> = test::read_body_json(
			service.call(req).await.unwrap()
		).await;
		//We expect what we have just PUT
		assert_eq!(resp[0], expected);
	}
}
