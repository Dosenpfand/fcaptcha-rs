use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use fcaptcha::build_puzzle;

fn criterion_benchmark(c: &mut Criterion) {
    let api_key = "b6db8801-4b39-4516-bd74-5eed7d7433a5".as_bytes();
    let ip_addresses = ["127.0.0.1", "192.168.0.0.1"];

    let mut group = c.benchmark_group("build_puzzle");

    for ip_address in ip_addresses {
        group.bench_with_input(
            BenchmarkId::from_parameter(ip_address),
            ip_address,
            |b, ip_address| {
                b.iter(|| build_puzzle(api_key, &ip_address));
            },
        );
    }
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
