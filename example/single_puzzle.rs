use fcaptcha::{build_puzzle, get, is_puzzle_result_valid};

fn main() {
    env_logger::init();
    let secret_key = get::<String>("SECRET_KEY");
    let api_key = get::<String>("API_KEY");
    let mut puzzle = build_puzzle(secret_key.as_bytes(), "127.0.0.1");
    for _i in 0..100 {
        puzzle = build_puzzle("TEST-KEY".as_bytes(), "127.0.0.1");
    }
    println!("Generated puzzle: {:?}", puzzle);

    let solution =
        "e90a8faafffa2d43f6f9e8a4e1ab0932aa451feaae98c2bc6e8d9596c83c93af.\
        ZO9waAAAAAEAAAABAQwzegAAAAAAAAAAxCeigrqpVmU=.\
        AAAAAGtTAAABAAAA9DkBAAIAAAAinAAAAwAAAKqmAAAEAAAAFCMAAAUAAAAhAAAABgAAAI1vAAAHAAAAeRgAAAgAAACcHwAACQAAAP2SAAAKAAAAwZsAAAsAAABxMgAADAAAAG0yAQANAAAAHz0AAA4AAABaXwAADwAAABopAAAQAAAA8G4AABEAAABIrgAAEgAAAAGCAAATAAAAF8kAABQAAAD5OwEAFQAAALscAAAWAAAANYEAABcAAADBZAAAGAAAAGgCAAAZAAAAJwkAABoAAABmjQAAGwAAAHB+AAAcAAAA9WQAAB0AAACvkAAAHgAAANUhAAAfAAAAbwQAACAAAABD+QAAIQAAALfoAAAiAAAA5D0AACMAAACOBQAAJAAAABtQAAAlAAAAgcoAACYAAAD5MAAAJwAAAKoJAgAoAAAAV4oAACkAAAA1oAAAKgAAAGhYAAArAAAAqSMAACwAAABIKgAALQAAACEdAAAuAAAA0LUBAC8AAAAIFAAAMAAAALCQAAAxAAAAPE0AADIAAABk/QAA.\
        AgAA";
    let is_solution_correct = is_puzzle_result_valid(solution, api_key.as_bytes());
    println!("Is solution correct: {:?}", is_solution_correct);
}
