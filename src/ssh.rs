use std::io::BufRead;
use std::io::BufReader;
use std::io::Read;
use std::process::Command;

#[derive(Debug)]
pub enum SshError {
  ExecuteError(std::io::Error),
}

fn stdio_stream_from_host(host: &str, s: &mut dyn Read) -> () {
  let reader = BufReader::new(s);
  for line in reader.lines() {
    println!("{}: {}", host, line.unwrap());
  }
}

pub fn host_command_send(
  hostname: &str,
  command: String,
) -> Result<(), SshError> {
  let mut command = Command::new("ssh")
    .arg(hostname)
    .arg(command)
    .stdout(std::process::Stdio::piped())
    .stderr(std::process::Stdio::piped())
    .spawn()
    .map_err(SshError::ExecuteError)
    ?;
  let stdout = command.stdout.as_mut().unwrap();
  let stderr = command.stderr.as_mut().unwrap();
  stdio_stream_from_host(hostname, stdout);
  stdio_stream_from_host(hostname, stderr);
  command.wait().unwrap();
  Ok(())
}
