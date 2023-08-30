use criterion::{black_box, criterion_group, criterion_main, Criterion};
use fcaptcha::build_puzzle;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("fib 20", |b| {
        b.iter(|| build_puzzle(black_box("TEST-KEY".as_bytes()), black_box("127.0.0.1")))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
