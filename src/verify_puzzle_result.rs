use crate::config::get;
use crate::util;
use base64::DecodeError;
use base64::{engine::general_purpose, Engine as _};
use blake2::{digest::consts::U32, Blake2b, Digest};
use digest::{InvalidLength, MacError};
use hex::FromHexError;
use hmac::{Hmac, Mac};
use log::Level::Info;
use sha2::Sha256;
use std::collections::{HashMap, HashSet};
use std::str;
use std::sync::{Mutex, PoisonError};
use std::time::SystemTimeError;
use thiserror::Error;

lazy_static! {
    // TODO: Empty maps periodically!
    static ref VERIFIED_PUZZLE_TO_TIMESTAMP_MAP: Mutex<HashMap<Vec<u8>, u64>> =
        Mutex::new(HashMap::new());
    static ref PUZZLE_TTL: u64 = get::<u64>("PUZZLE_TTL");
    static ref SECRET_KEY: Vec<u8> = get::<Vec<u8>>("SECRET_KEY");
}

#[derive(Error, Debug, PartialEq)]
pub enum VerifyPuzzleResultError {
    #[error("Signature key invalid.")]
    SignatureKeyInvalid(#[from] InvalidLength),
    #[error("Signatures do not match.")]
    SignatureMismatch(#[from] MacError),
    #[error("Puzzle is reused.")]
    PuzzleReuse,
    #[error("Puzzle is expired.")]
    PuzzleExpired,
    #[error("Duplicate Solution.")]
    DuplicateSolution,
    #[error("Solution below threshold.")]
    SolutionBelowThreshold,
    #[error("Data access failed.")]
    DataAccess,
    #[error("Data conversion failed.")]
    Conversion,
    #[error("Decoding hex failed.")]
    DecodeHex(#[from] FromHexError),
    #[error("Decoding base64 failed.")]
    DecodeBas64(#[from] DecodeError),
    #[error("Failed to get the time.")]
    TimeError,
    #[error("Unknown error.")]
    Unknown,
}

impl<T> From<PoisonError<T>> for VerifyPuzzleResultError {
    fn from(_err: PoisonError<T>) -> Self {
        Self::DataAccess
    }
}

impl From<SystemTimeError> for VerifyPuzzleResultError {
    fn from(_err: SystemTimeError) -> Self {
        Self::TimeError
    }
}

pub fn verify_puzzle_result(solution: &str) -> Result<(), VerifyPuzzleResultError> {
    let timestamp = util::get_timestamp()?;
    verify_puzzle_result_with(solution, timestamp, *PUZZLE_TTL, &SECRET_KEY)
}

fn verify_signature(
    secret_key: &[u8],
    puzzle: &[u8],
    signature: &[u8],
) -> Result<(), VerifyPuzzleResultError> {
    type HmacSha256 = Hmac<Sha256>;
    let mut macer = HmacSha256::new_from_slice(secret_key)?;
    macer.update(puzzle);
    macer.verify_slice(signature)?;
    Ok(())
}

fn check_puzzle_reuse(
    puzzle: &[u8],
    puzzle_ttl: u64,
    current_timestamp: u64,
) -> Result<(), VerifyPuzzleResultError> {
    let mut map = VERIFIED_PUZZLE_TO_TIMESTAMP_MAP.lock()?;

    let puzzle_option = map.get_mut(&puzzle.to_vec());

    match puzzle_option {
        Some(timestamp) => {
            if current_timestamp - *timestamp < puzzle_ttl {
                info!("Puzzle reuse with: {:?}", puzzle);
                return Err(VerifyPuzzleResultError::PuzzleReuse);
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
    Ok(())
}

fn check_puzzle_expiry(
    puzzle: &[u8],
    current_timestamp: u64,
) -> Result<(), VerifyPuzzleResultError> {
    let timestamp_received = u32::from_be_bytes(
        puzzle[0..4]
            .try_into()
            .map_err(|_| VerifyPuzzleResultError::Conversion)?,
    );
    let age: u64 = current_timestamp - u64::from(timestamp_received);
    let expiry: u32 = u32::from(puzzle[13]) * 300;

    if (expiry != 0) && (age > u64::from(expiry)) {
        info!("Expired puzzle, age: {:?}, expiry: {:?}", age, expiry);
        return Err(VerifyPuzzleResultError::PuzzleExpired);
    }
    Ok(())
}

fn verify_solutions(puzzle: &[u8], solutions: &str) -> Result<(), VerifyPuzzleResultError> {
    let difficulty = puzzle[15];
    let solutions_count = puzzle[14];
    // TODO: Why use floats?
    let threshold: u32 = (2_f32.powf(255.999 - f32::from(difficulty)) / 8_f32).floor() as u32;
    let mut seen_solutions = HashSet::<&[u8]>::new();
    let solutions_decoded = general_purpose::STANDARD.decode(solutions)?;

    for solution_idx in 0..solutions_count {
        let current_start_idx = usize::from(solution_idx);
        let current_solution = &solutions_decoded[current_start_idx..current_start_idx + 8];

        if seen_solutions.contains(current_solution) {
            info!("Duplicate solution found: {:?}", current_solution);
            return Err(VerifyPuzzleResultError::DuplicateSolution);
        }
        seen_solutions.insert(current_solution);

        let mut full_solution: [u8; 128] = [0; 128];
        full_solution[0..32].copy_from_slice(puzzle);
        full_solution[120..128].copy_from_slice(current_solution);
        info!("Full solution: {:?}", full_solution);

        type Blake2b256 = Blake2b<U32>;
        let hash = Blake2b256::digest(full_solution);
        info!("Solution hash: {:?}", full_solution);

        let solution_leading = u32::from_le_bytes(
            hash[0..4]
                .try_into()
                .map_err(|_| VerifyPuzzleResultError::Conversion)?,
        );

        if solution_leading >= threshold {
            info!(
                "Found invalid solution not below threshold: {:?} >= {:?}",
                solution_leading, threshold
            );
            return Err(VerifyPuzzleResultError::SolutionBelowThreshold);
        }
        info!(
            "Found one valid solution below threshold: {:?} < {:?}",
            solution_leading, threshold
        );
    }
    Ok(())
}

fn process_diagnostics(diagnostics: &str) -> Result<(), VerifyPuzzleResultError> {
    if log_enabled!(Info) {
        let diagnostics = general_purpose::STANDARD.decode(diagnostics)?;
        info!("Got diagnostics: {:?}", diagnostics);
    }
    Ok(())
}

pub fn verify_puzzle_result_with(
    solution: &str,
    current_timestamp: u64,
    puzzle_ttl: u64,
    secret_key: &[u8],
) -> Result<(), VerifyPuzzleResultError> {
    let solution_parts: Vec<&str> = solution.splitn(4, '.').collect();
    let signature = hex::decode(solution_parts[0])?;
    let mut puzzle: [u8; 32] = [0; 32];

    info!("Trying to decode puzzle: {:?}", solution_parts[1]);
    // TODO: Switch back to checked version?
    // But needs 2 additional bytes: https://docs.rs/base64/latest/base64/fn.decoded_len_estimate.html
    general_purpose::STANDARD.decode_slice_unchecked(solution_parts[1], &mut puzzle)?;

    verify_signature(secret_key, &puzzle, &signature)?;
    check_puzzle_reuse(&puzzle, puzzle_ttl, current_timestamp)?;
    check_puzzle_expiry(&puzzle, current_timestamp)?;
    process_diagnostics(solution_parts[3])?;
    verify_solutions(&puzzle, solution_parts[2])?;

    info!("Puzzle solutions verified successfully for: {:?}", solution);
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_verify_puzzle_result_with_primitive_success() {
        let secret_key = "NOT-A-SECRET-KEY".as_bytes();
        let solution = "3761fae80ef01b32dcf892d099ca07f31db7a97311cce59529a4bae93a801db4.\
        ZO+cGAAAAAEAAAABAQwzegAAAAAAAAAAWlXMkohinFU=.\
        AAAAAIgRAAABAAAAzHwAAAIAAAAuDQAAAwAAAPsUAAAEAAAACaMAAAUAAADEGgAABgAAAEcSAAAHAAAAvz0AAAgAAABhpQ\
        AACQAAAAstAAAKAAAA2CYAAAsAAADtNgEADAAAAC0CAAANAAAAFp8AAA4AAABdcgAADwAAAL6JAAAQAAAALYkAABEAAAD0\
        vAEAEgAAAPxaAAATAAAAvFAAABQAAAAA7wEAFQAAAPoWAAAWAAAAGoEAABcAAACovwAAGAAAAGXcAAAZAAAAP2sBABoAAA\
        D4BQAAGwAAAE9nAAAcAAAAFcQBAB0AAABQCgEAHgAAAB0FAAAfAAAAe9EAACAAAAClywAAIQAAAFYPAAAiAAAAtjcAACMA\
        AABIgQAAJAAAAJoPAQAlAAAAYlgAACYAAABIbAAAJwAAAGCwAAAoAAAAokkAACkAAADl6gAAKgAAAAo5AQArAAAA5igAAC\
        wAAADVfAAALQAAAHYfAAAuAAAALdYAAC8AAAC11gEAMAAAAN1dAAAxAAAAbyEAADIAAADjwAAA.\
        AgAA";
        let timestamp: u64 = 1693424664;

        let result = verify_puzzle_result_with(solution, timestamp, 0, &secret_key);
        assert!(result.is_ok())
    }

    #[test]
    fn test_verify_puzzle_result_with_primitive_signature_error() {
        let secret_key = "THE-WRONG-SECRET-KEY".as_bytes();
        let solution = "3761fae80ef01b32dcf892d099ca07f31db7a97311cce59529a4bae93a801db4.\
        ZO+cGAAAAAEAAAABAQwzegAAAAAAAAAAWlXMkohinFU=.\
        AAAAAIgRAAABAAAAzHwAAAIAAAAuDQAAAwAAAPsUAAAEAAAACaMAAAUAAADEGgAABgAAAEcSAAAHAAAAvz0AAAgAAABhpQ\
        AACQAAAAstAAAKAAAA2CYAAAsAAADtNgEADAAAAC0CAAANAAAAFp8AAA4AAABdcgAADwAAAL6JAAAQAAAALYkAABEAAAD0\
        vAEAEgAAAPxaAAATAAAAvFAAABQAAAAA7wEAFQAAAPoWAAAWAAAAGoEAABcAAACovwAAGAAAAGXcAAAZAAAAP2sBABoAAA\
        D4BQAAGwAAAE9nAAAcAAAAFcQBAB0AAABQCgEAHgAAAB0FAAAfAAAAe9EAACAAAAClywAAIQAAAFYPAAAiAAAAtjcAACMA\
        AABIgQAAJAAAAJoPAQAlAAAAYlgAACYAAABIbAAAJwAAAGCwAAAoAAAAokkAACkAAADl6gAAKgAAAAo5AQArAAAA5igAAC\
        wAAADVfAAALQAAAHYfAAAuAAAALdYAAC8AAAC11gEAMAAAAN1dAAAxAAAAbyEAADIAAADjwAAA.\
        AgAA";
        let timestamp: u64 = 1693424664;

        let result = verify_puzzle_result_with(solution, timestamp, 0, &secret_key);
        assert_eq!(
            result,
            Err(VerifyPuzzleResultError::SignatureMismatch(MacError))
        )
    }
}
