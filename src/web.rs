use actix_web::http::StatusCode;
use actix_web::{get, post, web, HttpRequest, Responder, Result};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::str;

use crate::build_puzzle::build_puzzle;
use crate::config::get;
use crate::verify_puzzle_result::is_puzzle_result_valid;

#[derive(Deserialize)]
pub struct BuildPuzzleServiceInput {
    sitekey: String,
}

#[derive(Serialize)]
struct BuildPuzzleServiceOutputData {
    puzzle: String,
}

#[derive(Serialize)]
struct BuildPuzzleServiceOutput {
    data: BuildPuzzleServiceOutputData,
}

impl BuildPuzzleServiceOutput {
    fn new(puzzle: String) -> BuildPuzzleServiceOutput {
        BuildPuzzleServiceOutput {
            data: BuildPuzzleServiceOutputData { puzzle },
        }
    }
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
    static ref API_KEY: String = get::<String>("API_KEY");
}

#[get("/build-puzzle")]
pub async fn build_puzzle_service(
    req: HttpRequest,
    input: web::Query<BuildPuzzleServiceInput>,
) -> Result<impl Responder> {
    if input.sitekey != *API_KEY {
        return Ok((
            web::Json(BuildPuzzleServiceOutput::new("".to_string())),
            StatusCode::FORBIDDEN,
        ));
    }
    let con_info = req.connection_info();
    let puzzle_result = build_puzzle(con_info.realip_remote_addr().unwrap());
    match puzzle_result {
        Ok(puzzle) => Ok((
            web::Json(BuildPuzzleServiceOutput::new(puzzle)),
            StatusCode::OK,
        )),
        Err(_) => Ok((
            // TODO: Propagate error information
            web::Json(BuildPuzzleServiceOutput::new("".to_string())),
            StatusCode::INTERNAL_SERVER_ERROR,
        )),
    }
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
