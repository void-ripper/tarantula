use askama::Template;
use axum::response::IntoResponse;

#[derive(Template)]
#[template(path = "coc.html")]
pub struct CocHtml {}

pub async fn coc() -> impl IntoResponse {
    CocHtml {}
}
