use serde::Deserialize;
use std::env;
use std::fs;

#[derive(Default, Deserialize)]
pub struct Config {
    pub meta_grpc_url: String,
}

#[derive(Deserialize)]
pub struct Settings {
    #[serde(default)]
    pub def: Config,
    pub dev: Config,
    pub prod: Config,
}

impl Settings {
    pub fn load(config_file: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let config_contents = fs::read_to_string(config_file)?;
        let config: Settings = toml::de::from_str(&config_contents)?;
        Ok(config)
    }
}

pub fn get_cfg() -> Config {
    let settings = Settings::load("config.toml").expect("Failed to load config");

    let env_var = env::var("OLAP_ENV");

    match env_var {
        Ok(value) => {
            println!("OLAP_ENV is set to '{}', using corresponding configuration.", value);
            match value.as_str() {
                "dev" => settings.dev,
                "prod" => settings.prod,
                _ => settings.def,
            }
        }
        Err(_) => {
            println!("OLAP_ENV not set, using default configuration.");
            settings.def
        }
    }
}
