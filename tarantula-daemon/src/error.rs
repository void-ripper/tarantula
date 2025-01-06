use std::fmt::Display;

use askama::Template;
use axum::{http::StatusCode, response::IntoResponse};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error {
    pub line: u32,
    pub module: String,
    pub msg: String,
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}[{}] => {}", self.module, self.line, self.msg)
    }
}

#[derive(Template)]
#[template(path = "error.html")]
struct ErrorHtml {
    msg: String,
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorHtml {
                msg: format!("{}[{}] => {}", self.module, self.line, self.msg),
            }
            .render()
            .unwrap(),
        )
            .into_response()
    }
}
