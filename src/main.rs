use actix_web::{App, HttpServer};
use fcaptcha_rs::web::{build_puzzle_service, verify_puzzle_result_service};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    HttpServer::new(|| {
        App::new()
            .service(build_puzzle_service)
            .service(verify_puzzle_result_service)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
