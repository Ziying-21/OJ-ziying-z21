use actix_web::{Responder, HttpResponse, options};

#[options("/jobs")]
pub async fn options() -> impl Responder {
    HttpResponse::Ok()
}