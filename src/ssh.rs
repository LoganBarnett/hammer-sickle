use std::process::Command;

#[derive(Debug)]
pub enum SshError {
    ExecuteError(std::io::Error),
    ParseError(std::string::FromUtf8Error),
}

pub fn host_command_send(
    hostname: String,
    command: String,
) -> Result<String, SshError> {
    Command::new("ssh")
        .arg(hostname)
        .arg(command)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .map_err(SshError::ExecuteError)
        .and_then(|x| {
            String::from_utf8(x.stdout)
                .map_err(SshError::ParseError)
        })
        .map(|x| x.to_string())
}
