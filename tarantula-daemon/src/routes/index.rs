use askama::Template;
use axum::{extract::State, response::IntoResponse, Form};
use serde::Deserialize;

use crate::{
    database::search::SearchResult,
    error::{Error, Result},
    ex, AppPtr,
};

#[derive(Template)]
#[template(path = "search.html")]
struct SearchHtml {
    results: Vec<SearchResult>,
}

pub async fn index(ctx: State<AppPtr>) -> Result<impl IntoResponse> {
    Ok(ex!(SearchHtml {
        results: Vec::new(),
    }
    .render()))
}

#[derive(Deserialize)]
pub struct SearchForm {
    query: String,
}

pub async fn search(
    state: State<AppPtr>,
    Form(search): Form<SearchForm>,
) -> Result<impl IntoResponse> {
    let results = ex!(state.db.search(&search.query).await);
    Ok(ex!(SearchHtml { results }.render()))
}
