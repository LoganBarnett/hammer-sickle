mod cli;
mod error;
mod foreman;
mod ssh;

use clap::Parser;

use cli::Cli;
use error::AppError;
use std::env;
use foreman::ForemanApiPage;
use foreman::ForemanApiHost;

fn env_var(s: &str) -> Result<String, AppError> {
  env::var(s).map_err(AppError::EnvError)
}

fn main() -> Result<(), AppError> {
  let args = Cli::parse();
  let username = env_var("FOREMAN_USER")
    .unwrap_or(env_var("USER")?);
  let password = env_var("FOREMAN_PASS")?;
  let client = reqwest::blocking::Client::new();
  let resp = client.get(
    format!(
      "{url_base}/api/hosts?search={search}",
      url_base = args.foreman_url,
      search = args.search,
    ).as_str()
  )
  // Foreman supports both OAuth and basic auth. Use basic for simplicity for
  // now. See https://projects.theforeman.org/projects/foreman/wiki/API_OAuth
  // when that fateful date arrives.
    .basic_auth(username, Some(password))
    .send()
    .map_err(AppError::EndpointError)?
    .json::<ForemanApiPage<ForemanApiHost>>()
    .map_err(AppError::EndpointError)?
  ;
  let hosts = resp.results.iter().map(|x| x.name.clone());
  let _ = hosts
    .map(|host| ssh::host_command_send(&host, args.command.clone()))
    .collect::<Vec<Result<(), ssh::SshError>>>()
    .into_iter()
    .map(|r| r.map_err(AppError::SshError))
    ;
  Ok(())
}
