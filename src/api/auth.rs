use super::prelude::*;

pub(super) struct AuthToken {
	pub(super) account_id: String
}

pub(super) fn jwt_guard(req: &actix_web::dev::RequestHead) -> bool {
	use hmac::NewMac;
	let key = hmac::Hmac::<sha2::Sha256>::new_varkey(b"Test")
    .unwrap();

	//Authorization: Bearer <claims>.<signature>
	let auth_value = req.headers.get("Authorization")
    .map(
			|value| value.to_str()
				.map(
					|value| value.trim()
						.strip_prefix("Bearer")
				)
		);
	use log::error;
	match auth_value {
		Some(Ok(Some(token))) => {
			use jwt::{VerifyWithKey, Error};
			use std::collections::HashMap;
			let claims: Result<HashMap<String, String>, Error> = token.verify_with_key(&key);
			match claims {
				Ok(claims) => {
					if let Some(acc_id) = claims.remove("sub") {
						req.extensions_mut().insert(
							AuthToken {
								account_id: acc_id
							}
						);
						return true
					} else {
						error!("Can't find sub(ject) in jwt");
					}
				},
				Err(error) => if log::log_enabled!(log::Level::Error){
					error!("Can't read Authorization header: {}", error)
				}
			}
		},
		Some(Ok(None)) => error!("Can't read Bearer token"),
		Some(Err(error)) => {
			if log::log_enabled!(log::Level::Error){
				error!("Can't read Authorization header: {}", error)
			}
		},
		None => error!("Can't find Authorization header")
	}
	false
}
