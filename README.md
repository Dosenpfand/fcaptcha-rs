# fcaptcha-rs

![Build Status](https://github.com/Dosenpfand/fcaptcha-rs/actions/workflows/ci.yml/badge.svg?branch=master)
[![Covde Coverage](https://codecov.io/github/Dosenpfand/fcaptcha-rs/coverage.svg?branch=master)](https://codecov.io/gh/Dosenpfand/fcaptcha-rs)

An *experimental* alternative implementation of [FriendlyCaptcha/friendly-lite-server](https://github.com/FriendlyCaptcha/friendly-lite-server) in Rust using [Actix Web](https://actix.rs/).

## Configuration

Adapt the values in `conf/default.toml` or set environment variables with the same names and an additional `FCAPTCHA_` prefix.

## Run

## Server

Barebone server hosting puzzle generation and solution verification.

```
cargo run
```
or
```
docker-compose up
```
and open http://localhost:8080/build-puzzle

## Web Demo

Demo with generation, verification and widget.

```
cargo run --example fcaptcha-demo
```
or
```
docker-compose --file example/docker-compose.yml up
```
and open http://localhost:8080

## CLI Demo

Demo generating a single puzzle and verifying a single solution.

```
cargo run --example fcaptcha-single-puzzle
```

## Benchmark

Benchmark puzzle generation and solution verification.

```
cargo bench
```

## Flamegraph

Requires `perf` and `cargo-flamegraph`.

```
CARGO_PROFILE_RELEASE_DEBUG=true cargo flamegraph --bench benchmark
```

## ToDo
1. [ ] Resolve all `TODO`s in the code.
2. [ ] Benchmark against reference implementation.
3. [ ] Documentation.
4. [ ] Tests.
5. [ ] Proper error handling: Get rid of all `unwrap`s, etc.
