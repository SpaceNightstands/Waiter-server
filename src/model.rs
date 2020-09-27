use serde::{Serialize, Deserialize};
use sqlx::types::{ self, chrono::{ self, NaiveDate } };

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
        write!(f, "{}\t{}", self.id, self.name)
    }
}
