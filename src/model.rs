use serde::{Serialize, Deserialize};
use std::fmt::Debug;
use derive_getters::Getters;

#[derive(Serialize, Deserialize, Debug, sqlx::Type)]
#[serde(rename_all="lowercase")]
#[sqlx(rename_all = "lowercase")]
pub enum ProductKind {
	Available,
	Orderable,
	Beverage
}

#[derive(Serialize, Deserialize, Getters, Debug, sqlx::FromRow)]
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

#[derive(Serialize, Deserialize, Getters, Debug)]
pub struct Order {
	#[serde(default)]
	pub(super) id: u32,
	pub(super) owner: String,
	pub(super) cart: Vec<(u32, u32)>,
}

