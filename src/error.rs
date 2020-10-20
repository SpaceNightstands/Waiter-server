use std::fmt::Display;
use std::error::Error as stdError;
use sqlx::Error as sqlx;
use actix_web::{
	error::ResponseError,
	http::StatusCode
};

#[derive(serde::Serialize, Debug)]
pub(super) struct DatabaseError{
	label: String,
	message: String
}

impl From<sqlx> for DatabaseError {
    fn from(error: sqlx) -> Self {
			DatabaseError {
				label: format!("{:?}", error),
				message: format!("{}", error)
			}
    }
}

impl Display for DatabaseError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str(&*self.message)
	}
}

impl ResponseError for DatabaseError {
    fn status_code(&self) -> StatusCode {
			StatusCode::INTERNAL_SERVER_ERROR
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
					actix_web::http::HeaderValue::from_static("application/json; charset=utf-8"),
				).json(self)
    }
}

#[derive(Debug)]
pub(super) struct DebugError<T: stdError>(StatusCode, T);

impl<T: stdError> DebugError<T> {
	pub(super) fn new(code: StatusCode, e: T) -> Self {
		Self(code, e)
	}
}

impl<T: stdError> ResponseError for DebugError<T> {
    fn status_code(&self) -> StatusCode {
			self.0
    }
}

impl<T: stdError> Display for DebugError<T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		Display::fmt(&self.1, f)
	}
}

#[derive(Debug)]
pub(super) struct StaticError(StatusCode, &'static str);

impl StaticError {
	pub(super) const fn new(code: StatusCode, e: &'static str) -> Self {
		Self(code, e)
	}
}

impl ResponseError for StaticError {
    fn status_code(&self) -> StatusCode {
			self.0
    }
}

impl Display for StaticError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		Display::fmt(&self.1, f)
	}
}

#[derive(Debug)]
pub(super) struct EnumError(pub(super) &'static str);

impl std::error::Error for EnumError {}

impl Display for EnumError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str(self.0)
	}
}
