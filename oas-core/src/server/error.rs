use crate::couch::CouchError;
use oas_common::DecodingError;
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
    Couch(#[from] CouchError),
    #[error("{0}")]
    Serde(#[from] serde_json::Error),
    #[error("{0}")]
    Other(String),
}

impl<'r> Responder<'r, 'static> for AppError {
    fn respond_to(self, req: &'r Request<'_>) -> response::Result<'static> {
        let json = json!({ "error": format!("{}", self) });
        let string = serde_json::to_string(&json).unwrap();
        let code = match self {
            // TODO: Change to 500
            AppError::Couch(_err) => Status::BadGateway,
            _ => Status::InternalServerError,
        };
        Custom(code, content::Json(string)).respond_to(req)
    }
}
