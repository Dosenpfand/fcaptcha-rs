use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use fcaptcha::{build_puzzle, get, verify_puzzle_result::verify_puzzle_result_with};

fn build_puzzle_benchmark(c: &mut Criterion) {
    let ip_addresses = ["127.0.0.1", "192.168.0.0.1"];

    let mut group = c.benchmark_group("build_puzzle");

    for ip_address in ip_addresses {
        group.bench_with_input(
            BenchmarkId::from_parameter(ip_address),
            ip_address,
            |b, ip_address| {
                b.iter(|| build_puzzle(&ip_address));
            },
        );
    }
    group.finish();
}

fn is_puzzle_result_valid_benchmark(c: &mut Criterion) {
    let secret_key = get::<Vec<u8>>("SECRET_KEY");
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

    c.bench_function(
        "is_puzzle_result_valid",
        |b: &mut criterion::Bencher<'_>| {
            b.iter(|| {
                let result = verify_puzzle_result_with(
                    black_box(solution),
                    black_box(timestamp),
                    black_box(0),
                    black_box(&secret_key),
                );
                assert!(result.is_ok())
            })
        },
    );
}

criterion_group!(
    benches,
    build_puzzle_benchmark,
    is_puzzle_result_valid_benchmark
);
criterion_main!(benches);
