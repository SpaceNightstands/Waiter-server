use serde::{Serialize, Deserialize};
use std::fmt::Debug;
use derive_getters::Getters;

#[derive(Serialize, Deserialize, Getters, Debug)]
pub struct Product {
	/*0 is the default for all numbers
	 *since AUTO_INCREMENT starts from 1
	 *0 is our None (in the contexts where
	 *id matters)*/
	#[serde(default)]
	pub(super) id: u32,
	//Can only be "available", "orderable" or "beverage"
	pub(super) kind: String, 
	pub(super) name: String 
}

#[derive(Serialize, Deserialize, Getters, Debug)]
pub struct Order {
	#[serde(default)]
	pub(super) id: u32,
	pub(super) owner: String,
	pub(super) cart: String,
}

