use actix::Addr;
use actix_web::{post, web, Responder};

use crate::classification::{ClassificationActor, Classify};

#[post("/classify")]
pub async fn classify_route(
    actor: web::Data<Addr<ClassificationActor>>,
    params: web::Json<Classify>,
) -> impl Responder {
    println!("classify_route");
    let result = actor.send(params.into_inner()).await;
    println!("classify_route result: {:?}", result);
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
