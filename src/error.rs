use std::{
	fmt::Display,
	convert::From
};
use actix_web::{
	error::ResponseError,
	http::{
		StatusCode,
		header::ToStrError as HeaderError
	},
};

#[derive(serde::Serialize, Debug)]
#[serde(untagged)]
pub(crate) enum Error {
	Static {
		#[serde(skip)]
		status: StatusCode,
		#[serde(alias = "type")]
		//Consider using an Enum
		reason: &'static str,
		message: &'static str
	},
	Passthrough {
		#[serde(skip)]
		status: StatusCode,
		#[serde(alias = "type")]
		reason: &'static str,
		message: String
	}
}

impl Error {
	pub(crate) fn Passthrough(status: StatusCode, reason: &'static str, message: &dyn std::error::Error) -> Error {
		Error::Passthrough {
			status,
			reason,
			message: message.to_string()
		}
	}
}

impl Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		use Error::*;
		match self {
			Static{message, ..} => f.write_str(message),
			Passthrough{message, ..} => write!(f, "{}", message)
		}
	}
}

impl From<sqlx::Error> for Error {
	fn from(error: sqlx::Error) -> Self {
		Error::Passthrough {
			status: StatusCode::INTERNAL_SERVER_ERROR,
			reason: "Database",
			message: error.to_string()
		}
	}
}

impl From<jwt::Error> for Error {
	fn from(error: jwt::Error) -> Self {
		Error::Passthrough {
			status: StatusCode::UNAUTHORIZED,
			reason: "Authorization",
			message: error.to_string()
		}
	}
}

impl From<HeaderError> for Error {
	fn from(error: HeaderError) -> Self {
		Error::Passthrough {
			status: StatusCode::UNAUTHORIZED,
			reason: "Authorization",
			message: error.to_string()
		}
	}
}

impl ResponseError for Error {
    fn status_code(&self) -> StatusCode {
			use Error::*;
			match self {
				Static{status, ..} => *status,
				Passthrough{status, ..} => *status
			}
		}

    fn error_response(&self) -> actix_web::HttpResponse {
			use log::{
				log_enabled, error, Level
			};
			if log_enabled!(Level::Error) {
				match self {
					Error::Static{reason, message, ..} => error!("[{}] {}", reason, message),
					Error::Passthrough{reason, message, ..} => error!("[{}] {}", reason, message)
				}
			}
			actix_web::HttpResponse::build(self.status_code())
				.set_header(
					actix_web::http::header::CONTENT_TYPE,
					actix_web::http::HeaderValue::from_static("application/json; charset=utf-8"),
				).json(self)
    }
}

#[derive(Debug)]
pub(crate) struct EnumError(pub(crate) &'static str);

impl std::error::Error for EnumError {}

impl Display for EnumError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str(self.0)
	}
}
