use askama::Template;
use axum::{routing::get, Router};

use crate::AppPtr;

mod index;
mod url;

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexHtml {}

pub fn config() -> Router<AppPtr> {
    Router::new()
        .route("/", get(index::index).post(index::search))
        .route("/add-url", get(url::add_url).post(url::add_url_to_db))
}
