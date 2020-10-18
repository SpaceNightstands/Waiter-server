mod menu;
mod orders;

use lazy_static::lazy_static;
use sqlx::MySqlPool;

lazy_static! {
	static ref DATABASE: MySqlPool = {
		MySqlPool::connect_lazy(dotenv_codegen::dotenv!("TEST_DATABASE_URL"))
			.unwrap()
	};
}
