use std::env;
use structopt::StructOpt;
mod foreman;
use foreman::ForemanApiPage;
use foreman::ForemanApiHost;
mod ssh;

#[derive(StructOpt)]
#[structopt(
    name = "hammer-sickle",
    about = "Run commands across Foreman controlled hosts.",
)]
struct Cli {
    #[structopt(short, long)]
    command: String,
    #[structopt(short, long)]
    search: String,
}

#[derive(Debug)]
enum AppError {
    EndpointError(reqwest::Error),
    EnvError(std::env::VarError),
    SshError(ssh::SshError),
}

fn env_var(s: &str) -> Result<String, AppError> {
    env::var(s).map_err(AppError::EnvError)
}

fn main() -> Result<(), AppError> {
    let username = env_var("FOREMAN_USER")
        .unwrap_or(env_var("USER")?);
    let password = env_var("FOREMAN_PASS")?;
    let url_base = env_var("FOREMAN_URL_BASE")?;
    let args = Cli::from_args();
    let client = reqwest::blocking::Client::new();
    let resp = client.get(
        format!(
            "{url_base}/api/hosts?search={search}",
            url_base = url_base,
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
    hosts
        .map(|host| ssh::host_command_send(host, args.command.clone()))
        .collect::<Result<Vec<String>, ssh::SshError>>()
        .map_err(AppError::SshError)
        // TODO: Show the output.
        .map(|_| ())
}
