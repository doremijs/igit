#![deny(clippy::all)]

#[macro_use]
extern crate napi_derive;

use napi::{Error, Result};
mod config;
mod git;
mod hooks;
mod command;
mod auto_commit;

#[napi]
pub fn init() -> Result<()> {
  config::init().map_err(|e| Error::from_reason(e.to_string()))
}

#[napi]
pub fn install() -> Result<()> {
  hooks::install().map_err(|e| Error::from_reason(e.to_string()))
}

#[napi]
pub fn run_hook(hook_name: String, args: Vec<String>) -> Result<()> {
  hooks::run_hook(&hook_name, &args).map_err(|e| Error::from_reason(e.to_string()))
}

#[napi]
pub async fn auto_commit(commit: bool) -> Result<()> {
  let ret = auto_commit::generate_commit_message(commit).await;
  if let Err(e) = ret {
    eprintln!("{}", e);
  }
  Ok(())
}
