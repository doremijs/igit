#![deny(clippy::all)]

#[macro_use]
extern crate napi_derive;

use command::ShellCommand;
use napi::{Error, Result};
mod config;
mod git;
mod hooks;
mod command;
mod auto_commit;
mod log;

#[napi]
pub fn init() -> Result<()> {
  config::init().map_err(|e| Error::from_reason(e.to_string()))
}

#[napi]
pub fn install() -> Result<()> {
  hooks::install().map_err(|e| Error::from_reason(e.to_string()))
}

#[napi]
pub fn collect_staged_commands() -> Result<Vec<ShellCommand>> {
  hooks::collect_staged_commands().map_err(|e| Error::from_reason(e.to_string()))
}

#[napi]
pub fn collect_hook_commands(hook_name: String, args: Vec<String>) -> Result<Vec<ShellCommand>> {
  hooks::collect_hook_commands(&hook_name, &args).map_err(|e| Error::from_reason(e.to_string()))
}

#[napi]
pub async fn auto_commit() -> Result<String> {
  auto_commit::generate_commit_message().await.map_err(|e| Error::from_reason(e))
}
