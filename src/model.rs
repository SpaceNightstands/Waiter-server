use std::{
	fmt::Debug,
	pin::Pin
};
use serde::{Serialize, Deserialize};
use sqlx::{
	Database,
	Decode,
	Encode,
	Type
};
use derive_getters::Getters;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all="lowercase")]
pub enum ProductKind {
	Available,
	Orderable,
	Beverage
}

#[derive(Serialize, Deserialize, Getters, sqlx::FromRow, Debug, PartialEq, Eq)]
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

		let mut encoded_image = String::new();
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
		Ok(image)
	}
}

#[derive(Serialize, Deserialize, Getters, Debug, PartialEq, Eq)]
pub struct Order {
	#[serde(default)]
	pub(crate) id: u32,
	pub(crate) owner: String,
	pub(crate) owner_name: String,
	pub(crate) cart: Vec<(u32, u32)>,
}

impl std::convert::Into<&'static str> for &ProductKind {
	fn into(self) -> &'static str {
		use ProductKind::*;
		match self {
			Available => "available",
			Orderable => "orderable",
			Beverage => "beverage"
		}
	}
}

impl Order {
	pub(crate) fn cart_mut(&mut self) -> Pin<&mut Vec<(u32, u32)>> {
		//TODO: make pin effective
		Pin::new(&mut self.cart)
	}
}

impl<DB> Type<DB> for ProductKind
where
	DB: Database,
	&'static str: Type<DB>
{
	fn type_info() -> DB::TypeInfo {
		<&'static str as Type<DB>>::type_info()
	}

	fn compatible(ty: &DB::TypeInfo) -> bool {
		<&'static str as Type<DB>>::compatible(ty)
	}
}

impl<'q, DB> Encode<'q, DB> for ProductKind
where
	DB: Database,
	&'q str: Encode<'q, DB>
{
	fn encode_by_ref(&self, buf: &mut <DB as sqlx::database::HasArguments<'q>>::ArgumentBuffer) -> sqlx::encode::IsNull {
		let string: &'static str = self.into();
		string.encode(buf)
	}

	fn produces(&self) -> Option<DB::TypeInfo> {
		<&'q str as Encode<'q, DB>>::produces(&self.into())
	}

	fn size_hint(&self) -> usize {
		<&'q str as Encode<'q, DB>>::size_hint(&self.into())
	}
}

impl<'r, DB> Decode<'r, DB> for ProductKind
where
	DB: Database,
	&'r str: Decode<'r, DB>
{
	fn decode(value: <DB as sqlx::database::HasValueRef<'r>>::ValueRef) -> Result<Self, sqlx::error::BoxDynError> {
		use ProductKind::*;
		let kind = <&str as Decode<DB>>::decode(value)?;
		match kind {
			"available" => Ok(Available),
			"orderable" => Ok(Orderable),
			"beverage" => Ok(Beverage),
			_ => Err(error::EnumError("Invalid ENUM variant").into())
		}
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
