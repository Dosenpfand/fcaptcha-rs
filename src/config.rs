use config::Config;

lazy_static! {
    pub static ref CONFIG: Config = Config::builder()
        .add_source(config::File::with_name("conf/default.toml"))
        .add_source(config::Environment::with_prefix("FCAPTCHA").separator("_"))
        .build()
        .unwrap();
}

pub fn get<'a, T: serde::Deserialize<'a>>(key: &str) -> T {
    CONFIG.get::<T>(key).unwrap()
}
