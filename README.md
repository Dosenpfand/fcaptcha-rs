# fcaptcha-rs

An *experimental* alternative implementation of [FriendlyCaptcha/friendly-lite-server](https://github.com/FriendlyCaptcha/friendly-lite-server) in Rust using [Actix Web](https://actix.rs/).

## Run

## Server

```
cargo run
```
or
```
docker-compose up
```
and open http://localhost:8080/build-puzzle

## Demo

```
cargo run --example fcaptcha-demo
```
or
```
docker-compose --file example/docker-compose.yml up
```
and open http://localhost:8080

## ToDo
1. [ ] Resolve all `TODO`s in the code.
2. [ ] Benchmark against reference implementation.
3. [ ] Documentation.
4. [ ] Tests.