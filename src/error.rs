use crate::ssh;

#[derive(Debug)]
pub enum AppError {
  EndpointError(reqwest::Error),
  EnvError(std::env::VarError),
  SshError(ssh::SshError),
}
