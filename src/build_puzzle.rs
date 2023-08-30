use base64::{engine::general_purpose, Engine as _};
use hmac_sha256::HMAC;
use std::collections::HashMap;
use std::str;
use std::sync::Mutex;
use std::time::SystemTime;
use crate::config::get;

#[derive(Clone, Debug)]
struct Access {
    count: u64,
    last_access: u64,
}

lazy_static! {
    // TODO: Empty maps periodically!
    static ref IP_ADDRESS_TO_ACCESS_MAP: Mutex<HashMap<String, Access>> =
        Mutex::new(HashMap::new());
    static ref ACCESS_TTL: u64 = get::<u64>("ACCESS_TTL");
}

// TODO: use proper types
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

pub fn build_puzzle(key: &[u8], ip_address: &str) -> String {
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
