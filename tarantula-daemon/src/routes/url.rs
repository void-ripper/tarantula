use crate::error::Error;
use askama::Template;
use axum::{
    extract::State,
    response::{Html, IntoResponse},
    Form,
};
use serde::Deserialize;

use crate::{error::Result, ex, AppPtr};

#[derive(Template)]
#[template(path = "url.html")]
struct UrlHtml {
    error: String,
}

pub async fn add_url() -> Result<impl IntoResponse> {
    Ok(Html(ex!(UrlHtml {
        error: "".to_owned(),
    }
    .render())))
}

#[derive(Deserialize)]
pub struct AddUrlForm {
    url: String,
}

pub async fn add_url_to_db(
    state: State<AppPtr>,
    Form(add): Form<AddUrlForm>,
) -> Result<impl IntoResponse> {
    state.db.add_url(add.url).await?;

    Ok(Html(ex!(UrlHtml {
        error: "".to_owned(),
    }
    .render())))
}
