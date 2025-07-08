// >> Code Review: Power of Hippo << --- doing......
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

// pub fn get_cfg() -> Config {
//     let settings = Settings::load("config.toml").expect("Failed to load config");

//     match env::var("OLAP_ENV").unwrap_or_else(|_| "def".to_string()).as_str() {
//         "dev" => settings.dev,
//         "prod" => settings.prod,
//         _ => settings.def,
//     }
// }

pub fn get_cfg() -> Config {
    // 加载配置
    let settings = Settings::load("config.toml").expect("Failed to load config");

    // 获取环境变量
    let env_var = env::var("OLAP_ENV");

    // 如果没有设置环境变量，打印消息
    match env_var {
        Ok(value) => {
            println!("<<<<<<<<<<<<<<<<< <<<<<<<<<<<<<<<<< <<<<<<<<<<<<<<<<< OLAP_ENV is set to: {}", value);
            match value.as_str() {
                "dev" => settings.dev,
                "prod" => settings.prod,
                _ => settings.def,
            }
        }
        Err(_) => {
            println!("<<<<<<<<<<<<<<<<< <<<<<<<<<<<<<<<<< <<<<<<<<<<<<<<<<< 没有OLAP_ENV环境变量");
            settings.def // 默认配置
        }
    }
}
