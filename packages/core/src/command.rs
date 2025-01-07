use std::fmt::{Display, Formatter, Result};

#[derive(Debug)]
#[napi(object)]
pub struct ShellCommand {
  #[napi(ts_type = "string")]
  pub command: String,
  #[napi(ts_type = "string[]")]
  pub args: Option<Vec<String>>,
}

impl ShellCommand {
  pub fn new<S: Into<String>>(command: S) -> Self {
    Self {
      command: command.into(),
      args: None,
    }
  }

  pub fn with_args<S: Into<String>, I: Into<String>>(command: S, args: Vec<I>) -> Self {
    Self {
      command: command.into(),
      args: Some(args.into_iter().map(|s| s.into()).collect()),
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
