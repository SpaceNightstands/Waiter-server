use sqlx::{
	MySqlPool,
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

pub async fn get_database(db_url: &str)->Result<(MySqlPool, oneshot::Sender<()>), Error> {
	let conn = MySqlPool::connect(db_url).await?;
	/*Check last order list addition,
		truncate if older than a day
	  Return truncator future*/
	let latest_insertion = sqlx::query!(
		"SELECT day FROM orders ORDER BY id DESC LIMIT 1"
	).fetch_one(&conn).await;

	/*Wipe the orders table if the latest insertion
	 *has been done before today*/
	match latest_insertion {
		Ok(row) => {
			if row.day.date() < Local::today().with_timezone(&chrono::Utc) {
				delete_data(&conn).await?;
			}
		},
		Err(Error::RowNotFound) => (),
		Err(error) => return Err(error)
	}

	let (routine_stopper, recv) = oneshot::channel::<()>();
	{
		let conn = conn.clone();
		actix_rt::spawn(
			async move {
				log::debug!("Scheduled Database wiper");
				let mut recv = recv.fuse();
				loop {
					//Wipe the orders table everyday
					futures::select_biased! {
						_ = recv => break,
						is_past_midnight = crate::wait_until_midnight() => if is_past_midnight {
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
async fn delete_data(db: &MySqlPool) -> Result<(), Error> {
	//Wipe orders and reset the auto incrementing index to 1
	let mut tx = db.begin().await?;
	sqlx::query!(
		"DELETE FROM orders"
	).execute(&mut tx).await?;
	sqlx::query!(
		"ALTER TABLE orders AUTO_INCREMENT = 1"
	).execute(&mut tx).await?;
	tx.commit().await?;
	Ok(())
}
