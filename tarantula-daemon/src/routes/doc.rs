use askama::Template;
use axum::response::IntoResponse;

#[derive(Template)]
#[template(path = "doc.html")]
pub struct DocHtml {}

pub async fn doc() -> impl IntoResponse {
    DocHtml {}
}
