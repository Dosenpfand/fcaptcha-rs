use actix_web::{get, post, web, HttpRequest, Responder};
use serde::{Deserialize, Serialize};
use std::env;
use std::error::Error;
use std::str;

use crate::build_puzzle::build_puzzle;
use crate::verify_puzzle_result::is_puzzle_result_valid;

#[derive(Serialize)]
struct BuildPuzzleServiceOutputData {
    puzzle: String,
}

#[derive(Serialize)]
struct BuildPuzzleServiceOutput {
    data: BuildPuzzleServiceOutputData,
}

#[derive(Deserialize, Debug)]
pub struct VerifyPuzzleResultServiceInput {
    solution: String,
    secret: String,
}

#[derive(Serialize)]
struct VerifyPuzzleResultServiceOutput {
    success: bool,
    errors: Option<String>,
}

lazy_static! {
    static ref SECRET_KEY: String =
        env::var("SECRET_KEY").unwrap_or(String::from("NOT-A-SECRET-KEY"));
}

#[get("/build-puzzle")]
pub async fn build_puzzle_service(req: HttpRequest) -> impl Responder {
    let con_info = req.connection_info();
    let resp_text = build_puzzle(
        SECRET_KEY.as_bytes(),
        con_info.realip_remote_addr().unwrap(),
    );
    Ok::<web::Json<BuildPuzzleServiceOutput>, Box<dyn Error>>(web::Json(BuildPuzzleServiceOutput {
        data: BuildPuzzleServiceOutputData { puzzle: resp_text },
    }))
}

#[post("/verify-puzzle-result")]
pub async fn verify_puzzle_result_service(
    input: web::Json<VerifyPuzzleResultServiceInput>,
) -> impl Responder {
    info!("Got puzzle result verify request with {:?}", input);

    let is_valid = is_puzzle_result_valid(&input.solution, input.secret.as_bytes());

    Ok::<web::Json<VerifyPuzzleResultServiceOutput>, Box<dyn Error>>(web::Json(
        VerifyPuzzleResultServiceOutput {
            success: is_valid,
            errors: None, // TODO: Expand error case
        },
    ))
}