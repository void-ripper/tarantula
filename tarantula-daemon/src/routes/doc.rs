use askama::Template;
use axum::response::{Html, IntoResponse};

// for syntax highlighting
// https://highlight.hohli.com/?theme=nord
//
#[derive(Template)]
#[template(path = "doc.html")]
pub struct DocHtml {}

pub async fn doc() -> impl IntoResponse {
    Html(DocHtml {}.render().unwrap())
}
