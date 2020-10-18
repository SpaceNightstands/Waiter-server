use crate::model::*;
use actix_web::test::TestRequest;
/*
async fn get_menu(db: web::Data<MySqlPool>) -> impl Responder
async fn put_menu(db: web::Data<MySqlPool>, prod: web::Json<Product>) -> Result<impl Responder, DBError>
async fn delete_menu(db: web::Data<MySqlPool>, web::Path(id): web::Path<u32>) -> Result<impl Responder, DBError>
*/
/*
pub struct Product {
	/*0 is the default for all numbers
	 *since AUTO_INCREMENT starts from 1
	 *0 is our None (in the contexts where
	 *id matters)*/
	#[serde(default)]
	pub(super) id: u32,
	pub(super) kind: ProductKind, 
	pub(super) name: String,
	pub(super) price: u16, 
	pub(super) max_num: u8,
	pub(super) ingredients: Option<String>,
	pub(super) image: Vec<u8> 
}
*/
#[actix_rt::test]
async fn menu_test() {
	let req = TestRequest::get()
		.set_json(
			&Product {
				id: 0,
				kind: ProductKind::Available,
				name: String::from("Test"),
				price: 100, //Divide it by 100
				max_num: 3,
				ingredients: None,
				image: vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]
			}
		).data(
			super::DATABASE.clone()
		);
}
