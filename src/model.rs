use serde::{Serialize, Deserialize};
use sqlx::types::chrono::{ self, NaiveDate };

#[derive(Serialize, Deserialize)]
pub struct Product {
	/*0 is the default for all numbers
	 *since AUTO_INCREMENT starts from 1
	 *0 is our None (in the contexts where
	 *id matters)*/
	#[serde(default)]
	pub id: u32,
	pub kind: u8,
	pub name: String 
}

impl std::fmt::Debug for Product {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Product{{\n\tid: {}\n\tkind: {}\n\tname: {}\n}}", self.id, self.kind, self.name)
    }
}

#[derive(Serialize, Deserialize)]
pub struct Order {
	#[serde(default)]
	pub id: u32,
	#[serde(
		serialize_with = "serialize_date",
		deserialize_with = "deserialize_date"
	)]
	pub day: NaiveDate,
	pub owner: String,
	pub cart: String,
}

impl std::fmt::Debug for Order {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      write!(
				f, "Order{{\n\tid: {}\n\tday: {}\n\towner: {}\n\tcart: {:?}\n}}",
				self.id, self.day, self.owner, self.cart
			)
    }
}

fn serialize_date<S: serde::Serializer>(day: &NaiveDate, ser: S) -> Result<S::Ok, S::Error> {
	ser.serialize_i64(
		day.and_hms(0, 0, 0)
			 .timestamp()
	)
}

fn deserialize_date<'de, D: serde::Deserializer<'de>>(de: D) -> Result<NaiveDate, D::Error> {
	let ts = <i64 as serde::Deserialize>::deserialize(de)?;
	Ok(chrono::NaiveDateTime::from_timestamp(ts, 0).date())
}
