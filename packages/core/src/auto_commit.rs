use crate::config;
use reqwest::Client;
use serde_json::json;
use spinners::{Spinner, Spinners};
use std::env;
use std::process::Command;

pub async fn generate_commit_message() -> Result<String, String> {
  let config = config::check()?;
  if !config.ai.enabled {
    return Err("Auto commit is not enabled".to_string());
  }
  // 获取git diff内容
  let diff = get_git_diff()?;
  if diff.is_empty() {
    return Err("No changes to commit".to_string());
  }

  let api_key = config
    .ai
    .api_key
    .or_else(|| env::var("OPENAI_API_KEY").ok())
    .ok_or("API key not found")?;
  let base_url = config.ai.base_url.unwrap();
  let model = config.ai.model.unwrap();

  // Build prompt
  let system_prompt = "You are a git commit message generator. Generate a concise, standardized commit message directly with no format wrapper based on the git diff. Focus on the overall purpose or functionality of the changes, rather than listing individual file changes. Follow these guidelines:\n\
    - **Type**: Choose one of the following: build, chore, ci, docs, feat, fix, perf, refactor, revert, style, test.\n\
    - **Scope**: Optional scope of the commit, like 'cli', 'core', 'ui', 'api', etc.\n\
    - **Description**: A short one-line description starting with a present-tense verb, summarizing the overall change.\n\
    - **Body**: Optional detailed explanation, focusing on the 'why' and 'how' of the change, not the 'what'.\n\
    - **Footers**: Optional footers like 'BREAKING CHANGE: `extends` key in config file is now used for extending other config files'.\n\
    \n\
    The commit message should follow this format:\n\
    <type>[optional scope]: <description>\n\
    \n\
    [optional body]\n\
    \n\
    [optional footer(s)]\n\
    \n\
    **Important**:\n\
    - Do not list individual file changes.\n\
    - Focus on the high-level purpose of the commit.\n\
    - Keep the description concise and meaningful.\n\
    - Only include a body or footer if they add significant value.";
  let system_prompt = if let Some(lang) = config.ai.respond_in {
    format!(
      "{}\nPlease always generate the commit message in {} language",
      system_prompt, lang
    )
  } else {
    system_prompt.to_string()
  };

  let messages = vec![
    json!({
        "role": "system",
        "content": system_prompt
    }),
    json!({
        "role": "user",
        "content": diff
    }),
  ];

  let client = Client::new();
  let mut spinner = Spinner::new(Spinners::Dots, "Generating commit message...".into());
  let response = client
    .post(format!("{}/chat/completions", base_url))
    .header("Content-Type", "application/json")
    .header("Authorization", format!("Bearer {}", api_key))
    .json(&json!({
        "model": model,
        "messages": messages,
        "temperature": 0.0
    }))
    .send()
    .await
    .map_err(|e| format!("Failed to send request: {}", e))?;

  let response_json: serde_json::Value = response
    .json()
    .await
    .map_err(|e| format!("Failed to parse response: {}", e))?;

  spinner.stop();
  println!("");

  let content = response_json["choices"][0]["message"]["content"]
    .as_str()
    .ok_or("Invalid response format")
    .map_err(|e| format!("Failed to parse response: {}", e))?
    .to_string();

  if content.is_empty() {
    return Err("No commit message generated".to_string());
  }
  Ok(content)
}

fn get_git_diff() -> Result<String, String> {
  let output = Command::new("git")
    .args(&[
      "diff",
      "--staged",
      "--ignore-all-space",
      "--diff-algorithm=minimal",
      "--function-context",
      "--no-ext-diff",
      "--no-color",
    ])
    .output()
    .map_err(|e| format!("Failed to get git diff: {}", e))?;

  if !output.status.success() {
    return Err("Git diff command failed".to_string());
  }

  Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
