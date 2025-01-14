use std::collections::HashMap;

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    routing::post,
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::error::Result;

use super::AppPtr;

#[derive(Deserialize)]
pub struct AddUrl {
    pub url: String,
}

pub async fn add_url(state: State<AppPtr>, Json(add): Json<AddUrl>) -> Result<impl IntoResponse> {
    state.db.add_url(add.url).await?;

    Ok(())
}

#[derive(Serialize)]
pub struct NextWork {
    url: String,
}

pub async fn next_work(
    state: State<AppPtr>,
    Path(pubkey): Path<String>,
) -> Result<impl IntoResponse> {
    let pubk = hex::decode(pubkey).unwrap();
    let mut pubkey = [0u8; 33];
    pubkey.copy_from_slice(&pubk);

    let work = state.db.get_next_work(pubkey).await?;

    Ok(Json(NextWork { url: work }))
}

#[derive(Deserialize)]
pub struct ScrapResult {
    url: String,
    keywords: HashMap<String, u32>,
    links: Vec<String>,
}

pub async fn scrap_result(
    state: State<AppPtr>,
    Json(result): Json<ScrapResult>,
) -> Result<impl IntoResponse> {
    state
        .db
        .scrap_result(result.url, result.keywords, result.links)
        .await?;
    Ok(())
}

pub fn config() -> Router<AppPtr> {
    let router = Router::new();

    router
        .route("/add-url", post(add_url))
        .route("/next-work/{pubkey}", post(next_work))
        .route("/scrap-result", post(scrap_result))
}
