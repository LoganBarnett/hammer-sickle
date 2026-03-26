use clap::Parser;
use hammer_sickle_lib::{LogFormat, LogLevel};
use serde::Deserialize;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
  #[error(
    "Failed to read configuration file at {path:?} during startup: {source}"
  )]
  ConfigFileRead {
    path: PathBuf,
    #[source]
    source: std::io::Error,
  },

  #[error("Failed to parse configuration file at {path:?}: {source}")]
  ConfigFileParse {
    path: PathBuf,
    #[source]
    source: toml::de::Error,
  },

  #[error("Configuration validation failed: {0}")]
  Validation(String),
}

#[derive(Debug, Parser)]
#[command(
  name = "hammer-sickle",
  version,
  about = "Run commands across Foreman-managed hosts."
)]
pub struct CliRaw {
  /// Shell command to run on matched hosts (omit to list matching hostnames)
  #[arg(short = 'c', long)]
  pub command: Option<String>,

  /// Foreman host search expression
  #[arg(short = 's', long)]
  pub search: String,

  /// Foreman base URL
  #[arg(short = 'U', long, env = "FOREMAN_URL")]
  pub foreman_url: Option<String>,

  /// Foreman username (defaults to $FOREMAN_USER then $USER)
  #[arg(short = 'u', long, env = "FOREMAN_USER")]
  pub foreman_user: Option<String>,

  /// Foreman password
  #[arg(short = 'p', long, env = "FOREMAN_PASS")]
  pub foreman_password: Option<String>,

  /// Log level (trace, debug, info, warn, error)
  #[arg(long, env = "LOG_LEVEL")]
  pub log_level: Option<String>,

  /// Log format (text, json)
  #[arg(long, env = "LOG_FORMAT")]
  pub log_format: Option<String>,

  /// Maximum number of hosts to contact simultaneously (default: 20, 0 = all at once)
  #[arg(short = 'j', long, env = "CONCURRENCY")]
  pub concurrency: Option<usize>,

  /// Path to configuration file
  #[arg(long, env = "CONFIG_FILE")]
  pub config: Option<PathBuf>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ConfigFileRaw {
  pub log_level: Option<String>,
  pub log_format: Option<String>,
  pub foreman_url: Option<String>,
  pub foreman_user: Option<String>,
  pub foreman_password: Option<String>,
  pub concurrency: Option<usize>,
}

impl ConfigFileRaw {
  pub fn from_file(path: &PathBuf) -> Result<Self, ConfigError> {
    let contents = std::fs::read_to_string(path).map_err(|source| {
      ConfigError::ConfigFileRead {
        path: path.clone(),
        source,
      }
    })?;

    toml::from_str(&contents).map_err(|source| ConfigError::ConfigFileParse {
      path: path.clone(),
      source,
    })
  }
}

#[derive(Debug)]
pub struct Config {
  pub log_level: LogLevel,
  pub log_format: LogFormat,
  pub command: Option<String>,
  pub search: String,
  pub foreman_url: String,
  pub foreman_user: String,
  pub foreman_password: String,
  // 0 means rayon auto-detects (one thread per logical CPU).
  pub concurrency: usize,
}

impl Config {
  pub fn from_cli_and_file(cli: CliRaw) -> Result<Self, ConfigError> {
    let config_file = if let Some(config_path) = &cli.config {
      ConfigFileRaw::from_file(config_path)?
    } else {
      let default_config_path = PathBuf::from("config.toml");
      if default_config_path.exists() {
        ConfigFileRaw::from_file(&default_config_path)?
      } else {
        ConfigFileRaw::default()
      }
    };

    let log_level = cli
      .log_level
      .or(config_file.log_level)
      .unwrap_or_else(|| "info".to_string())
      .parse::<LogLevel>()
      .map_err(|e| ConfigError::Validation(e.to_string()))?;

    let log_format = cli
      .log_format
      .or(config_file.log_format)
      .unwrap_or_else(|| "text".to_string())
      .parse::<LogFormat>()
      .map_err(|e| ConfigError::Validation(e.to_string()))?;

    let foreman_url =
      cli.foreman_url.or(config_file.foreman_url).ok_or_else(|| {
        ConfigError::Validation(
          "Foreman URL is required (--foreman-url or FOREMAN_URL)".to_string(),
        )
      })?;

    // Prefer FOREMAN_USER/--foreman-user, then fall back to the login name.
    let foreman_user = cli
      .foreman_user
      .or(config_file.foreman_user)
      .or_else(|| std::env::var("USER").ok())
      .ok_or_else(|| {
        ConfigError::Validation(
          "Foreman user could not be determined (--foreman-user, \
           FOREMAN_USER, or USER)"
            .to_string(),
        )
      })?;

    let foreman_password = cli
      .foreman_password
      .or(config_file.foreman_password)
      .ok_or_else(|| {
        ConfigError::Validation(
          "Foreman password is required (--foreman-password or FOREMAN_PASS)"
            .to_string(),
        )
      })?;

    let concurrency = cli.concurrency.or(config_file.concurrency).unwrap_or(20);

    Ok(Config {
      log_level,
      log_format,
      command: cli.command,
      search: cli.search,
      foreman_url,
      foreman_user,
      foreman_password,
      concurrency,
    })
  }
}
