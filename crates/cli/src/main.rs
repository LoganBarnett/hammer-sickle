//! hammer-sickle - Run commands across Foreman-managed hosts
//!
//! # LLM Development Guidelines
//! When modifying this code:
//! - Keep configuration logic in config.rs
//! - Keep business logic out of main.rs - use separate modules
//! - Maintain the staged configuration pattern (CliRaw -> ConfigFileRaw -> Config)
//! - Use semantic error types with thiserror - NO anyhow blindly wrapping errors
//! - Add context at each error site explaining WHAT failed and WHY
//! - Keep logging structured and consistent

mod config;
mod foreman;
mod logging;
mod ssh;

use clap::Parser;
use config::{CliRaw, Config, ConfigError};
use foreman::ForemanError;
use logging::init_logging;
use rayon::prelude::*;
use thiserror::Error;
use tracing::{info, warn};

#[derive(Debug, Error)]
enum ApplicationError {
  #[error("Failed to load configuration during startup: {0}")]
  ConfigurationLoad(#[from] ConfigError),

  #[error("Failed to fetch hosts from Foreman: {0}")]
  ForemanFetch(#[from] ForemanError),

  #[error("Failed to build thread pool with {threads} threads: {source}")]
  ThreadPoolBuild {
    threads: usize,
    #[source]
    source: rayon::ThreadPoolBuildError,
  },
}

fn main() -> Result<(), ApplicationError> {
  let cli = CliRaw::parse();

  let config = Config::from_cli_and_file(cli).map_err(|e| {
    eprintln!("Configuration error: {}", e);
    ApplicationError::ConfigurationLoad(e)
  })?;

  init_logging(config.log_level, config.log_format);

  info!("Starting hammer-sickle");

  run(config)?;

  info!("Done");
  Ok(())
}

fn run(config: Config) -> Result<(), ApplicationError> {
  let hosts = foreman::fetch_hosts(&config)?;
  info!(
    count = hosts.len(),
    search = %config.search,
    "Fetched hosts from Foreman",
  );

  let Some(command) = config.command else {
    hosts.iter().for_each(|h| println!("{}", h));
    return Ok(());
  };

  rayon::ThreadPoolBuilder::new()
    .num_threads(config.concurrency)
    .build()
    .map_err(|source| ApplicationError::ThreadPoolBuild {
      threads: config.concurrency,
      source,
    })?
    .install(|| {
      hosts.par_iter().for_each(|host| {
        if let Err(e) = ssh::host_command_send(host, &command) {
          warn!(host = %host, error = %e, "SSH command failed");
        }
      });
    });

  Ok(())
}
