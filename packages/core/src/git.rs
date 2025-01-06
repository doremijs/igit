use std::{path::Path, process::Command};

pub fn is_git_installed() -> bool {
  Command::new("git")
    .arg("--version")
    .stdout(std::process::Stdio::null())
    .stderr(std::process::Stdio::null())
    .status()
    .map_err(|e| format!("Failed to check if git exists: {}", e))
    .unwrap()
    .success()
}

pub fn is_git_repo() -> bool {
  // let is_in_repo = Command::new("git")
  //   .arg("rev-parse")
  //   .arg("--is-inside-work-tree")
  //   .stdout(std::process::Stdio::null())
  //   .stderr(std::process::Stdio::null())
  //   .status()
  //   .map_err(|e| {
  //     format!(
  //       "Failed to check if current directory is a git repository: {}",
  //       e
  //     )
  //   })
  //   .unwrap()
  //   .success();

  // if !is_in_repo {
  //   return false;
  // }
  let git_dir = Path::new(".git");
  if !git_dir.exists() || !git_dir.is_dir() {
    return false;
  }
  true
}
