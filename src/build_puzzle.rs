use crate::config::get;
use crate::util;
use base64::{engine::general_purpose, Engine as _};
use hmac_sha256::HMAC;
use std::collections::HashMap;
use std::str;
use std::sync::Mutex;

lazy_static! {
    // TODO: Empty maps periodically!
    static ref IP_ADDRESS_TO_ACCESS_MAP: Mutex<HashMap<String, Access>> =
        Mutex::new(HashMap::new());
    static ref ACCESS_TTL: u64 = get::<u64>("ACCESS_TTL");
}
#[derive(Clone, Debug)]
struct Access {
    count: u64,
    last_access: u64,
}

impl Access {
    fn get(ip_address: &str, timestamp: u64) -> Access {
        IP_ADDRESS_TO_ACCESS_MAP
            .lock()
            .unwrap()
            .entry(ip_address.to_string())
            .and_modify(|access| {
                println!("now: {:?}, last: {:?}", timestamp, access.last_access);
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
            .clone()
    }
}

#[derive(Debug)]
struct Scaling {
    solution_count: u8,
    difficulty: u8,
}

impl Scaling {
    fn new(solution_count: u8, difficulty: u8) -> Scaling {
        Scaling {
            solution_count,
            difficulty,
        }
    }

    fn get(access_count: u64) -> Scaling {
        if access_count > 20 {
            Scaling::new(45, 149)
        } else if access_count > 10 {
            Scaling::new(45, 141)
        } else if access_count > 4 {
            Scaling::new(51, 130)
        } else {
            Scaling::new(51, 122)
        }
    }
}

fn construct_puzzle_data(timestamp: u64, scaling: Scaling, data_buffer: &mut [u8]) {
    let timestamp_truncated: u32 = timestamp.try_into().unwrap();
    // TODO: Make configurable
    let account_id: u32 = 1;
    let app_id: u32 = 1;
    let puzzle_ver: u8 = 1;
    let puzzle_expiry: u8 = 12;
    let nonce: u64 = rand::random();

    data_buffer[0..][..4].copy_from_slice(&timestamp_truncated.to_be_bytes());
    data_buffer[4..][..4].copy_from_slice(&account_id.to_be_bytes());
    data_buffer[8..][..4].copy_from_slice(&app_id.to_be_bytes());
    data_buffer[12] = puzzle_ver;
    data_buffer[13] = puzzle_expiry;
    data_buffer[14] = scaling.solution_count;
    data_buffer[15] = scaling.difficulty;
    // [16..][..4] = Reserved: zero
    data_buffer[24..][..8].copy_from_slice(&nonce.to_be_bytes());
}

pub fn build_puzzle(key: &[u8], ip_address: &str) -> String {
    let timestamp = util::get_timestamp();

    let access = Access::get(ip_address, timestamp);
    let scaling = Scaling::get(access.count);

    info!(
        "Creating puzzle for ip_address: {:?}, timestamp: {:?}, access: {:?}, scaling: {:?}",
        ip_address, timestamp, access, scaling
    );

    let mut puzzle_data: [u8; 32] = [0; 32];
    construct_puzzle_data(timestamp, scaling, &mut puzzle_data);

    // HMAC data
    let hmac = HMAC::mac(puzzle_data, key);

    // Base 64 encode data
    let mut data_b64: [u8; 44] = [0; 44];
    let data_b64_len = general_purpose::STANDARD
        .encode_slice(puzzle_data, &mut data_b64)
        .unwrap();
    debug_assert!(data_b64_len == 44);

    // Concatenate HMAC and data
    let puzzle = format!(
        "{}.{}",
        hex::encode(hmac),
        String::from_utf8_lossy(&data_b64)
    );

    puzzle
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_access_first() {
        let ip_address: &str = "192.168.0.1";
        let timestamp = 1234_u64;
        let access = Access::get(ip_address, timestamp);
        assert!(access.count == 1);
        assert!(access.last_access == timestamp);
    }

    #[test]
    fn test_get_access_second() {
        let ip_address = "192.168.0.2";
        let timestamp: u64 = 1234_u64;
        Access::get(ip_address, timestamp);
        let timestamp: u64 = 1235_u64;
        let access = Access::get(ip_address, timestamp);
        assert_eq!(access.count, 2);
        assert_eq!(access.last_access, timestamp);
    }

    #[test]
    fn test_get_access_second_within_ttl() {
        let ip_address = "192.168.0.3";
        let timestamp: u64 = 1234_u64;
        Access::get(ip_address, timestamp);
        let timestamp = 1234_u64 + *ACCESS_TTL;
        let access = Access::get(ip_address, timestamp);
        assert_eq!(access.count, 2);
        assert_eq!(access.last_access, timestamp);
    }

    #[test]
    fn test_get_access_second_after_ttl() {
        let ip_address = "192.168.0.4";
        let timestamp: u64 = 1234_u64;
        Access::get(ip_address, timestamp);
        let timestamp = 1234_u64 + *ACCESS_TTL + 1;
        let access = Access::get(ip_address, timestamp);
        assert_eq!(access.count, 1);
        assert_eq!(access.last_access, timestamp);
    }
}
