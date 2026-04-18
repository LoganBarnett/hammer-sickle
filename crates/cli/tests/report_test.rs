use serde::Deserialize;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener};
use std::path::PathBuf;
use std::process::Command;
use std::thread::{self, JoinHandle};

/// Minimal mirror of the HostResult struct in main.rs, for deserialization.
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct HostResult {
  host: String,
  exit_code: Option<i32>,
  error: Option<String>,
}

fn get_binary_path() -> PathBuf {
  let mut path =
    std::env::current_exe().expect("Failed to get current executable path");
  path.pop();
  path.pop();
  path.push("hammer-sickle");

  if !path.exists() {
    path.pop();
    path.pop();
    path.push("debug");
    path.push("hammer-sickle");
  }

  path
}

/// RAII guard that removes a temporary directory on drop.
struct TempDir {
  path: PathBuf,
}

impl TempDir {
  fn new(prefix: &str) -> Self {
    let path = std::env::temp_dir().join(format!(
      "{}-{}-{}",
      prefix,
      std::process::id(),
      std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos()
    ));
    std::fs::create_dir_all(&path).expect("Failed to create temp directory");
    Self { path }
  }
}

impl Drop for TempDir {
  fn drop(&mut self) {
    let _ = std::fs::remove_dir_all(&self.path);
  }
}

/// Creates a temp directory containing a mock `ssh` script that echoes the
/// hostname and exits with the given code.
fn mock_ssh_dir(exit_code: i32) -> TempDir {
  let dir = TempDir::new("mock-ssh");
  let script_path = dir.path.join("ssh");
  let script =
    format!("#!/bin/sh\necho \"$1: mock output\"\nexit {}\n", exit_code);
  std::fs::write(&script_path, &script)
    .expect("Failed to write mock ssh script");

  #[cfg(unix)]
  {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(
      &script_path,
      std::fs::Permissions::from_mode(0o755),
    )
    .expect("Failed to chmod mock ssh script");
  }

  dir
}

/// Starts a minimal HTTP server that returns a single-page Foreman API
/// response containing the given hostnames.  The thread exits when the
/// listener is dropped (accept returns an error).
fn start_mock_foreman(hosts: &[&str]) -> (SocketAddr, JoinHandle<()>) {
  let listener =
    TcpListener::bind("127.0.0.1:0").expect("Failed to bind mock Foreman");
  let addr = listener.local_addr().unwrap();

  let results: Vec<String> = hosts
    .iter()
    .enumerate()
    .map(|(i, name)| format!(r#"{{"id": {}, "name": "{}"}}"#, i + 1, name))
    .collect();

  let json_body = format!(
    r#"{{"page":1,"per_page":250,"search":null,"results":[{}],"sort":null,"subtotal":{},"total":{}}}"#,
    results.join(","),
    hosts.len(),
    hosts.len()
  );

  let handle = thread::spawn(move || {
    while let Ok((mut stream, _)) = listener.accept() {
      let mut buf = [0u8; 4096];
      let _ = stream.read(&mut buf);
      let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
        json_body.len(),
        json_body
      );
      let _ = stream.write_all(response.as_bytes());
    }
  });

  (addr, handle)
}

/// Builds a Command pre-configured with mock SSH PATH, mock Foreman URL,
/// dummy credentials, and quiet logging.
fn base_command(foreman_addr: &SocketAddr, ssh_dir: &TempDir) -> Command {
  let mut cmd = Command::new(get_binary_path());

  let original_path = std::env::var("PATH").unwrap_or_default();
  let new_path = format!("{}:{}", ssh_dir.path.display(), original_path);

  cmd
    .env("PATH", new_path)
    .env("NO_COLOR", "1")
    .arg("--foreman-url")
    .arg(format!("http://{}", foreman_addr))
    .arg("--foreman-user")
    .arg("test")
    .arg("--foreman-password")
    .arg("test")
    .arg("--search")
    .arg("test")
    .arg("--log-level")
    .arg("error")
    .arg("-j")
    .arg("1");

  cmd
}

#[test]
fn test_report_json_success() {
  let ssh_dir = mock_ssh_dir(0);
  let (addr, _handle) =
    start_mock_foreman(&["host1.example.com", "host2.example.com"]);

  let output = base_command(&addr, &ssh_dir)
    .arg("--report-json")
    .arg("-c")
    .arg("hostname")
    .output()
    .expect("Failed to execute binary");

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    output.status.success(),
    "Expected exit 0, got {:?}\nstdout: {}\nstderr: {}",
    output.status.code(),
    stdout,
    String::from_utf8_lossy(&output.stderr)
  );

  let results: Vec<HostResult> =
    serde_json::from_str(&stdout).unwrap_or_else(|e| {
      panic!("Failed to parse JSON: {}\nstdout: {}", e, stdout)
    });

  assert_eq!(results.len(), 2);
  for result in &results {
    assert_eq!(result.exit_code, Some(0));
    assert!(result.error.is_none());
  }
}

