mod menu;
mod orders;
mod integration;
use sqlx::MySqlPool;

#[actix_rt::test]
async fn test() {
	let db = MySqlPool::connect(dotenv_codegen::dotenv!("TEST_DATABASE_URL")).await
		.unwrap();
	wipe_db(&db).await;

	//TODO: make concurrent tests
	/*As of right now, these rely on the database being empty
	  which means that we can't test multiple endpoints/actix services safely*/
	menu::menu_test(&db).await;
	orders::orders_test(&db).await;
	integration::integration_test(&db).await;

	/*If an assertion fails, the unwinding
	  test won't wipe the database.
	  the actix runtime doesn't let us use block_on
	  if there's a System Runtime running.*/
	//TODO: make all tests return Result instead of panic, use try_join!, 
	//clean database before handling errors
	wipe_db(&db).await;
}

#[inline]
async fn wipe_db(db: &MySqlPool) {
	let queries = [
		"TRUNCATE TABLE carts",
		"DELETE FROM orders",
		"ALTER TABLE orders AUTO_INCREMENT = 1",
		"DELETE FROM products",
		"ALTER TABLE products AUTO_INCREMENT = 1"
	];
	for query in queries.iter() {
		sqlx::query(query).execute(db).await.unwrap();
	}
}
