use actix::{Actor, Addr};
use actix_files as fs;
use actix_cors::Cors;
use actix_web::{middleware::Logger, web, App, HttpServer};
use api::classify_route;
use classification::ClassificationActor;

mod api;
mod classification;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let classification_actor_addr: Addr<ClassificationActor> = ClassificationActor::new().start();

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header(),
            )
            .app_data(web::Data::new(classification_actor_addr.clone()))
            .service(web::scope("/api").service(classify_route))
            .service(fs::Files::new("/", "static").index_file("index.html"))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
