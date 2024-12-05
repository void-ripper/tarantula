use askama::Template;
use axum::response::IntoResponse;

use crate::error::Result;

#[derive(Template)]
#[template(path = "faq.html")]
struct FaqHtml {
    faqs: Vec<(String, String)>,
}

pub async fn faq() -> Result<impl IntoResponse> {
    let secp =
        r#"<a class="has-text-info" href="https://en.bitcoin.it/wiki/Secp256k1">Secp256k1</a>"#;

    let faqs = vec![
        (
            "Why does this site reload at every interaction?".to_owned(),
            "Because this site works entierly without JavaScript!".to_owned(),
        ),
        (
            "Why no JavaScript?".to_owned(),
            "To make it impossible to inject anykind of code via third party software.".to_owned(),
        ),
        (
            "Why do i have to download this cryptic key?".to_owned(),
            format!(
                "Because this is your private key. We use {} Public and Private key encryption.",
                secp
            ),
        ),
        (
            "Are you storing my private key?".to_owned(),
            "No we are not storing your private key.".to_owned(),
        ),
        (
            "I have lost my private key. What can i do?".into(),
            r#"Nothing! Your account is <b>LOST</b>!
            <br>
            I mean like in lost, lost. There is no way we are able to fix that."#
                .into(),
        ),
    ];

    Ok(FaqHtml { faqs })
}
