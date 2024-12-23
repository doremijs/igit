#![deny(clippy::all)]

#[macro_use]
extern crate napi_derive;

use napi::{Error, Result};
mod config;
mod git;
mod hooks;
mod command;

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
