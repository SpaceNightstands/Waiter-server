use crate::{
	model::*,
	middleware::*,
	api::*,
	pointer::SharedPointer
};
use actix_web::{
	test,
	dev::Service
};
use sqlx::types::chrono;
use hmac::NewMac;
use jwt::SignWithKey;

#[actix_rt::test]
pub(super) async fn integration_test() {
	let database = super::get_database().await;
	crate::MIGRATOR.run(&database).await.unwrap();

	let key = auth::Key::new_varkey(b"Test").unwrap();
	let cache = dashmap::DashSet::<String>::new();
	let mut filter = std::collections::HashSet::with_capacity(1);
	filter.insert("admin".to_string());

	let mut service = {
		let (key_ref, filter_ref, cache_ref) = unsafe {
			(
				SharedPointer::new(&key),
				SharedPointer::new(&filter),
				SharedPointer::new(&cache)
			)
		};
		test::init_service(
			actix_web::App::new()
				.data(database.clone())
				.wrap(cache::IdempotencyCache(cache_ref))
				.wrap(auth::JWTAuth(key_ref))
				.wrap(actix_web::middleware::Logger::default())
				.service(menu::get_service(Some(filter_ref)))
				.service(order::get_service(Some(filter_ref)))
		)
	}.await;

	//Common JWT exp attribute
	let common_expiry = chrono::Utc::today()
		.succ()
		.succ()
		.and_hms(0, 0, 0)
		.with_timezone(&chrono::FixedOffset::east(0));

	/*Testing idempotency token cache by sending
	 *two requests with the same Authentication Token*/
	{
		let auth = auth::AuthToken {
			sub: "test".to_string(),
			exp: common_expiry.clone(),
			idempotency: "test0".to_string()
		}.sign_with_key(&key).unwrap();
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
		//Unwrap panics if the response is not an error
		resp.as_error::<crate::error::Error>()
			.unwrap();
		//TODO: maybe add an assertion?
	}

	/*Testing subject authentication by sending
	 *two requests and seeing if the server responds
	 *only with the items inserted previously by the same sub(ject)*/
	{
		let auth = auth::AuthToken {
			sub: "admin".to_string(),
			exp: common_expiry.clone(),
			idempotency: "test1".to_string()
		}.sign_with_key(&key).unwrap();
		let req = test::TestRequest::get()
			.uri("/order")
			.header(
				actix_web::http::header::AUTHORIZATION,
				format!("Bearer {}", auth)
			).to_request();
		let resp: Vec<Order> = test::read_body_json(
			service.call(req).await.unwrap()
		).await;
		/*What has been added by subject "test" should not be
		 *visible to admin, therefore the length of this array should
		 *be 0 to indicate the lack of orders made by "admin"*/
		assert_eq!(resp.len(), 0);

		let auth = auth::AuthToken {
			sub: "test".to_string(),
			exp: common_expiry.clone(),
			idempotency: "test2".to_string()
		}.sign_with_key(&key).unwrap();
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

	/*Testing subject filter by sending a request
	 *to one of the protected endpoints with an 
	 *unauthorized subject value first and with
	 *an authorized one afterwards*/
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
		}.sign_with_key(&key).unwrap();
		let req = test::TestRequest::put()
			.uri("/menu")
			.header(
				actix_web::http::header::AUTHORIZATION,
				format!("Bearer {}", auth)
			).set_json(&prod)
			.to_request();
		let resp = service.call(req).await
			//Expecting an error here, "test" isn't authorized
			.err()
			.unwrap();
		println!("{:?}", resp);

		let auth = auth::AuthToken {
			sub: "admin".to_string(),
			exp: common_expiry.clone(),
			idempotency: "test4".to_string()
		}.sign_with_key(&key).unwrap();
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
