use actix_cors::Cors;
use actix_web::{App, HttpServer};
use fcaptcha::config::get;
use fcaptcha::web::{build_puzzle_service, verify_puzzle_result_service};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    HttpServer::new(|| {
        // TODO: Switch to non-permissive
        let cors = Cors::permissive();
        App::new()
            .wrap(cors)
            .service(build_puzzle_service)
            .service(verify_puzzle_result_service)
    })
    .bind((get::<String>("BIND_ADDRESS"), get::<u16>("BIND_PORT")))?
    .run()
    .await
}
