use std::{env, fs::File, io::BufReader};

use anyhow::{bail, Context};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct NotifyConfig {
    pub server: ServerConfig,
    pub auth: AuthConfig,
}

#[derive(Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub db_url: String,
}

#[derive(Serialize, Deserialize)]
pub struct AuthConfig {
    pub pk: String,
}

impl NotifyConfig {
    pub fn load() -> anyhow::Result<Self> {
        let ret = match (
            File::open("notify.yml"),
            File::open("notify.yaml"),
            env::var("NOTIFY_CONFIG"),
        ) {
            (_, _, Ok(path)) => {
                let file =
                    File::open(path).with_context(|| "Unable to open config file".to_string())?;
                let reader = BufReader::new(file);
                serde_yaml::from_reader(reader)?
            }
            (Ok(file), _, _) => {
                let reader = BufReader::new(file);
                serde_yaml::from_reader(reader)?
            }
            (_, Ok(file), _) => {
                let reader = BufReader::new(file);
                serde_yaml::from_reader(reader)?
            }
            _ => bail!("No config file found"),
        };

        Ok(ret)
    }
}
