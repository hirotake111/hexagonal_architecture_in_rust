use std::env;

pub struct Config {
    pub port: u16,
}

pub fn get_config() -> anyhow::Result<Config> {
    let port = env::var("PORT")?.parse()?;
    Ok(Config { port })
}
