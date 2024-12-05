use askama::Template;
use axum::{extract::State, response::IntoResponse, Form};
use serde::Deserialize;

use crate::{error::Result, AppPtr};

#[derive(Template)]
#[template(path = "search.html")]
struct SearchHtml {}

pub async fn index(ctx: State<AppPtr>) -> Result<impl IntoResponse> {
    Ok(SearchHtml {})
}

#[derive(Deserialize)]
pub struct SearchForm {
    query: String,
}

pub async fn search(
    state: State<AppPtr>,
    Form(search): Form<SearchForm>,
) -> Result<impl IntoResponse> {
    Ok(SearchHtml {})
}
