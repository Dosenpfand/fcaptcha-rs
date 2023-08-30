use base64::{engine::general_purpose, Engine as _};
use blake2::{digest::consts::U32, Blake2b, Digest};
use hmac_sha256::HMAC;
use std::collections::{HashMap, HashSet};
use std::env;
use std::str;
use std::sync::Mutex;
use std::time::SystemTime;

lazy_static! {
    // TODO: Empty maps periodically!
    static ref VERIFIED_PUZZLE_TO_TIMESTAMP_MAP: Mutex<HashMap<Vec<u8>, u64>> =
        Mutex::new(HashMap::new());
    static ref API_KEY: String = env::var("API_KEY").unwrap_or(String::from("NOT-AN-API-KEY"));
    static ref PUZZLE_TTL: u64 = env::var("PUZZLE_TTL")
        .unwrap_or(String::from("3600"))
        .parse::<u64>()
        .unwrap();
    // TODO: Duplicated in web.rs
    static ref SECRET_KEY: String =
        env::var("SECRET_KEY").unwrap_or(String::from("NOT-A-SECRET-KEY"));
}

pub fn is_puzzle_result_valid(solution: &str, key: &[u8]) -> bool {
    if str::from_utf8(key).unwrap() != *API_KEY {
        return false;
    }

    let solution_parts: Vec<&str> = solution.splitn(4, '.').collect();
    let signature = solution_parts[0];
    let mut puzzle: [u8; 32] = [0; 32];

    info!("Trying to decode puzzle: {:?}", solution_parts[1]);
    // TODO: Switch back to checked version?
    // But needs 2 additional bytes: https://docs.rs/base64/latest/base64/fn.decoded_len_estimate.html
    general_purpose::STANDARD
        .decode_slice_unchecked(solution_parts[1], &mut puzzle)
        .unwrap();

    let calc_signature = hex::encode(HMAC::mac(puzzle, SECRET_KEY.as_str()));

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

    // TODO: Refactor, improve, maybe use entry()?
    {
        let mut map = VERIFIED_PUZZLE_TO_TIMESTAMP_MAP.lock().unwrap();

        let puzzle_option = map.get_mut(&puzzle.to_vec());

        match puzzle_option {
            Some(timestamp) => {
                if current_timestamp - *timestamp < *PUZZLE_TTL {
                    info!("Puzzle reuse with: {:?}", puzzle);
                    return false;
                } else {
                    info!("Expired puzzle reuse with: {:?}", puzzle);
                    *timestamp = current_timestamp;
                }
            }
            None => {
                info!("New puzzle with: {:?}", puzzle);
                map.insert(puzzle.to_vec(), current_timestamp);
            }
        }
    }

    let solutions_count = puzzle[14];
    let timestamp_received = u32::from_be_bytes(puzzle[0..4].try_into().unwrap());
    let age: u64 = current_timestamp - u64::from(timestamp_received);
    let expiry: u32 = u32::from(puzzle[13]) * 300;

    let solutions = general_purpose::STANDARD.decode(solution_parts[2]).unwrap();

    let _diagnostics = solution_parts[3];
    // TODO: log diagnostics

    if (expiry != 0) && (age > u64::from(expiry)) {
        info!("Expired puzzle, age: {:?}, expiry: {:?}", age, expiry);
        return false;
    }

    let difficulty = puzzle[15];
    // TODO: Why use floats?
    let threshold: u32 = (2_f32.powf(255.999 - f32::from(difficulty)) / 8_f32).floor() as u32;
    let mut seen_solutions = HashSet::<&[u8]>::new();

    for solution_idx in 0..solutions_count {
        let current_start_idx = usize::from(solution_idx);
        let current_solution = &solutions[current_start_idx..current_start_idx + 8];

        if seen_solutions.contains(current_solution) {
            info!("Duplicate solution found: {:?}", current_solution);
            return false;
        }
        seen_solutions.insert(current_solution);

        let mut full_solution: [u8; 128] = [0; 128];
        full_solution[0..32].copy_from_slice(&puzzle);
        full_solution[120..128].copy_from_slice(current_solution);
        info!("Full solution: {:?}", full_solution);

        type Blake2b256 = Blake2b<U32>;
        let hash = Blake2b256::digest(full_solution);
        info!("Solution hash: {:?}", full_solution);

        let solution_leading = u32::from_le_bytes(hash[0..4].try_into().unwrap());

        if solution_leading >= threshold {
            info!(
                "Found invalid solution not below threshold: {:?} >= {:?}",
                solution_leading, threshold
            );
            return false;
        }
        info!(
            "Found one valid solution below threshold: {:?} < {:?}",
            solution_leading, threshold
        );
    }

    info!("Puzzle solutions verified successfully for: {:?}", solution);
    true
}
