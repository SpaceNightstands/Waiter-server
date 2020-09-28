use serde::{Serialize, Deserialize};

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
	pub owner: String,
	pub cart: String,
}

impl std::fmt::Debug for Order {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      write!(
				f, "Order{{\n\tid: {}\n\towner: {}\n\tcart: {:?}\n}}",
				self.id, self.owner, self.cart
			)
    }
}

