use std::{
	fmt::Debug,
	pin::Pin
};
use serde::{Serialize, Deserialize};
use sqlx::{
	Type,
	FromRow
};
use derive_getters::Getters;

#[derive(Serialize, Deserialize, Type, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all="lowercase")]
#[sqlx(rename_all = "lowercase")]
pub enum ProductKind {
	Available,
	Orderable,
	Beverage
}

#[derive(Serialize, Deserialize, Getters, FromRow, Debug, PartialEq, Eq)]
pub struct Product {
	#[serde(default)]
	pub(crate) id: u32,
	pub(crate) kind: ProductKind, 
	pub(crate) name: String,
	pub(crate) price: u16, 
	pub(crate) max_num: u8,
	pub(crate) ingredients: Option<String>,
	#[serde(with = "image_from_base64")]
	pub(crate) image: Vec<u8> 
}

mod image_from_base64 {
	use serde::{
		Serializer,
		Deserializer
	};
	use base64_stream::{
		FromBase64Reader as Decoder,
		ToBase64Reader as Encoder
	};
	use std::io::Read;

	pub(super) fn serialize<S>(image: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error>
	where S: Serializer {
		use serde::ser::Error;

		//This capacity is not enough, but will avoid allocations for much of the conversion.
		let mut encoded_image = String::with_capacity(image.len());
		Encoder::new(&**image) // &[u8] implements Read
			.read_to_string(&mut encoded_image)
			.map_err(S::Error::custom)?;

		serializer.serialize_str(&*encoded_image)
	}

	pub(super) fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
	where D: Deserializer<'de> {
		use serde::de::Error;

		let base64: &'de str = serde::Deserialize::deserialize(deserializer)?;
		let mut image = Vec::with_capacity(base64.as_bytes().len());
		Decoder::new(base64.as_bytes())
			.read_to_end(&mut image)
			.map_err(D::Error::custom)?;
		image.shrink_to_fit();
		Ok(image)
	}
}

#[derive(Serialize, Deserialize, Getters, Debug)]
pub struct Order {
	#[serde(default)]
	pub(crate) id: u32,
	#[serde(default)]
	pub(crate) owner: String,
	pub(crate) owner_name: String,
	pub(crate) cart: Vec<(u32, u32)>,
}

impl Order {
	pub(crate) fn cart_mut(&mut self) -> Pin<&mut Vec<(u32, u32)>> {
		//TODO: make pin effective
		Pin::new(&mut self.cart)
	}
}

mod error {
	#[derive(Debug)]
	pub(crate) struct EnumError(pub(crate) &'static str);

	impl std::error::Error for EnumError {}

	impl std::fmt::Display for EnumError {
		fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
			f.write_str(self.0)
		}
	}
}

