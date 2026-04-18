use hash_color_lib::{detect_color_support, ColorizerOptions, HashColorizer};
use std::io::{BufRead, BufReader, Read, Write};
use std::process::Command;
use thiserror::Error;
use tracing::warn;

#[derive(Debug, Error)]
pub enum SshError {
  #[error("Failed to spawn SSH command on {host}: {source}")]
  SpawnFailed {
    host: String,
    #[source]
    source: std::io::Error,
  },

  #[error("SSH process on {host} was killed by a signal")]
  KilledBySignal { host: String },

  #[error("Failed to wait for SSH process on {host}: {source}")]
  WaitFailed {
    host: String,
    #[source]
    source: std::io::Error,
  },
}

fn stream_lines_prefixed(
  host: &str,
  colorizer: &HashColorizer,
  s: &mut dyn Read,
  output: &mut dyn Write,
) {
  let colored_host = colorizer.colorize(host);
  let reader = BufReader::new(s);
  for line in reader.lines() {
    match line {
      Ok(text) => {
        let _ = writeln!(output, "{}: {}", colored_host, text);
      }
      Err(e) => {
        warn!(host = %host, error = %e, "Failed to read line from SSH output");
      }
    }
  }
}

pub fn host_command_send(
  hostname: &str,
  command: &str,
  output: &mut dyn Write,
) -> Result<i32, SshError> {
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
  stream_lines_prefixed(hostname, &colorizer, stdout, output);

  let stderr = child.stderr.as_mut().unwrap();
  stream_lines_prefixed(hostname, &colorizer, stderr, output);

  child
    .wait()
    .map_err(|source| SshError::WaitFailed {
      host: hostname.to_string(),
      source,
    })?
    .code()
    .ok_or_else(|| SshError::KilledBySignal {
      host: hostname.to_string(),
    })
}
