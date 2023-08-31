use crate::config::get;
use crate::util;
use base64::{engine::general_purpose, Engine as _};
use blake2::{digest::consts::U32, Blake2b, Digest};
use hmac::{Hmac, Mac};
use log::Level::Info;
use sha2::Sha256;
use std::collections::{HashMap, HashSet};
use std::str;
use std::sync::Mutex;

lazy_static! {
    // TODO: Empty maps periodically!
    static ref VERIFIED_PUZZLE_TO_TIMESTAMP_MAP: Mutex<HashMap<Vec<u8>, u64>> =
        Mutex::new(HashMap::new());
    static ref API_KEY: String = get::<String>("API_KEY");
    static ref PUZZLE_TTL: u64 = get::<u64>("PUZZLE_TTL");
    static ref SECRET_KEY: String = get::<String>("SECRET_KEY");
}

pub fn is_puzzle_result_valid(solution: &str, key: &[u8]) -> bool {
    is_puzzle_result_valid_with_ttl_and_timestamp(solution, key, *PUZZLE_TTL, util::get_timestamp())
}

pub fn is_puzzle_result_valid_with_ttl_and_timestamp(
    solution: &str,
    key: &[u8],
    puzzle_ttl: u64,
    current_timestamp: u64,
) -> bool {
    if str::from_utf8(key).unwrap() != *API_KEY {
        return false;
    }

    let solution_parts: Vec<&str> = solution.splitn(4, '.').collect();
    let signature = hex::decode(solution_parts[0]).unwrap();
    let mut puzzle: [u8; 32] = [0; 32];

    info!("Trying to decode puzzle: {:?}", solution_parts[1]);
    // TODO: Switch back to checked version?
    // But needs 2 additional bytes: https://docs.rs/base64/latest/base64/fn.decoded_len_estimate.html
    general_purpose::STANDARD
        .decode_slice_unchecked(solution_parts[1], &mut puzzle)
        .unwrap();

    type HmacSha256 = Hmac<Sha256>;
    let mut macer = HmacSha256::new_from_slice(SECRET_KEY.as_bytes()).unwrap();
    macer.update(&puzzle);
    let verify_result = macer.verify_slice(&signature);

    match verify_result {
        Ok(_) => {
            info!("Signature match");
        }
        Err(_) => {
            info!("Signature mismatch");
            return false;
        }
    }

    {
        let mut map = VERIFIED_PUZZLE_TO_TIMESTAMP_MAP.lock().unwrap();

        let puzzle_option = map.get_mut(&puzzle.to_vec());

        match puzzle_option {
            Some(timestamp) => {
                if current_timestamp - *timestamp < puzzle_ttl {
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

    if log_enabled!(Info) {
        let diagnostics = general_purpose::STANDARD.decode(solution_parts[3]).unwrap();
        info!("Got diagnostics: {:?}", diagnostics);
    }

    let timestamp_received = u32::from_be_bytes(puzzle[0..4].try_into().unwrap());
    let age: u64 = current_timestamp - u64::from(timestamp_received);
    let expiry: u32 = u32::from(puzzle[13]) * 300;

    if (expiry != 0) && (age > u64::from(expiry)) {
        info!("Expired puzzle, age: {:?}, expiry: {:?}", age, expiry);
        return false;
    }

    let difficulty = puzzle[15];
    let solutions_count = puzzle[14];
    // TODO: Why use floats?
    let threshold: u32 = (2_f32.powf(255.999 - f32::from(difficulty)) / 8_f32).floor() as u32;
    let mut seen_solutions = HashSet::<&[u8]>::new();
    let solutions = general_purpose::STANDARD.decode(solution_parts[2]).unwrap();
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
