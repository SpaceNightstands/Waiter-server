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

pub(super) use impls::*;

#[cfg(not(test))]
mod impls {
	use super::*;

	// sqlx::Type doesn't work that well with the ENUM SQL Type
	#[derive(Serialize, Deserialize, Debug)]
	#[serde(rename_all="lowercase")]
	pub enum ProductKind {
		Available,
		Orderable,
		Beverage
	}

	#[derive(Serialize, Deserialize, Getters, sqlx::FromRow, Debug)]
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
		pub(crate) id: u32,
		pub(crate) owner: String,
		pub(crate) cart: Vec<(u32, u32)>,
	}
}

#[cfg(test)]
mod impls {
	use super::*;

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
		pub(crate) image: Vec<u8> 
	}

	#[derive(Serialize, Deserialize, Getters, Debug, PartialEq, Eq)]
	pub struct Order {
		#[serde(default)]
		pub(crate) id: u32,
		pub(crate) owner: String,
		pub(crate) cart: Vec<(u32, u32)>,
	}
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

impl Product {
	pub(crate) fn new(id: u32, kind: ProductKind, name: String, price: u16, max_num: u8, ingredients: Option<String>, image: Vec<u8>) -> Self{
		Product {
			id,
			kind,
			name,
			price,
			max_num,
			ingredients,
			image
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
			_ => Err(crate::error::EnumError("Invalid ENUM variant").into())
		}
	}
}

