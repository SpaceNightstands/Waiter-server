use actix_web::dev::Server;
use actix_web::rt::{signal, spawn};
use futures::future::FutureExt;

type JoinHandle = actix_web::rt::task::JoinHandle<()>;

#[cfg(not(unix))]
pub(super) fn handle_kill_signals(
	server: Server, database_stopper: JoinHandle, cache_stopper: JoinHandle,
) {
	spawn(signal::ctrl_c().then(|_| async move {
		log::debug!("Received Ctrl-C");
		stopper(server, database_stopper, cache_stopper).await
	}));
}

#[cfg(unix)]
pub(super) fn handle_kill_signals(
	server: Server, database_stopper: JoinHandle, cache_stopper: JoinHandle,
) {
	/*On *nix register a listener for every terminating
	 *signal*/
	use signal::unix::{self, SignalKind};
	use std::task::Poll;

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
			Err(e) => {
				if log::log_enabled!(log::Level::Error) {
					log::error!("Cannot initialize stream handler for {:?} err: {}", kind, e)
				}
			}
		}
	}

	//Poll every stream and stop everything if any signal is received
	spawn(
		futures::future::poll_fn(move |ctx| {
			for sig in signals.iter_mut() {
				if let Poll::Ready(Some(())) = sig.poll_recv(ctx) {
					return Poll::Ready(());
				}
			}
			Poll::Pending
		})
		.then(|_| async move { stopper(server, database_stopper, cache_stopper).await }),
	);
}

#[inline]
async fn stopper(server: Server, database: JoinHandle, cache: JoinHandle) {
	server.stop(true).await;
	database.abort();
	cache.abort();
}
