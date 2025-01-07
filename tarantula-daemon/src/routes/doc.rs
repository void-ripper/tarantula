use askama::Template;
use axum::response::{Html, IntoResponse};

#[derive(Template)]
#[template(path = "doc.html")]
pub struct DocHtml {}

pub async fn doc() -> impl IntoResponse {
    Html(DocHtml {}.render().unwrap())
}
