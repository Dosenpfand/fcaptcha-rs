use actix_web::http::StatusCode;
use actix_web::{get, post, web, HttpRequest, Responder, Result};
use serde::{Deserialize, Serialize};
use std::str;

use crate::build_puzzle::build_puzzle;
use crate::config::get;
use crate::verify_puzzle_result::verify_puzzle_result;

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
    static ref API_KEY: Vec<u8> = get::<Vec<u8>>("API_KEY");
}

#[get("/build-puzzle")]
pub async fn build_puzzle_service(
    req: HttpRequest,
    input: web::Query<BuildPuzzleServiceInput>,
) -> Result<impl Responder> {
    let con_info = req.connection_info();
    let remote_address = con_info.realip_remote_addr();
    if (input.sitekey.as_bytes() != *API_KEY) || (remote_address.is_none()) {
        return Ok((
            web::Json(BuildPuzzleServiceOutput::new("".to_string())),
            StatusCode::FORBIDDEN,
        ));
    }

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
) -> Result<impl Responder> {
    if input.secret.as_bytes() != *API_KEY {
        return Ok((
            web::Json(VerifyPuzzleResultServiceOutput {
                success: false,
                errors: Some("secret_invalid".to_string()),
            }),
            StatusCode::FORBIDDEN,
        ));
    }

    info!("Got puzzle result verify request with {:?}", input);
    let puzzle_result = verify_puzzle_result(&input.solution);

    match puzzle_result {
        Ok(_) => Ok((
            web::Json(VerifyPuzzleResultServiceOutput {
                success: true,
                errors: None,
            }),
            StatusCode::OK,
        )),
        Err(_) => Ok((
            web::Json(VerifyPuzzleResultServiceOutput {
                success: false,
                errors: None, // TODO: Add description
            }),
            StatusCode::OK,
        )),
    }
}
