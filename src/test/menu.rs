use crate::model::*;
use actix_web::test;
use actix_web::dev::Service;

pub(super) async fn menu_test(database: &sqlx::MySqlPool) {
	let mut service = test::init_service(
		actix_web::App::new()
			.data(database.clone())
			.service(crate::api::menu::get_service())
	).await;

	let prod = Product {
		id: 1,
		kind: ProductKind::Available,
		name: String::from("Test"),
		price: 100,
		max_num: 3,
		ingredients: None,
		image: vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]
	};

	//Put
	{
		let req = test::TestRequest::put()
			.uri("/menu")
			.set_json(&prod)
			.to_request();
		let resp: Product = test::read_body_json(
			service.call(req).await.unwrap()
		).await;
		assert_eq!(resp, prod, "Expected: {:?}\n\nResponse: {:?}", prod, resp);
	}
	//Get
	{
		let req = test::TestRequest::get()
			.uri("/menu")
			.to_request();
		let resp: Vec<Product> = test::read_body_json(
			service.call(req).await.unwrap()
		).await;
		assert_eq!(resp[0], prod, "Expected: {:?}\n\nResponse: {:?}", prod, resp);
	}
	//Delete
	{
		let req = test::TestRequest::delete()
			.uri("/menu/1")
			.to_request();
		let resp: Product = test::read_body_json(
			service.call(req).await.unwrap()
		).await;
		assert_eq!(resp, prod, "Expected: {:?}\n\nResponse: {:?}", prod, resp);
		let req = test::TestRequest::get()
			.uri("/menu")
			.to_request();
		let resp: Vec<Product> = test::read_body_json(
			service.call(req).await.unwrap()
		).await;
		assert_eq!(resp.len(), 0, "Response: {:?}", resp);
	}
}

