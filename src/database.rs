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

pub async fn get_database(db_url: &str)->Result<(MySqlPool, oneshot::Sender<()>, impl std::future::Future), Error> {
	let conn = MySqlPool::connect(db_url).await?;
	/*Check last order list addition,
		truncate if older than a day
	  Return truncator future*/
	let latest_insertion = sqlx::query!(
		"SELECT day FROM orders ORDER BY id DESC LIMIT 1"
	).fetch_one(&conn).await;
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
	let cleaning_routine = {
		let conn = conn.clone();
		async move {
			log::debug!("Scheduled Database wiper");
			let mut recv = recv.fuse();
			loop {
				futures::select_biased! {
					_ = recv => break,
					is_past_midnight = crate::wait_until_midnight() => if is_past_midnight {
						log::debug!("Truncating Database");
						delete_data(&conn).await.unwrap();
					}
				}
			}
		}
	};
	Ok((conn, routine_stopper, cleaning_routine))
}

#[inline]
async fn delete_data(db: &MySqlPool) -> Result<(), Error> {
	let mut tx = db.begin().await?;
	sqlx::query!(
		"TRUNCATE TABLE orders"
	).execute(&mut tx)
		.await?;
	sqlx::query!(
		"TRUNCATE TABLE carts"
	).execute(&mut tx)
		.await?;
	sqlx::query!(
		"ALTER TABLE orders AUTO_INCREMENT = 1"
	).execute(&mut tx)
		.await?;
	tx.commit().await?;
	Ok(())
}
