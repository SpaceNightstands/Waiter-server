use serde::{Serialize, Deserialize};
use derive_getters::Getters;

#[derive(Serialize, Deserialize, Getters)]
pub struct Product {
	/*0 is the default for all numbers
	 *since AUTO_INCREMENT starts from 1
	 *0 is our None (in the contexts where
	 *id matters)*/
	#[serde(default)]
	id: u32,
	kind: u8,
	name: String 
}

impl std::fmt::Debug for Product {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Product{{\n\tid: {}\n\tkind: {}\n\tname: {}\n}}", self.id, self.kind, self.name)
    }
}

#[derive(Serialize, Deserialize, Getters)]
pub struct Order {
	#[serde(default)]
	id: u32,
	owner: String,
	cart: String,
}

impl std::fmt::Debug for Order {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      write!(
				f, "Order{{\n\tid: {}\n\towner: {}\n\tcart: {:?}\n}}",
				self.id, self.owner, self.cart
			)
    }
}

