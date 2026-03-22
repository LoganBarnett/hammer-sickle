use std::io::{BufRead, BufReader, Read};
use std::process::Command;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SshError {
  #[error("Failed to spawn SSH command on {host}: {source}")]
  SpawnFailed {
    host: String,
    #[source]
    source: std::io::Error,
  },
}

fn stream_lines_prefixed(host: &str, s: &mut dyn Read) {
  let reader = BufReader::new(s);
  for line in reader.lines() {
    println!("{}: {}", host, line.unwrap());
  }
}

pub fn host_command_send(
  hostname: &str,
  command: &str,
) -> Result<(), SshError> {
  let mut child = Command::new("ssh")
    .arg(hostname)
    .arg(command)
    .stdout(std::process::Stdio::piped())
    .stderr(std::process::Stdio::piped())
    .spawn()
    .map_err(|source| SshError::SpawnFailed {
      host: hostname.to_string(),
      source,
    })?;

  let stdout = child.stdout.as_mut().unwrap();
  let stderr = child.stderr.as_mut().unwrap();
  stream_lines_prefixed(hostname, stdout);
  stream_lines_prefixed(hostname, stderr);
  child.wait().unwrap();
  Ok(())
}
