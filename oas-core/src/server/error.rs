use crate::couch::CouchError;
use oas_common::{DecodingError, EncodingError};
use rocket::http::Status;
use rocket::response::status::Custom;
use rocket::response::Responder;
use rocket::{response, response::content, Request};
use serde_json::json;
use thiserror::Error;

pub type Result<T> = std::result::Result<rocket::serde::json::Json<T>, AppError>;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("{0}")]
    DecodingError(#[from] DecodingError),
    #[error("{0}")]
    EncodingError(#[from] EncodingError),
    #[error("{0}")]
    Couch(#[from] CouchError),
    #[error("{0}")]
    Serde(#[from] serde_json::Error),
    #[error("{0}")]
    Other(String),
    #[error("{0}")]
    Elastic(#[from] elasticsearch::Error),
    #[error("HTTP error: {0} {1}")]
    Http(Status, String),
}

impl<'r> Responder<'r, 'static> for AppError {
    fn respond_to(self, req: &'r Request<'_>) -> response::Result<'static> {
        let code = match &self {
            // TODO: Change to 500
            AppError::Couch(_err) => Status::BadGateway,
            AppError::Http(code, _) => *code,
            AppError::EncodingError(_) => Status::BadRequest,
            AppError::Elastic(err) => err
                .status_code()
                .map(|code| Status::from_code(code.as_u16()).unwrap())
                .unwrap_or(Status::InternalServerError),
            _ => Status::InternalServerError,
        };

        let message = match self {
            AppError::Http(_code, message) => message,
            _ => format!("{}", self),
        };

        let json = json!({ "error": message });
        let json_string = serde_json::to_string(&json).unwrap();
        Custom(code, content::Json(json_string)).respond_to(req)
    }
}
