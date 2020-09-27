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

#[derive(serde::Serialize)]
pub struct SerializableError {
	label: String,
	message: String
}

impl From<&Error> for SerializableError {
    fn from(error: &Error) -> Self {
			SerializableError {
				label: format!("{:?}", error),
				message: format!("{}", error)
			}
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
			actix_web::HttpResponse::build(self.status_code())
				.set_header(
					actix_web::http::header::CONTENT_TYPE,
					actix_web::http::HeaderValue::from_static("text/plain; charset=utf-8"),
				).json::<SerializableError>(
					self.into()
				)
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
