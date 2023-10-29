use actix::{Actor, Addr};
use actix_cors::Cors;
use actix_web::{middleware::Logger, web, App, HttpServer};
use api::summarize_route;
use summarizer::SummarizeActor;

mod api;
mod summarizer;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let summarize_actor_addr: Addr<SummarizeActor> = SummarizeActor::new().start();

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header(),
            )
            .app_data(web::Data::new(summarize_actor_addr.clone()))
            .service(web::scope("/api").service(summarize_route))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
