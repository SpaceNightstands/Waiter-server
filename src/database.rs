use actix_web::rt;
use sqlx::{
	Error,
	types::chrono::{
		self,
		Local
	}
};
use futures::{
	channel::oneshot,
	future::FutureExt
};
use super::Pool;

pub async fn get_database(db_url: &str)->Result<(Pool, oneshot::Sender<()>), Error> {
	let conn = Pool::connect(db_url).await?;

	let mut transaction = conn.begin().await?;
	crate::MIGRATOR.run(&mut transaction).await?;
	transaction.commit().await?;

	/*Delete every order that's been made before today*/
	sqlx::query!(
		"DELETE FROM orders WHERE day < ?",
		Local::today()
			.and_hms(0, 0, 0)
			.with_timezone(&chrono::Utc)
	).execute(&conn).await?;

	let (routine_stopper, recv) = oneshot::channel::<()>();
	{
		let conn = conn.clone();
		rt::spawn(
			async move {
				log::debug!("Scheduled Database wiper");
				let mut recv = recv.fuse();
				loop {
					//Wipe the orders table everyday
					futures::select_biased! {
						_ = recv => break,
						is_past_midnight = crate::until_midnight() => if is_past_midnight {
							log::debug!("Truncating Database");
							delete_data(&conn).await.unwrap();
						}
					}
				}
			}
		);
	};
	Ok((conn, routine_stopper))
}

#[inline]
async fn delete_data(db: &Pool) -> Result<(), Error> {
	//Wipe orders and reset the auto incrementing index to 1
	let mut transaction = db.begin().await?;
	sqlx::query!(
		"DELETE FROM orders"
	).execute(&mut transaction).await?;
	sqlx::query!(
		"ALTER TABLE orders AUTO_INCREMENT = 1"
	).execute(&mut transaction).await?;
	transaction.commit().await?;
	Ok(())
}
