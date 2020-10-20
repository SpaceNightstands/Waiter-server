use serde::{Serialize, Deserialize};
use sqlx::{
	Database,
	Decode,
	Encode,
	Type
};
use std::fmt::Debug;
use derive_getters::Getters;

// sqlx::Type doesn't work that well with the ENUM SQL Type
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all="lowercase")]
pub enum ProductKind {
	Available,
	Orderable,
	Beverage
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

#[derive(Serialize, Deserialize, Getters, Debug, sqlx::FromRow, Clone, PartialEq, Eq)]
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

#[derive(Serialize, Deserialize, Getters, Debug, Clone, PartialEq, Eq)]
pub struct Order {
	#[serde(default)]
	pub(super) id: u32,
	pub(super) owner: String,
	pub(super) cart: Vec<(u32, u32)>,
}

