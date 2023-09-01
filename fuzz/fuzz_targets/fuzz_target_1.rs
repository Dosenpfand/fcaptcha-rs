#![no_main]

use libfuzzer_sys::fuzz_target;
use fcaptcha::verify_puzzle_result;

fuzz_target!(|data: &str| {
    verify_puzzle_result(data);
});
