use askama::Template;
use axum::response::{Html, IntoResponse};

#[derive(Template)]
#[template(path = "coc.html")]
pub struct CocHtml {}

pub async fn coc() -> impl IntoResponse {
    Html(CocHtml {}.render().unwrap())
}
