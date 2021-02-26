use std::cmp::PartialEq;
use super::{
	Product,
	Order
};

impl PartialEq for Product {
	fn eq(&self, other: &Self) -> bool {
		self.id == other.id || (
			self.kind == other.kind &&
			self.name == other.name &&
			self.price == other.price &&
			self.max_num == other.max_num &&
			self.ingredients == other.ingredients &&
			self.image == other.image
		)
	}
}

impl PartialEq for Order {
	fn eq(&self, other: &Self) -> bool {
		self.id == other.id || (
			self.owner == other.owner &&
			self.owner_name == other.owner_name &&
			self.cart == other.cart
		)
	}
}
