use crate::model::*;
use actix_web::test;
use actix_web::dev::Service;
/*
async fn get_orders(db: web::Data<MySqlPool>, req: web::HttpRequest) -> Result<impl Responder, Error>
async fn put_orders(db: web::Data<MySqlPool>, mut cart: web::Json<Vec<(u32, u32)>>, req: web::HttpRequest) -> Result<impl Responder, Error>
*/

pub(super) async fn orders_test(database: sqlx::MySqlPool) {
	let mut service = test::init_service(
		actix_web::App::new()
			.data(database.clone())
			.service(crate::api::order::get_service())
			.service(crate::api::menu::get_service())
	).await;

	let prod = {
		let req = test::TestRequest::put()
			.uri("/menu")
			.set_json(
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
		resp
	};

	//Get
	{
		let req = test::TestRequest::get()
			.uri("/order")
			.to_request();
		let resp: Vec<Product> = test::read_body_json(
			service.call(req).await.unwrap()
		).await;
		assert_eq!(resp[0], prod, "Sample: {:?}\n\nResponse: {:?}", prod, resp);
	}
}
