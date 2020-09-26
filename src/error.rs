use sqlx::Error as sqlx;

pub enum Error {
	SQLx(sqlx)
}

impl Error {
	pub fn new<E: Into<Error>>(error: E) -> Error {
		error.into()
	}
}

impl From<sqlx> for Error {
    fn from(e: sqlx) -> Self {
			Error::SQLx(e)
    }
}

impl actix_web::error::ResponseError for Error {
    fn status_code(&self) -> actix_web::http::StatusCode {
			actix_web::http::StatusCode::INTERNAL_SERVER_ERROR
    }

    fn error_response(&self) -> actix_web::HttpResponse {
			use log::{
				log_enabled, error, Level::Error
			};
			if log_enabled!(Error) {
				error!("{}", self)
			}
			let mut resp = actix_web::HttpResponse::new(self.status_code());
			resp.headers_mut().insert(
					actix_web::http::header::CONTENT_TYPE,
					actix_web::http::HeaderValue::from_static("text/plain; charset=utf-8"),
			);
			resp.set_body(actix_web::dev::Body::Empty)
    }
}

use std::fmt::{Display, Debug};
impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
			Display::fmt(
				match self {
					Error::SQLx(error) => error
				},
				f
			)
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
			Debug::fmt(
				match self {
					Error::SQLx(error) => error
				},
				f
			)
    }
}
