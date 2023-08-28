#[macro_use]
extern crate lazy_static;

use actix_web::{get, App, HttpRequest, HttpResponse, HttpServer, Responder};
use base64::{engine::general_purpose, Engine as _};
use hmac_sha256::HMAC;
use std::collections::HashMap;
use std::env;
use std::sync::Mutex;
use std::time::SystemTime;

lazy_static! {
    static ref IP_ADDRESS_TO_ACCESS_COUNT_MAP: Mutex<HashMap<String, u64>> =
        Mutex::new(HashMap::new());
    static ref SECRET_KEY: String =
        env::var("SECRET_KEY").unwrap_or(String::from("NOT-A-SECRET-KEY"));
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(build_puzzle_service))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}

#[get("/build-puzzle")]
async fn build_puzzle_service(req: HttpRequest) -> impl Responder {
    let con_info = req.connection_info();
    let resp_text = build_puzzle(
        &SECRET_KEY.as_bytes(),
        con_info.realip_remote_addr().unwrap(),
    );
    HttpResponse::Ok().body(resp_text)
}

fn get_scaling(access_count: u64) -> (u8, u8) {
    if access_count > 20 {
        (45, 149)
    }
    // TODO: other vals
    else {
        (51, 122)
    }
}

fn build_puzzle(key: &[u8], ip_address: &str) -> String {
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let access_count = IP_ADDRESS_TO_ACCESS_COUNT_MAP
        .lock()
        .unwrap()
        .entry(ip_address.to_string())
        .and_modify(|count| *count += 1)
        .or_insert(1)
        .clone();

    let (count_solutions, difficulty) = get_scaling(access_count);
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
