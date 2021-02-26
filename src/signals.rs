use actix_web::dev::Server;
//TODO: Use single atomic boolean
use futures::{
	future::FutureExt,
	channel::oneshot::Sender
};

#[cfg(not(unix))]
pub(super) fn handle_kill_signals(server: Server, database_stopper: Sender<()>, cache_stopper: Sender<()>){
	actix_rt::spawn(
		actix_rt::signal::ctrl_c()
			.then(
				|_| async move {
					log::debug!("Received Ctrl-C");
					stopper(server, database_stopper, cache_stopper).await
				}
			)
	);
}

#[cfg(unix)]
pub(super) fn handle_kill_signals(server: Server, database_stopper: Sender<()>, cache_stopper: Sender<()>){
	/*On *nix register a listener for every terminating
	 *signal*/
	use actix_rt::signal::unix::{
		self,
		SignalKind
	};
	let mut signals = Vec::new();
	let signal_list: [SignalKind; 4] = [
		SignalKind::interrupt(),
		SignalKind::hangup(),
		SignalKind::terminate(),
		SignalKind::quit(),
	];
	for kind in signal_list.iter() {
		match unix::signal(*kind) {
			Ok(stream) => signals.push(stream),
			Err(e) => if log::log_enabled!(log::Level::Error) {
				log::error!(
					"Cannot initialize stream handler for {:?} err: {}",
					kind, e
				)
			}
		}
	}

	//Poll every stream and stop everything if any signal is received
	use std::task::Poll;
	actix_rt::spawn(
		futures::future::poll_fn(
			move |ctx|{
				for sig in signals.iter_mut() {
					if let Poll::Ready(Some(())) = sig.poll_recv(ctx) {
						return Poll::Ready(())
					}
				}
				Poll::Pending
			}
		).then(
			|_| async move {
				stopper(server, database_stopper, cache_stopper).await
			}
		)
	)
}

#[inline]
async fn stopper(server: Server, database: Sender<()>, cache: Sender<()>) {
	//TODO: Replace with blocking channels
	database.send(()).unwrap();
	cache.send(()).unwrap();
	server.stop(true).await;
}
