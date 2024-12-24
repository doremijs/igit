use std::error::Error;
use std::process::Command;

pub fn run_command(command: &str, args: &str) -> Result<(), Box<dyn Error>> {
  let output = Command::new("sh")
    .arg("-c")
    .arg(format!("PATH=$PATH:./node_modules/.bin {} {}", command, args))
    // .stdout(std::process::Stdio::inherit())
    .stderr(std::process::Stdio::inherit())
    .output()?;
  if !output.status.success() {
    return Err("Command failed".into());
  }
  Ok(())
}
