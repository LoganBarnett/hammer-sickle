use hash_color_lib::{detect_color_support, ColorizerOptions, HashColorizer};
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

fn stream_lines_prefixed(
  host: &str,
  colorizer: &HashColorizer,
  s: &mut dyn Read,
) {
  let colored_host = colorizer.colorize(host);
  let reader = BufReader::new(s);
  for line in reader.lines() {
    println!("{}: {}", colored_host, line.unwrap());
  }
}

pub fn host_command_send(
  hostname: &str,
  command: &str,
) -> Result<(), SshError> {
  let colorizer = HashColorizer::new(ColorizerOptions {
    color_support: Some(detect_color_support()),
    ..ColorizerOptions::default()
  });

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
  stream_lines_prefixed(hostname, &colorizer, stdout);
  stream_lines_prefixed(hostname, &colorizer, stderr);
  child.wait().unwrap();
  Ok(())
}
