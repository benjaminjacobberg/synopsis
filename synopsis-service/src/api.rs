use actix::Addr;
use actix_web::{post, web, Responder};

use crate::summarizer::{Summarize, SummarizeActor};

#[post("/summarize")]
pub async fn summarize_route(
    actor: web::Data<Addr<SummarizeActor>>,
    params: web::Json<Summarize>,
) -> impl Responder {
    let result = actor.send(params.into_inner()).await;
    match result {
        Ok(res) => match res {
            Ok(r) => {
                return actix_web::HttpResponse::Ok().json(r);
            }
            Err(_) => {
                return actix_web::HttpResponse::InternalServerError().finish();
            }
        },
        Err(_) => actix_web::HttpResponse::InternalServerError().finish(),
    }
}
