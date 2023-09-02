use config::Config;

lazy_static! {
    static ref CONFIG: Config = Config::builder()
        .set_default("BIND_ADDRESS", "0.0.0.0")
        .unwrap()
        .set_default("BIND_PORT", 8080)
        .unwrap()
        .set_default("ACCESS_TTL", 1800)
        .unwrap()
        .set_default("PUZZLE_TTL", 3600)
        .unwrap()
        .set_default("SECRET_KEY", "NOT-A-SECRET-KEY".as_bytes().to_vec())
        .unwrap()
        .set_default("API_KEY", "NOT-AN-API-KEY".as_bytes().to_vec())
        .unwrap()
        .add_source(config::Environment::with_prefix("FCAPTCHA").separator("_"))
        .build()
        .unwrap();
}

/// Get a configuration element.
pub fn get<'a, T: serde::Deserialize<'a>>(key: &str) -> T {
    CONFIG.get::<T>(key).unwrap()
}
