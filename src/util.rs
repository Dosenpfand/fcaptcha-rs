use std::time::{SystemTime, SystemTimeError};

pub fn get_timestamp() -> Result<u64, SystemTimeError> {
    let duration_result = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH);

    match duration_result {
        Ok(duration) => Ok(duration.as_secs()),
        Err(val) => Err(val),
    }
}
