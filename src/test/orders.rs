use crate::{api::order::new_order, middleware::auth, model::*};
use actix_web::{dev::Service, http, test};
use jwt::SignWithKey;
use sqlx::types::chrono;

//Common JSON Auth Token
static AUTH: String = {
	let token = auth::AuthToken {
		sub: String::from("test"),
		exp: chrono::Utc::today()
			.succ()
			.succ()
			.and_hms(0, 0, 0)
			.with_timezone(&chrono::FixedOffset::east(0)),
		idempotency: String::from("test"),
	};
	format!("Bearer {}", token.sign_with_key(&super::JWT_KEY).unwrap())
};

#[actix_rt::test]
pub(super) async fn orders_test() {
	let database = super::get_database().await;
	crate::MIGRATOR.run(&database).await.unwrap();

	let mut service = test::init_service(
		actix_web::App::new()
			.data(database)
			.wrap(auth::JWTAuth(unsafe {
				crate::pointer::SharedPointer::new(&super::JWT_KEY)
			}))
			.service(crate::api::order::get_service(None))
			.service(crate::api::menu::get_service(None)),
	)
	.await;

	//Common Product
	let prod = {
		let req = test::TestRequest::put()
			.uri("/menu")
			.header(http::header::AUTHORIZATION, AUTH)
			.set_json(&super::EXAMPLE_PRODUCT)
			.to_request();
		let resp: Product = test::read_body_json(service.call(req).await.unwrap()).await;
		*resp.id()
	};

	//Wrong PUTs
	{
		let req = test::TestRequest::put()
			.uri("/order")
			.header(http::header::AUTHORIZATION, AUTH)
			.set_json(&new_order(
				String::from("Test"),
				vec![(prod, 4)], //4 is over max_num for the product
			))
			.to_request();
		let resp = service.call(req).await.unwrap();
		assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST);
	}
	{
		let req = test::TestRequest::put()
			.uri("/order")
			.header(http::header::AUTHORIZATION, AUTH)
			.set_json(&new_order(
				String::from("Test"),
				vec![(prod, 4)], //Empty carts aren't accepted
			))
			.to_request();
		let resp = service.call(req).await.unwrap();
		assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST);
	}

	//Expected return from both the PUT and the GET
	let expected = Order {
		id: 2,
		owner: String::from("test"),
		owner_name: String::from("Test"),
		cart: vec![(prod, 3)],
	};

	//Correct PUT
	{
		let req = test::TestRequest::put()
			.uri("/order")
			.header(http::header::AUTHORIZATION, AUTH)
			.set_json(&new_order(String::from("Test"), vec![(prod, 3)]))
			.to_request();
		let resp: Order = test::read_body_json(service.call(req).await.unwrap()).await;
		assert_eq!(resp, expected);
	}
	//GET
	{
		let req = test::TestRequest::get()
			.uri("/order")
			.header(actix_web::http::header::AUTHORIZATION, AUTH)
			.to_request();
		let resp: Vec<Order> = test::read_body_json(service.call(req).await.unwrap()).await;
		//We expect what we have just PUT
		assert_eq!(resp[0], expected);
	}
}
