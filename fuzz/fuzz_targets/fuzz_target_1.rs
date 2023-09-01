#![no_main]

use fcaptcha::verify_puzzle_result_with;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &str| {
    // TODO: Can not proceed further then verify_signature()
    let _ = verify_puzzle_result_with(data, 0, 0, "".as_bytes());
});
