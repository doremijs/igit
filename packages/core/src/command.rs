use std::error::Error;
use std::process::Command;

pub fn run_command(command: &str, args: &str) -> Result<(), Box<dyn Error>> {
  let cmd = format!("{} {}", command, args);

  let shell = if Command::new("zsh").output().is_ok() {
    "zsh"
  } else if Command::new("bash").output().is_ok() {
    "bash"
  } else {
    "sh"
  };

  let output = Command::new(shell)
    .env("PATH", format!("{}:./node_modules/.bin", std::env::var("PATH").unwrap_or_default()))
    .arg("-c")
    .arg(&cmd)
    .stdout(std::process::Stdio::inherit())
    .stderr(std::process::Stdio::inherit())
    .output()?;

  if !output.status.success() {
    let code = output.status.code().unwrap_or(-1);
    let stderr = String::from_utf8_lossy(&output.stderr);
    return Err(format!(
      "Command '{}' failed with exit code {}\nError: {}",
      cmd, code, stderr
    ).into());
  }
  Ok(())
}
