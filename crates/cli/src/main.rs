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

#[derive(Debug, serde::Serialize)]
struct HostResult {
  host: String,
  exit_code: Option<i32>,
  #[serde(skip_serializing_if = "Option::is_none")]
  error: Option<String>,
}

fn main() {
  let cli = CliRaw::parse();

  let config = match Config::from_cli_and_file(cli) {
    Ok(c) => c,
    Err(e) => {
      eprintln!("Configuration error: {}", e);
      std::process::exit(1);
    }
  };

  init_logging(config.log_level, config.log_format);

  info!("Starting hammer-sickle");

  let results = match run(&config) {
    Ok(r) => r,
    Err(e) => {
      eprintln!("{}", e);
      std::process::exit(1);
    }
  };

  if config.report_json {
    println!(
      "{}",
      serde_json::to_string_pretty(&results)
        .expect("Failed to serialize results")
    );
  }

  let any_failed = results.iter().any(|r| match r.exit_code {
    Some(code) => !config.success_codes.contains(&code),
    None => true,
  });

  info!("Done");

  if any_failed {
    std::process::exit(1);
  }
}

fn run(config: &Config) -> Result<Vec<HostResult>, ApplicationError> {
  let hosts = foreman::fetch_hosts(config)?;
  info!(
    count = hosts.len(),
    search = %config.search,
    "Fetched hosts from Foreman",
  );

  let Some(command) = config.command.as_deref() else {
    hosts.iter().for_each(|h| println!("{}", h));
    return Ok(Vec::new());
  };

  rayon::ThreadPoolBuilder::new()
    .num_threads(config.concurrency)
    .build()
    .map_err(|source| ApplicationError::ThreadPoolBuild {
      threads: config.concurrency,
      source,
    })?
    .install(|| {
      Ok(
        hosts
          .par_iter()
          .map(|host| {
            let mut output: Box<dyn std::io::Write + Send> =
              if config.report_json {
                Box::new(std::io::stderr())
              } else {
                Box::new(std::io::stdout())
              };

            match ssh::host_command_send(host, command, &mut *output) {
              Ok(code) => HostResult {
                host: host.clone(),
                exit_code: Some(code),
                error: None,
              },
              Err(e) => {
                warn!(host = %host, error = %e, "SSH command failed");
                HostResult {
                  host: host.clone(),
                  exit_code: None,
                  error: Some(e.to_string()),
                }
              }
            }
          })
          .collect(),
      )
    })
}
