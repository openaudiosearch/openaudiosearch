use crate::couch::CouchError;
use oas_common::{DecodingError, EncodingError, ValidationError};
use okapi::openapi3::Responses;
use rocket::http::{Header, Status};
use rocket::response::Responder;
use rocket::{response, response::content, Request};
use rocket_okapi::gen::OpenApiGenerator;
use rocket_okapi::response::OpenApiResponderInner;
use rocket_okapi::util::add_schema_response;
use schemars::JsonSchema;
use serde::Serialize;

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
    #[error("Validation error: {0}")]
    ValidationError(ValidationError),
    #[error("Unauthorized")]
    Unauthorized,
}

impl<'r> Responder<'r, 'static> for AppError {
    fn respond_to(self, req: &'r Request<'_>) -> response::Result<'static> {
        let code = match &self {
            AppError::Couch(err) => map_u16_status(err.status_code()),
            AppError::Http(code, _) => *code,
            AppError::EncodingError(_) => Status::BadRequest,
            AppError::ValidationError(_) => Status::UnprocessableEntity,
            AppError::Elastic(err) => map_u16_status(err.status_code().map(|code| code.as_u16())),
            AppError::Unauthorized => Status::Unauthorized,
            _ => Status::InternalServerError,
        };

        let message = match &self {
            AppError::Http(_code, message) => message.clone(),
            _ => format!("{}", self),
        };

        let response = ErrorResponse { error: message };

        // let json = json!({ "error": message });
        let json_string = serde_json::to_string(&response).unwrap();
        let res = content::Json(json_string).respond_to(req);

        // Handle authentication error: Add WWW-Authenticate header.
        match res {
            Err(res) => Err(res),
            Ok(mut res) => {
                res.set_status(code);
                match self {
                    Self::Unauthorized => {
                        let header_value = format!(
                            r#"Basic realm="{}", charset="UTF-8""#,
                            "Please enter user username and password"
                        );
                        let header =
                            Header::new(http::header::WWW_AUTHENTICATE.as_str(), header_value);
                        res.set_header(header);
                    }
                    _ => {}
                }
                Ok(res)
            }
        }
        // let res = Custom(code, content::Json(json_string)).respond_to(req);
    }
}

fn map_u16_status(status: Option<u16>) -> Status {
    status
        .map(|code| Status::from_code(code).unwrap())
        .unwrap_or(Status::InternalServerError)
}

#[derive(Serialize, JsonSchema, Debug, Default)]
struct ErrorResponse {
    error: String,
}

impl OpenApiResponderInner for AppError {
    fn responses(gen: &mut OpenApiGenerator) -> rocket_okapi::Result<Responses> {
        let mut responses = Responses::default();
        let schema = gen.json_schema::<ErrorResponse>();
        // TODO: Find out how different codes are displayed.
        add_schema_response(&mut responses, 500, "application/json", schema)?;
        Ok(responses)
    }
}
