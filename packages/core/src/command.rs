use std::error::Error;
use std::process::{Command, Stdio};

pub fn run_command(command: &str, args: &str) -> Result<(), Box<dyn Error>> {
  let output = Command::new("sh")
    .arg("-c")
    .arg(format!("PATH=$PATH:./node_modules/.bin {} {}", command, args))
    .stdin(Stdio::inherit())
    .stdout(Stdio::inherit())
    .stderr(Stdio::inherit())
    .output()?;
  if !output.status.success() {
    return Err("Command failed".into());
  }
  Ok(())
}
