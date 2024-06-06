use crate::{error::Error, Result};
use serde::Deserialize;
use std::{net::IpAddr, path::Path};

const SRV_ADDR: &str = "127.0.0.1";
const SRV_PORT: usize = 8080;
const SRV_KEEP_ALIVE: usize = 60;
const SRV_FORMS_LIMIT: usize = 1024 * 256;
const SRV_JSON_LIMIT: usize = 1024 * 256;
const SRV_SECRET_KEY: &str = "t/xZkYvxfC8CSfTSH9ANiIR9t1SvLHqOYZ7vH4fp11s=";
const SRV_LOG_LEVEL: &str = "info";
const SVR_STORAGE_PATH: &str = "/tmp/test";

/// Rocket API Server parameters
#[derive(Deserialize, Clone, Debug, Default)]
pub struct Settings {
    /// Server config related parameters
    #[serde(default)]
    pub server: ServerConfig,

    /// Application configuration
    pub app: Option<App>,
}

impl Settings {
    /// Creates a new instance of `Self` by reading configuration from a file.
    ///
    /// # Arguments
    ///
    /// * `path` - A path to the configuration file.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the parsed configuration if successful, or an `Error` if an error occurred.
    ///
    /// # Errors
    ///
    /// Returns an `Error::ConfigurationError` if there was an error parsing the configuration file.
    pub fn from_file<P: AsRef<Path> + ToString>(path: P) -> Result<Self> {
        let cfg = config::Config::builder()
            .add_source(config::File::with_name(path.as_ref().to_str().unwrap()))
            .build()
            .unwrap();

        match cfg.try_deserialize() {
            Ok(c) => Ok(c),
            Err(e) => {
                println!("err: {:?}", e);
                Err(Error::ConfigurationError)
            }
        }
    }
}

/// Rocket Server params
#[derive(Deserialize, Clone, Debug)]
pub struct ServerConfig {
    /// Server Ip Address to start Rocket API Server
    #[serde(default = "default_server_host")]
    pub host: IpAddr,
    /// Server port to listen Rocket API Server
    #[serde(default = "default_server_port")]
    pub port: usize,
    /// Server Keep Alive
    #[serde(default = "default_server_keep_alive")]
    pub keep_alive: usize,
    /// Forms limitation
    #[serde(default = "default_server_forms_limit")]
    pub forms_limit: usize,
    /// JSON transfer limitation
    #[serde(default = "default_server_json_limit")]
    pub json_limit: usize,
    /// Api Server Secret key
    #[serde(default = "default_server_secret_key")]
    pub secret_key: String,
    /// Allow CORS (sometimes helps in testing APIs across domains)
    #[serde(default)]
    pub allow_cors: bool,
    #[serde(default = "default_server_log_level")]
    pub log_level: String,
    #[serde(default = "default_server_storage_path")]
    pub storage_path: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: SRV_ADDR.parse().unwrap(),
            port: SRV_PORT,
            keep_alive: SRV_KEEP_ALIVE,
            forms_limit: SRV_FORMS_LIMIT,
            json_limit: SRV_JSON_LIMIT,
            secret_key: SRV_SECRET_KEY.into(),
            allow_cors: false,
            log_level: SRV_LOG_LEVEL.into(),
            storage_path: SVR_STORAGE_PATH.into(),
        }
    }
}

/// Application related parameters
#[derive(Deserialize, Clone, Debug)]
pub struct App {
    #[serde(default = "default_db_path")]
    pub db_path: String,
}

impl Default for App {
    fn default() -> Self {
        Self {
            db_path: default_db_path(),
        }
    }
}

// All Server defaults
fn default_server_host() -> IpAddr {
    SRV_ADDR.parse().unwrap()
}

fn default_server_port() -> usize {
    SRV_PORT
}

fn default_server_keep_alive() -> usize {
    SRV_KEEP_ALIVE
}

fn default_server_forms_limit() -> usize {
    SRV_FORMS_LIMIT
}

fn default_server_json_limit() -> usize {
    SRV_JSON_LIMIT
}

fn default_server_secret_key() -> String {
    SRV_SECRET_KEY.into()
}

fn default_server_log_level() -> String {
    SRV_LOG_LEVEL.into()
}

fn default_server_storage_path() -> String {
    SVR_STORAGE_PATH.into()
}
// All Application defaults
fn default_db_path() -> String {
    "db.sqlite".into()
}
