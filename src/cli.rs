use clap::Parser;
use std::env;

#[derive(Parser)]
#[command(
  name = "hammer-sickle",
  about = "Run commands across Foreman controlled hosts.",
)]
pub struct Cli {
  #[arg(short='c', long)]
  pub command: String,
  #[arg(short='s', long)]
  pub search: String,
  #[command(flatten)]
  pub verbosity: clap_verbosity_flag::Verbosity,
  #[arg(short='u', long, default_value_t = env_user())]
  pub foreman_user: String,
  #[arg(short='p', long)]
  pub foreman_password: String,
  #[arg(short='U', long)]
  pub foreman_url: String,
}

fn env_user() -> String {
  env::var("USER").unwrap()
}
