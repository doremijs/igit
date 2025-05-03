use std::fmt::{Display, Formatter, Result};

#[derive(Debug)]
#[napi(object)]
pub struct ShellCommand {
  #[napi(ts_type = "string")]
  pub command: String,
  #[napi(ts_type = "string[]")]
  pub args: Option<Vec<String>>,
  #[napi(ts_type = "string[]")]
  pub files: Option<Vec<String>>,
}

impl ShellCommand {
  pub fn new<S: Into<String>>(command: S) -> Self {
    Self {
      command: command.into(),
      args: None,
      files: None
    }
  }

  pub fn with_args<S: Into<String>, I: Into<String>>(command: S, args: Vec<I>) -> Self {
    Self {
      command: command.into(),
      args: Some(args.into_iter().map(|s| s.into()).collect()),
      files: None,
    }
  }

  pub fn with_args_and_files<S: Into<String>, I: Into<String> + std::fmt::Display>(command: S, args: &[I], files: &[I]) -> Self {
    Self {
      command: command.into(),
      args: Some(args.iter().map(|s| s.to_string()).collect()),
      files: Some(files.iter().map(|s| s.to_string()).collect()),
    }
  }
}

impl Display for ShellCommand {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result {
    match &self.args {
      Some(args) => write!(f, "{} {}", self.command, args.join(" ")),
      None => write!(f, "{}", self.command),
    }
  }
}
