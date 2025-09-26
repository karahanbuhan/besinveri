use std::{
    fs::{self, File},
    io::Write,
};

use anyhow::Error;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

#[derive(Serialize, Deserialize)]
pub(crate) struct Config {
    pub(crate) api: APIConfig,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct APIConfig {
    pub(crate) base_url: String,
    pub(crate) static_url: String,
}

pub(crate) fn load_config_with_defaults() -> Result<Config, Error> {
    let path = "config.yml";

    if !fs::exists(path)? {
        create_default_config_file(path)?;
        return Ok(get_default_config());
    }

    let file: Vec<u8> = fs::read(path)?;
    let config: Config = toml::from_slice(&file.as_slice())?;

    Ok(config)
}

fn get_default_config() -> Config {
    Config {
        api: APIConfig {
            base_url: "https://api.besinveri.com".to_owned(),
            static_url: "https://besinveri.com/static".to_owned()
        },
    }
}

fn create_default_config_file(path: &str) -> Result<(), Error> {
    if fs::exists(path)? {
        // Eğer config mevcutsa bir şey yapmaya gerek yok, sadece debug için log atılacak ve işlem atlanacak
        debug!("Config dosyası mevcut, varsayılan config oluşturma işlemi atlanıyor");
        return Ok(());
    }

    let mut file = File::create(path)?;
    let toml = toml::to_string_pretty(&get_default_config())?;
    file.write_all(toml.as_bytes())?;

    info!("Yeni bir config dosyası varsayılan değerler ile oluşturuldu");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_correct_base_url() {
        let config = get_default_config();
        assert_eq!(config.api.base_url, "https://api.besinveri.com");
    }

    #[test]
    fn default_config_is_valid() {
        // TOML serialize edilebiliyor mu test et
        let config = get_default_config();
        let toml_str = toml::to_string_pretty(&config).unwrap();
        assert!(!toml_str.is_empty());

        // Deserialize geri dönebiliyor mu
        let deserialized: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(deserialized.api.base_url, "https://api.besinveri.com");
    }
}
