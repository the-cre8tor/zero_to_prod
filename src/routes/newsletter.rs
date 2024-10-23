use actix_web::{web::Json, HttpResponse};

#[derive(serde::Deserialize)]
pub struct BodyData {
    title: String,
    content: Content,
}

#[derive(serde::Deserialize)]
pub struct Content {
    html: String,
    text: String,
}

pub async fn publish_newsletter(_body: Json<BodyData>) -> HttpResponse {
    HttpResponse::Ok().finish()
}
