#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;

use actix_web::{get, web, App, HttpRequest, HttpServer, Responder};
use base64::{engine::general_purpose, Engine as _};
use hmac_sha256::HMAC;
use serde::Serialize;
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::str;
use std::sync::Mutex;
use std::time::SystemTime;

#[derive(Clone, Debug)]
struct Access {
    count: u64,
    last_access: u64,
}

#[derive(Serialize)]
struct BuildPuzzleServiceResultData {
    puzzle: String,
}

#[derive(Serialize)]
struct BuildPuzzleServiceResult {
    data: BuildPuzzleServiceResultData,
}

lazy_static! {
    static ref IP_ADDRESS_TO_ACCESS_MAP: Mutex<HashMap<String, Access>> =
        Mutex::new(HashMap::new());
    static ref VERIFIED_PUZZLE_TO_TIMESTAMP_MAP: Mutex<HashMap<String, u64>> =
        Mutex::new(HashMap::new());
    static ref SECRET_KEY: String =
        env::var("SECRET_KEY").unwrap_or(String::from("NOT-A-SECRET-KEY"));
    static ref ACCESS_TTL: u64 = env::var("ACCESS_TTL")
        .unwrap_or(String::from("1800"))
        .parse::<u64>()
        .unwrap();
    static ref PUZZLE_TTL: u64 = env::var("PUZZLE_TTL")
        .unwrap_or(String::from("3600"))
        .parse::<u64>()
        .unwrap();
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    HttpServer::new(|| App::new().service(build_puzzle_service))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}

#[get("/build-puzzle")]
async fn build_puzzle_service(req: HttpRequest) -> impl Responder {
    let con_info = req.connection_info();
    let resp_text = build_puzzle(
        SECRET_KEY.as_bytes(),
        con_info.realip_remote_addr().unwrap(),
    );
    Ok::<web::Json<BuildPuzzleServiceResult>, Box<dyn Error>>(web::Json(BuildPuzzleServiceResult {
        data: BuildPuzzleServiceResultData { puzzle: resp_text },
    }))
}

fn get_scaling(access_count: u64) -> (u8, u8) {
    if access_count > 20 {
        (45, 149)
    } else if access_count > 10 {
        (45, 141)
    } else if access_count > 4 {
        (51, 130)
    } else {
        (51, 122)
    }
}

fn build_puzzle(key: &[u8], ip_address: &str) -> String {
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let access = IP_ADDRESS_TO_ACCESS_MAP
        .lock()
        .unwrap()
        .entry(ip_address.to_string())
        .and_modify(|access| {
            if timestamp - access.last_access > *ACCESS_TTL {
                access.count = 1;
            } else {
                access.count += 1;
            }
            access.last_access = timestamp;
        })
        .or_insert(Access {
            count: 1,
            last_access: timestamp,
        })
        .clone();

    let (count_solutions, difficulty) = get_scaling(access.count);

    info!("Creating puzzle for ip_address: {:?}, timestamp: {:?}, access: {:?}, count_solutions: {:?}, difficulty: {:?}",
        ip_address, timestamp, access, count_solutions, difficulty);

    let timestamp_truncated: u32 = timestamp.try_into().unwrap();
    let account_id: u32 = 1;
    let app_id: u32 = 1;
    let puzzle_ver: u8 = 1;
    let puzzle_expiry: u8 = 12;
    let nonce: u64 = rand::random();

    // TODO: Optimize
    let mut data: [u8; 32] = [0; 32];
    data[0..][..4].copy_from_slice(&timestamp_truncated.to_be_bytes());
    data[4..][..4].copy_from_slice(&account_id.to_be_bytes());
    data[8..][..4].copy_from_slice(&app_id.to_be_bytes());
    data[12] = puzzle_ver;
    data[13] = puzzle_expiry;
    data[14] = count_solutions;
    data[15] = difficulty;
    // [16..][..4] = Reserved: zero
    data[24..][..8].copy_from_slice(&nonce.to_be_bytes());

    // HMAC data
    let hmac = HMAC::mac(data, key);

    // Base 64 encode data
    let mut data_b64: [u8; 44] = [0; 44];
    let data_b64_len = general_purpose::STANDARD
        .encode_slice(data, &mut data_b64)
        .unwrap();
    debug_assert!(data_b64_len == 44);

    // Concatenate HMAC and data
    // TODO: Optimize?
    let puzzle = format!(
        "{}.{}",
        hex::encode(hmac),
        String::from_utf8_lossy(&data_b64)
    );

    puzzle
}

fn is_puzzle_valid(solution: &str, key: &[u8]) -> bool {
    let solution_parts: Vec<&str> = solution.splitn(4, ".").collect();
    let signature = solution_parts[0];
    let mut puzzle: [u8; 32] = [0; 32];
    general_purpose::STANDARD
        .decode_slice(solution_parts[1], &mut puzzle)
        .unwrap();

    let calc_signature = hex::encode(HMAC::mac(puzzle, key));

    if calc_signature != signature {
        info!(
            "Signature mismatch, received: {:?}, calculated: {:?}",
            signature, calc_signature
        );
        return false;
    }

    let current_timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let puzzle_option = VERIFIED_PUZZLE_TO_TIMESTAMP_MAP
        .lock()
        .unwrap()
        .get(str::from_utf8(&puzzle).unwrap());

    // TODO: here
    // match puzzle_option {
    //     Some(timestamp) => {
    //         if current_timestamp - timestamp < *PUZZLE_TTL {
    //             info!("Puzzle reuse with {:?}", puzzle);
    //             return false
    //         }
    //     },
    //     None => todo!(),
    // }

    let solutions = solution_parts[2];
    let diagnostics = solution_parts[3];

    true
}
