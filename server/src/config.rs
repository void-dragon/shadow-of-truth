use log::info;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
  pub port: u16,
  pub private_key: String,
}

pub fn load() -> Result<Config, Box<dyn std::error::Error>> {
  let path = std::path::Path::new("data/config.toml");

  if path.exists() {
    info!("load config");

    let config = std::fs::read_to_string(path)?;
    Ok(toml::from_str(&config)?)
  }
  else {
    info!("generate default config");

    let config = Config {
      port: 3000,
      private_key: "data/server.key".to_owned(),
    };
    let data = toml::to_string_pretty(&config)?;
    std::fs::write(path, data)?;

    Ok(config)
  }
}