use crate::config::get;
use crate::util;
use base64::EncodeSliceError;
use base64::{engine::general_purpose, Engine as _};
use blake2::digest::InvalidLength;
use displaydoc::Display;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::collections::HashMap;
use std::str;
use std::sync::{Mutex, PoisonError};
use std::time::SystemTimeError;
use thiserror::Error;

lazy_static! {
    // TODO: Empty maps periodically!
    static ref IP_ADDRESS_TO_ACCESS_MAP: Mutex<HashMap<String, Access>> =
        Mutex::new(HashMap::new());
    static ref ACCESS_TTL: u64 = get::<u64>("ACCESS_TTL");
    static ref SECRET_KEY: Vec<u8> = get::<Vec<u8>>("SECRET_KEY");
}

/// Describes an error that occurred during building a puzzle.
#[derive(Display, Error, Debug)]
pub enum BuildPuzzleError {
    /// Encoding failed.
    Encoding(#[from] EncodeSliceError),
    /// Hashing failed.
    Hashing(#[from] InvalidLength),
    /// Data access failed.
    DataAccess,
    /// Data conversion failed.
    Conversion,
    /// Failed to get the time.
    TimeError(#[from] SystemTimeError),
    /// Unknown error.
    Unknown,
}

impl<T> From<PoisonError<T>> for BuildPuzzleError {
    fn from(_err: PoisonError<T>) -> Self {
        Self::DataAccess
    }
}

#[derive(Clone, Debug)]
struct Access {
    count: u64,
    last_access: u64,
}

impl Access {
    fn get(ip_address: &str, timestamp: u64, access_ttl: u64) -> Result<Access, BuildPuzzleError> {
        let mut lock = IP_ADDRESS_TO_ACCESS_MAP.lock()?;
        Ok(lock
            .entry(ip_address.to_string())
            .and_modify(|access| {
                if timestamp - access.last_access > access_ttl {
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
            .clone())
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

fn construct_puzzle_data(
    timestamp: u64,
    nonce: u64,
    scaling: Scaling,
    data_buffer: &mut [u8],
) -> Result<(), BuildPuzzleError> {
    let timestamp_truncated: u32 = timestamp
        .try_into()
        .map_err(|_| BuildPuzzleError::Conversion)?;
    // TODO: Make configurable
    let account_id: u32 = 1;
    let app_id: u32 = 1;
    let puzzle_ver: u8 = 1;
    let puzzle_expiry: u8 = 12;

    data_buffer[0..][..4].copy_from_slice(&timestamp_truncated.to_be_bytes());
    data_buffer[4..][..4].copy_from_slice(&account_id.to_be_bytes());
    data_buffer[8..][..4].copy_from_slice(&app_id.to_be_bytes());
    data_buffer[12] = puzzle_ver;
    data_buffer[13] = puzzle_expiry;
    data_buffer[14] = scaling.solution_count;
    data_buffer[15] = scaling.difficulty;
    // [16..][..4] = Reserved: zero
    data_buffer[24..][..8].copy_from_slice(&nonce.to_be_bytes());
    Ok(())
}

/// Builds a new puzzle for an `ip_address`.
/// Can be configured with the environment variables `FCAPTCHA_ACCESS_TTL` and `FCAPTCHA_SECRET_KEY`.
///
/// # Examples
///
/// ```
/// let ip_address = "127.0.0.1";
/// let puzzle = fcaptcha::build_puzzle(ip_address);
///
/// println!("{:?}", puzzle.unwrap());
/// ```
pub fn build_puzzle(ip_address: &str) -> Result<String, BuildPuzzleError> {
    let timestamp = util::get_timestamp()?;
    let nonce: u64 = rand::random();
    build_puzzle_with(ip_address, timestamp, nonce, &SECRET_KEY, *ACCESS_TTL)
}

/// Builds a new puzzle. In contrast to [build_puzzle] all input variables can be controlled
/// directly instead deriving them from environment variables.
///
/// # Examples
///
/// ```
/// use std::time::SystemTime;
/// let ip_address = "127.0.0.1";
/// let timestamp = SystemTime::now()
///     .duration_since(SystemTime::UNIX_EPOCH)
///     .unwrap()
///     .as_secs();
/// let secret_key = "SECRET-KEY".as_bytes();
/// let nonce = rand::random();
/// let access_ttl_secs = 1800;
/// let puzzle =
///      fcaptcha::build_puzzle_with(ip_address, timestamp, nonce, secret_key, access_ttl_secs);
/// println!("{:?}", puzzle.unwrap());
/// ```
pub fn build_puzzle_with(
    ip_address: &str,
    timestamp: u64,
    nonce: u64,
    secret_key: &[u8],
    access_ttl_secs: u64,
) -> Result<String, BuildPuzzleError> {
    let access = Access::get(ip_address, timestamp, access_ttl_secs)?;
    let scaling = Scaling::get(access.count);

    info!(
        "Creating puzzle for ip_address: {:?}, timestamp: {:?}, access: {:?}, scaling: {:?}",
        ip_address, timestamp, access, scaling
    );

    let mut puzzle_data: [u8; 32] = [0; 32];
    construct_puzzle_data(timestamp, nonce, scaling, &mut puzzle_data)?;

    // HMAC data
    type HmacSha256 = Hmac<Sha256>;
    let mut macer = HmacSha256::new_from_slice(secret_key)?;
    macer.update(&puzzle_data);
    let hmac = macer.finalize().into_bytes();

    // Base 64 encode data
    let mut data_b64: [u8; 44] = [0; 44];
    let data_b64_len = general_purpose::STANDARD.encode_slice(puzzle_data, &mut data_b64)?;
    debug_assert!(data_b64_len == 44);

    // Concatenate HMAC and data
    let puzzle = format!(
        "{}.{}",
        hex::encode(hmac),
        String::from_utf8_lossy(&data_b64)
    );

    Ok(puzzle)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_puzzle_with_timestamp_and_nonce() -> Result<(), BuildPuzzleError> {
        let secret_key = "TEST-KEY".as_bytes();
        let ip_address = "127.0.0.1";
        let timestamp = 1693469848;
        let nonce = 0x1122334455667788;
        let expected_puzzle = "86505156a95e735652e7fd6d9eaaa9e5f839fc0a886268bebf5b8d2ad1038df5.\
        ZPBMmAAAAAEAAAABAQwzegAAAAAAAAAAESIzRFVmd4g=";
        let access_ttl = 1800;

        let puzzle = build_puzzle_with(ip_address, timestamp, nonce, secret_key, access_ttl)?;

        assert_eq!(expected_puzzle, puzzle);
        Ok(())
    }

    #[test]
    fn test_get_access_first() -> Result<(), BuildPuzzleError> {
        let ip_address: &str = "192.168.0.1";
        let timestamp = 1234_u64;
        let access_ttl = 1800;
        let access = Access::get(ip_address, timestamp, access_ttl)?;
        assert!(access.count == 1);
        assert!(access.last_access == timestamp);
        Ok(())
    }

    #[test]
    fn test_get_access_second() -> Result<(), BuildPuzzleError> {
        let ip_address = "192.168.0.2";
        let timestamp: u64 = 1234_u64;
        let access_ttl = 1800;
        Access::get(ip_address, timestamp, access_ttl)?;
        let timestamp: u64 = 1235_u64;
        let access = Access::get(ip_address, timestamp, access_ttl)?;
        assert_eq!(access.count, 2);
        assert_eq!(access.last_access, timestamp);
        Ok(())
    }

    #[test]
    fn test_get_access_second_within_ttl() -> Result<(), BuildPuzzleError> {
        let ip_address = "192.168.0.3";
        let timestamp: u64 = 1234_u64;
        let access_ttl = 1800;
        Access::get(ip_address, timestamp, access_ttl)?;
        let timestamp = 1234_u64 + *ACCESS_TTL;
        let access = Access::get(ip_address, timestamp, access_ttl)?;
        assert_eq!(access.count, 2);
        assert_eq!(access.last_access, timestamp);
        Ok(())
    }

    #[test]
    fn test_get_access_second_after_ttl() -> Result<(), BuildPuzzleError> {
        let ip_address = "192.168.0.4";
        let timestamp: u64 = 1234_u64;
        let access_ttl = 1800;
        Access::get(ip_address, timestamp, access_ttl)?;
        let timestamp = 1234_u64 + *ACCESS_TTL + 1;
        let access = Access::get(ip_address, timestamp, access_ttl)?;
        assert_eq!(access.count, 1);
        assert_eq!(access.last_access, timestamp);
        Ok(())
    }
}
