use actix_web::{
	body::Body,
	error::ResponseError,
	http::{header::ToStrError as HeaderError, StatusCode},
	BaseHttpResponse,
};
use std::{convert::From, fmt::Display};

#[derive(serde::Serialize, Debug)]
#[serde(untagged)]
pub(crate) enum Error {
	Static {
		#[serde(skip)]
		status: StatusCode,
		#[serde(alias = "type")]
		//Consider using an Enum
		reason: &'static str,
		message: &'static str,
	},
	Passthrough {
		#[serde(skip)]
		status: StatusCode,
		#[serde(alias = "type")]
		reason: &'static str,
		message: String,
	},
}

impl Error {
	pub(crate) fn passthrough<T: std::error::Error>(
		status: StatusCode, reason: &'static str, message: &T,
	) -> Error {
		Error::Passthrough {
			status,
			reason,
			message: message.to_string(),
		}
	}
}

impl Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		use Error::*;
		match self {
			Static { message, .. } => f.write_str(message),
			Passthrough { message, .. } => write!(f, "{}", message),
		}
	}
}

impl From<sqlx::Error> for Error {
	fn from(error: sqlx::Error) -> Self {
		Error::Passthrough {
			status: StatusCode::INTERNAL_SERVER_ERROR,
			reason: "Database",
			message: error.to_string(),
		}
	}
}

impl From<jwt::Error> for Error {
	fn from(error: jwt::Error) -> Self {
		Error::Passthrough {
			status: StatusCode::UNAUTHORIZED,
			reason: "Authorization",
			message: error.to_string(),
		}
	}
}

impl From<HeaderError> for Error {
	fn from(error: HeaderError) -> Self {
		Error::Passthrough {
			status: StatusCode::UNAUTHORIZED,
			reason: "Authorization",
			message: error.to_string(),
		}
	}
}

impl ResponseError for Error {
	fn status_code(&self) -> StatusCode {
		use Error::*;
		match self {
			Static { status, .. } => *status,
			Passthrough { status, .. } => *status,
		}
	}

	fn error_response(&self) -> BaseHttpResponse<Body> {
		use log::{error, log_enabled, Level};
		if log_enabled!(Level::Error) {
			match self {
				Error::Static {
					reason, message, ..
				} => error!("[{}] {}", reason, message),
				Error::Passthrough {
					reason, message, ..
				} => error!("[{}] {}", reason, message),
			}
		}

		//TODO: Change default error message
		let json = serde_json::to_string(self).unwrap_or(String::from("Error serializing error"));

		actix_web::BaseHttpResponse::build(self.status_code())
			.content_type(actix_web::http::header::ContentType::json())
			.body(json)
	}
}
