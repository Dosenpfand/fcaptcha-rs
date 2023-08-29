use std::env;

use actix_cors::Cors;
use actix_web::{
    get, http::StatusCode, post, web, App, Error, HttpResponse, HttpServer, Responder,
};
use fcaptcha::{
    is_puzzle_result_valid,
    web::{build_puzzle_service, verify_puzzle_result_service},
};
use log::info;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct FormInput {
    name: String,
    #[serde(rename = "frc-captcha-solution")]
    frc_captcha_solution: String,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    HttpServer::new(|| {
        let cors = Cors::permissive();
        App::new()
            .wrap(cors)
            .service(build_puzzle_service)
            .service(verify_puzzle_result_service)
            .service(demo_form)
            .service(index)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

#[get("/")]
async fn index() -> impl Responder {
    Ok::<HttpResponse, Error>(
        HttpResponse::build(StatusCode::OK)
            .content_type("text/html; charset=utf-8")
            .body(include_str!("../demo/index.html")),
    )
}

#[post("/demo-form")]
async fn demo_form(web::Form(input): web::Form<FormInput>) -> String {
    info!(
        "Got puzzle result verify request with name: {:?} and captcha solution: {:?}",
        input.name, input.frc_captcha_solution
    );

    // TODO: Instead of calling directly, post JSON over HTTP?
    let api_key: String = env::var("API_KEY").unwrap_or(String::from("NOT-AN-API-KEY"));
    let is_valid = is_puzzle_result_valid(&input.frc_captcha_solution, api_key.as_bytes());
    format!(
        "Got: {:?}, result for captcha validation:{:?}",
        input, is_valid
    )
}
