use fcaptcha::{build_puzzle, get, is_puzzle_result_valid};

fn main() {
    env_logger::init();
    let api_key = get::<String>("API_KEY");
    let puzzle = build_puzzle("127.0.0.1");
    println!("Generated puzzle: {:?}", puzzle);

    let solution = "3761fae80ef01b32dcf892d099ca07f31db7a97311cce59529a4bae93a801db4.\
    ZO+cGAAAAAEAAAABAQwzegAAAAAAAAAAWlXMkohinFU=.\
    AAAAAIgRAAABAAAAzHwAAAIAAAAuDQAAAwAAAPsUAAAEAAAACaMAAAUAAADEGgAABgAAAEcSAAAHAAAAvz0AAAgAAABhpQ\
    AACQAAAAstAAAKAAAA2CYAAAsAAADtNgEADAAAAC0CAAANAAAAFp8AAA4AAABdcgAADwAAAL6JAAAQAAAALYkAABEAAAD0\
    vAEAEgAAAPxaAAATAAAAvFAAABQAAAAA7wEAFQAAAPoWAAAWAAAAGoEAABcAAACovwAAGAAAAGXcAAAZAAAAP2sBABoAAA\
    D4BQAAGwAAAE9nAAAcAAAAFcQBAB0AAABQCgEAHgAAAB0FAAAfAAAAe9EAACAAAAClywAAIQAAAFYPAAAiAAAAtjcAACMA\
    AABIgQAAJAAAAJoPAQAlAAAAYlgAACYAAABIbAAAJwAAAGCwAAAoAAAAokkAACkAAADl6gAAKgAAAAo5AQArAAAA5igAAC\
    wAAADVfAAALQAAAHYfAAAuAAAALdYAAC8AAAC11gEAMAAAAN1dAAAxAAAAbyEAADIAAADjwAAA.\
    AgAA";
    let is_solution_correct = is_puzzle_result_valid(solution, api_key.as_bytes());
    println!("Is solution correct: {:?}", is_solution_correct);
}
