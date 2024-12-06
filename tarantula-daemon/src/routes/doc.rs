use askama::Template;

#[derive(Template)]
#[template(path = "doc.html")]
pub struct DocHtml {}
