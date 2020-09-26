#[derive(serde::Serialize)]
pub struct Product {
	pub id: u32,
	pub kind: u8,
	pub name: String 
}

impl std::fmt::Debug for Product {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\t{}", self.id, self.name)
    }
}