#[test]
fn test_report_json_failure() {
  let ssh_dir = mock_ssh_dir(1);
  let (addr, _handle) =
    start_mock_foreman(&["host1.example.com", "host2.example.com"]);

  let output = base_command(&addr, &ssh_dir)
    .arg("--report-json")
    .arg("-c")
    .arg("fail")
    .output()
    .expect("Failed to execute binary");

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert_eq!(
    output.status.code(),
    Some(1),
    "Expected exit 1\nstdout: {}\nstderr: {}",
    stdout,
    String::from_utf8_lossy(&output.stderr)
  );

  let results: Vec<HostResult> =
    serde_json::from_str(&stdout).unwrap_or_else(|e| {
      panic!("Failed to parse JSON: {}\nstdout: {}", e, stdout)
    });

  assert_eq!(results.len(), 2);
  for result in &results {
    assert_eq!(result.exit_code, Some(1));
  }
}

#[test]
fn test_custom_success_codes() {
  let ssh_dir = mock_ssh_dir(2);
  let (addr, _handle) = start_mock_foreman(&["host1.example.com"]);

  let output = base_command(&addr, &ssh_dir)
    .arg("--report-json")
    .arg("--success-codes")
    .arg("0,2")
    .arg("-c")
    .arg("puppet")
    .output()
    .expect("Failed to execute binary");

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    output.status.success(),
    "Expected exit 0 (code 2 is in success set)\nstdout: {}\nstderr: {}",
    stdout,
    String::from_utf8_lossy(&output.stderr)
  );
}

#[test]
fn test_exit_propagation_without_json() {
  let ssh_dir = mock_ssh_dir(1);
  let (addr, _handle) = start_mock_foreman(&["host1.example.com"]);

  let output = base_command(&addr, &ssh_dir)
    .arg("-c")
    .arg("fail")
    .output()
    .expect("Failed to execute binary");

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert_eq!(
    output.status.code(),
    Some(1),
    "Expected exit 1\nstdout: {}\nstderr: {}",
    stdout,
    String::from_utf8_lossy(&output.stderr)
  );

  // Without --report-json, stdout should contain prefixed lines, not JSON.
  assert!(
    serde_json::from_str::<Vec<HostResult>>(&stdout).is_err(),
    "Stdout should not be valid JSON without --report-json"
  );
  assert!(
    stdout.contains("host1.example.com"),
    "Expected host-prefixed output, got: {}",
    stdout
  );
}

#[test]
fn test_no_command_lists_hosts() {
  let ssh_dir = mock_ssh_dir(0);
  let (addr, _handle) =
    start_mock_foreman(&["alpha.example.com", "beta.example.com"]);

  let output = base_command(&addr, &ssh_dir)
    .output()
    .expect("Failed to execute binary");

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    output.status.success(),
    "Expected exit 0\nstdout: {}\nstderr: {}",
    stdout,
    String::from_utf8_lossy(&output.stderr)
  );

  let lines: Vec<&str> = stdout.lines().collect();
  assert!(
    lines.contains(&"alpha.example.com"),
    "Expected alpha.example.com on its own line, got: {:?}",
    lines
  );
  assert!(
    lines.contains(&"beta.example.com"),
    "Expected beta.example.com on its own line, got: {:?}",
    lines
  );
}
