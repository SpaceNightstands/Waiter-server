use crate::model::*;
use actix_web::dev::Service;
use actix_web::test;

#[actix_rt::test]
pub(super) async fn menu_test() {
	let database = super::get_database().await;
	crate::MIGRATOR.run(&database).await.unwrap();

	let mut service = test::init_service(
		actix_web::App::new()
			.data(database)
			.service(crate::api::menu::get_service(None)),
	)
	.await;

	//Put
	{
		let req = test::TestRequest::put()
			.uri("/menu")
			.set_json(&super::EXAMPLE_PRODUCT)
			.to_request();
		let resp: Product = test::read_body_json(service.call(req).await.unwrap()).await;

		assert_eq!(&resp, &super::EXAMPLE_PRODUCT);
	}
	//Get
	{
		let req = test::TestRequest::get().uri("/menu").to_request();
		let resp: Vec<Product> = test::read_body_json(service.call(req).await.unwrap()).await;
		assert_eq!(&resp[0], &super::EXAMPLE_PRODUCT);
	}
	//Delete
	{
		let req = test::TestRequest::delete().uri("/menu/1").to_request();
		let resp: Product = test::read_body_json(service.call(req).await.unwrap()).await;
		assert_eq!(&resp, &super::EXAMPLE_PRODUCT);

		//Now the menu should be empty
		let req = test::TestRequest::get().uri("/menu").to_request();
		let resp: Vec<Product> = test::read_body_json(service.call(req).await.unwrap()).await;
		assert_eq!(resp.len(), 0, "Response: {:?}", resp);
	}
}
