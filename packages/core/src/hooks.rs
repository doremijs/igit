use crate::{
  config::{parse, IgitConfig},
  git::{is_git_installed, is_git_repo},
};
use std::error::Error;
use std::fs;
use std::path::Path;
use std::process::Command;
use fast_glob::glob_match;

pub fn install() -> Result<(), Box<dyn Error>> {
  let git_exists = is_git_installed();

  if !git_exists {
    return Err("Git is not installed".into());
  }

  let is_git_repo = is_git_repo();

  if !is_git_repo {
    return Err("Current directory is not a git repository".into());
  }
  let config = parse()?;
  let git_config_output = Command::new("git")
    .arg("config")
    .arg("--get")
    .arg("core.hooksPath")
    .output()
    .expect("Failed to get git hooks path");

  let git_hooks_path = if git_config_output.status.success() {
    String::from_utf8(git_config_output.stdout)?
      .trim()
      .to_string()
  } else {
    String::from(".git/hooks")
  };

  let hooks_dir = Path::new(&git_hooks_path);
  if !hooks_dir.exists() {
    fs::create_dir_all(hooks_dir)?;
  }

  if config.hooks.enabled {
    // all hooks
    let mut hooks = config.hooks.hooks.keys().map(|s| s.as_str()).collect::<Vec<&str>>();
    if config.commit_msg.enabled && !hooks.contains(&"commit-msg") {
      hooks.push("commit-msg");
    }
    if config.staged_hooks.enabled && !hooks.contains(&"pre-commit") {
      hooks.push("pre-commit");
    }
    // clear hooks
    let entries = fs::read_dir(hooks_dir)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().is_none() {
            fs::remove_file(path)?;
        }
    }

    // generate hooks
    for (hook_name, script) in config.hooks.hooks.iter() {
      let hook_path = hooks_dir.join(hook_name);
      let hook_content = format!(
        r#"#!/bin/sh
run_command(entry, command, args) {{
    if command -v bun > /dev/null 2>&1; then
        bun $entry $command $args
    elif command -v deno > /dev/null 2>&1; then
        deno run $entry $command $args
    else
        node $entry $command $args
    fi
}}

run_command ../cli/bin/index.mjs run "{}" "$@"
"#,
        script
      );
      fs::write(&hook_path, hook_content)?;
      Command::new("chmod").arg("+x").arg(&hook_path).output()?;
    }
  }

  Ok(())
}

#[derive(Debug)]
struct CommitMessage {
  commit_type: String,
  #[warn(dead_code)]
  scope: Option<String>,
  #[warn(dead_code)]
  is_breaking: bool,
  #[warn(dead_code)]
  description: String,
  #[warn(dead_code)]
  body: Option<String>,
  #[warn(dead_code)]
  footers: Option<Vec<CommitMessageFooter>>,
}

#[derive(Debug)]
struct CommitMessageFooter {
  key: String,
  #[warn(dead_code)]
  value: String,
}

static DEFAULT_VALID_TYPES: &[&str] = &["build", "chore", "ci", "docs", "feat", "fix", "perf", "refactor", "revert", "style", "test"];
static DEFAULT_TYPES_EMOJI: &[&str] = &["📦", "🔧", "👷", "📚", "✨", "🐛", "⚡️", "♻️", "⏪", "🎨", "🚨"];
static UNKNOWN_TYPE_EMOJI: &str = "💡";
// static UNKNOWN_TYPE_EMOJI_CODE: &str = ":bulb:";
// static DEFAULT_TYPES_EMOJI_CODE: &[&str] = &[":package:", ":wrench:", ":construction_worker:", ":books:", ":sparkles:", ":bug:", ":zap:", ":recycle:", ":rewind:", ":art:", ":rotating_light:"];
// static BREAKING_CHANGE_EMOJI: &str = "💥";
// static BREAKING_CHANGE_EMOJI_CODE: &str = ":boom:";

/**
* commit message lint
* <type>[optional scope]: <description>

* [optional body]
*
* [optional footer(s)]
*/
fn parse_commit_message(commit_message: &str) -> Result<CommitMessage, String> {
  let lines: Vec<&str> = commit_message.lines().collect();
  if lines.is_empty() {
    return Err("Commit message is empty".to_string());
  }

  let first_line = lines[0];
  let mut parts = first_line.splitn(3, ": ");
  let mut commit_type = parts.next().ok_or("Invalid commit type")?.to_string();
  let description = parts.next().ok_or("Invalid description")?.to_string();

  let mut scope = None;
  let mut is_breaking = false;

  if commit_type.ends_with('!') {
    is_breaking = true;
    commit_type = commit_type.trim_end_matches('!').to_string();
  }

  if let Some(scope_start) = commit_type.find('(') {
    if let Some(scope_end) = commit_type.find(')') {
      scope = Some(commit_type[scope_start + 1..scope_end].to_string());
      commit_type = commit_type[..scope_start].to_string();
    }
  }

  let mut body = None;
  let mut footers = Vec::new();

  if lines.len() > 1 {
    let mut body_lines = Vec::new();
    let mut footer_start_index = lines.len();

    for i in (1..lines.len()).rev() {
      let line = lines[i];
      if let Some(colon_pos) = line.find(':') {
        if colon_pos != 0 {
          let key = line[..colon_pos].trim().to_string();
          let value = line[colon_pos + 1..].trim().to_string();
          let _ = &footers.push(CommitMessageFooter { key, value });
          footer_start_index = i;
        } else {
          break;
        }
      } else {
        break;
      }
    }

    for line in &lines[1..footer_start_index] {
      body_lines.push(*line);
    }

    if !body_lines.is_empty() {
      let _body = body_lines.join("\n").trim().to_string();
      body = if _body.is_empty() { None } else { Some(_body) };
    }

    for line in &footers {
      if line.key == "BREAKING CHANGE" {
        is_breaking = true;
      }
    }
  }

  Ok(CommitMessage {
    commit_type,
    scope,
    is_breaking,
    description,
    body,
    footers: if footers.is_empty() {
      None
    } else {
      Some(footers)
    },
  })
}

/**
 * append emoji for commit message
 */
fn append_emoji_for_message(message: &CommitMessage, original_message: &str) -> String {
  let commit_type = message.commit_type.to_string();
  let index = DEFAULT_VALID_TYPES.iter().position(|&t| t == commit_type);
  let emoji = if let Some(index) = index {
    DEFAULT_TYPES_EMOJI[index]
  } else {
    UNKNOWN_TYPE_EMOJI
  };
  let mut commit_type_with_scope = format!("{}", message.commit_type);
  if let Some(scope) = &message.scope {
    commit_type_with_scope = format!("{}({})", commit_type_with_scope, scope);
  }
  let ret = original_message.replace(&format!("{}:", commit_type_with_scope), &format!("{}: {}", commit_type_with_scope, emoji));
  println!("{}", ret);
  ret
}

/**
 * run commands for staged files
 */
fn run_commands_for_staged_files(config: &IgitConfig) -> Result<(), Box<dyn Error>> {
  let staged_files = Command::new("git")
    .arg("diff")
    .arg("--cached")
    .arg("--name-only")
    .output()?;
  if !staged_files.status.success() {
    return Err("Failed to get staged files".into());
  }
  let files = String::from_utf8(staged_files.stdout)?;
  let staged_files = files.trim().split('\n').collect::<Vec<&str>>();
  for (pattern, command) in config.staged_hooks.rules.iter() {
    let mut matched_files = Vec::new();
    for file in staged_files.iter() {
      if glob_match(&pattern, file) {
        matched_files.push(*file);
      }
    }
    if !matched_files.is_empty() {
      Command::new("sh")
        .arg("-c")
        .arg(format!("{} {}", command, matched_files.join(" ")))
        .output()?;
    }
  }
  Ok(())
}

/**
 * run hooks
 */
pub fn run_hook(hook_name: &str, args: &Vec<String>) -> Result<(), Box<dyn Error>> {
  let config = parse()?;
  if let Some(script) = config.hooks.hooks.get(hook_name) {
    // pre-commit hook
    if hook_name == "pre-commit" {
      if config.staged_hooks.enabled {
        if config.staged_hooks.rules.keys().len() > 0 {
          run_commands_for_staged_files(&config)?;
        }
      }
    }
    // commit message hook
    else if hook_name == "commit-msg" {
      if config.commit_msg.enabled {
        let commit_message_path = &args[0];
        let mut commit_message_str = fs::read_to_string(commit_message_path)
            .map_err(|e| format!("Failed to read commit message from {}: {}", commit_message_path, e))?;
        commit_message_str = commit_message_str.trim().to_string();
        let commit_message = parse_commit_message(&commit_message_str)?;
        // is valid commit type
        let valid_types = if config.commit_msg.valid_types.is_some() {
          config.commit_msg.valid_types.unwrap()
        } else {
          // default valid types
          DEFAULT_VALID_TYPES.iter().map(|&s| s.to_string()).collect::<Vec<String>>()
        };
        if !valid_types.contains(&commit_message.commit_type) {
          return Err(format!("Invalid commit type: {}", commit_message.commit_type).into());
        }
        // prepend emoji
        if config.commit_msg.prepend_emoji {
          commit_message_str = append_emoji_for_message(&commit_message, &mut commit_message_str);
          fs::write(commit_message_path, commit_message_str)?;
        }
      }

    }
    // hooks
    if config.hooks.enabled {
      let output: std::process::Output = Command::new("sh")
        .arg("-c")
        .arg(format!("{} {}", script, args.join(" ")))
        .output()?;
      if !output.status.success() {
        return Err(
          format!(
            "Hook {} failed with exit code {}",
            hook_name,
            output.status.code().unwrap_or(-1)
          )
          .into(),
        );
      }
    }
  }
  Ok(())
}

/* ------------ test ------------ */
#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_commit_message_with_description_and_breaking_change_footer() {
    let commit_message = "feat: allow provided config object to extend other configs\n\nBREAKING CHANGE: `extends` key in config file is now used for extending other config files";
    let result = parse_commit_message(commit_message);
    assert!(result.is_ok());
    let parsed = result.unwrap();
    assert_eq!(parsed.commit_type, "feat");
    assert_eq!(parsed.scope, None);
    assert!(parsed.is_breaking);
    assert_eq!(
      parsed.description,
      "allow provided config object to extend other configs"
    );
    assert_eq!(parsed.body, None);
    assert_eq!(parsed.footers.unwrap().len(), 1);
  }

  #[test]
  fn test_commit_message_with_bang_for_breaking_change() {
    let commit_message = "feat!: send an email to the customer when a product is shipped";
    let result = parse_commit_message(commit_message);
    assert!(result.is_ok());
    let parsed = result.unwrap();
    assert_eq!(parsed.commit_type, "feat");
    assert_eq!(parsed.scope, None);
    assert!(parsed.is_breaking);
    assert_eq!(
      parsed.description,
      "send an email to the customer when a product is shipped"
    );
    assert_eq!(parsed.body, None);
    assert!(parsed.footers.is_none());
  }

  #[test]
  fn test_commit_message_with_scope_and_bang_for_breaking_change() {
    let commit_message = "feat(api)!: send an email to the customer when a product is shipped";
    let result = parse_commit_message(commit_message);
    assert!(result.is_ok());
    let parsed = result.unwrap();
    assert_eq!(parsed.commit_type, "feat");
    assert_eq!(parsed.scope.unwrap(), "api");
    assert!(parsed.is_breaking);
    assert_eq!(
      parsed.description,
      "send an email to the customer when a product is shipped"
    );
    assert_eq!(parsed.body, None);
    assert!(parsed.footers.is_none());
  }

  #[test]
  fn test_commit_message_with_both_bang_and_breaking_change_footer() {
    let commit_message = "chore!: drop support for Node 6\n\nBREAKING CHANGE: use JavaScript features not available in Node 6.";
    let result = parse_commit_message(commit_message);
    assert!(result.is_ok());
    let parsed = result.unwrap();
    assert_eq!(parsed.commit_type, "chore");
    assert_eq!(parsed.scope, None);
    assert!(parsed.is_breaking);
    assert_eq!(parsed.description, "drop support for Node 6");
    assert_eq!(parsed.body, None);
    assert_eq!(parsed.footers.unwrap().len(), 1);
  }

  #[test]
  fn test_commit_message_with_no_body() {
    let commit_message = "docs: correct spelling of CHANGELOG";
    let result = parse_commit_message(commit_message);
    assert!(result.is_ok());
    let parsed = result.unwrap();
    assert_eq!(parsed.commit_type, "docs");
    assert_eq!(parsed.scope, None);
    assert!(!parsed.is_breaking);
    assert_eq!(parsed.description, "correct spelling of CHANGELOG");
    assert_eq!(parsed.body, None);
    assert!(parsed.footers.is_none());
  }

  #[test]
  fn test_commit_message_with_scope() {
    let commit_message = "feat(lang): add Polish language";
    let result = parse_commit_message(commit_message);
    assert!(result.is_ok());
    let parsed = result.unwrap();
    assert_eq!(parsed.commit_type, "feat");
    assert_eq!(parsed.scope.unwrap(), "lang");
    assert!(!parsed.is_breaking);
    assert_eq!(parsed.description, "add Polish language");
    assert_eq!(parsed.body, None);
    assert!(parsed.footers.is_none());
  }

  #[test]
  fn test_commit_message_with_multi_paragraph_body_and_multiple_footers() {
    let commit_message = "fix: prevent racing of requests\n\nIntroduce a request id and a reference to latest request. Dismiss\nincoming responses other than from latest request.\n\nRemove timeouts which were used to mitigate the racing issue but are\nobsolete now.\n\nReviewed-by: Z\nRefs: #123";
    let result = parse_commit_message(commit_message);
    assert!(result.is_ok());
    let parsed = result.unwrap();
    assert_eq!(parsed.commit_type, "fix");
    assert_eq!(parsed.scope, None);
    assert!(!parsed.is_breaking);
    assert_eq!(parsed.description, "prevent racing of requests");
    println!("{:?}", parsed.body);
    println!("{:?}", parsed.footers);
    assert_eq!(parsed.body.unwrap(), "Introduce a request id and a reference to latest request. Dismiss\nincoming responses other than from latest request.\n\nRemove timeouts which were used to mitigate the racing issue but are\nobsolete now.");
    assert_eq!(parsed.footers.unwrap().len(), 2);
  }

  #[test]
  fn test_append_emoji_for_message_with_valid_types() {
    let message_str = "feat: add new feature";
    let message = parse_commit_message(message_str).unwrap();
    let result = append_emoji_for_message(&message, &message_str);
    assert_eq!(result, "feat: ✨ add new feature");

    let message1_str = "fix: resolve bug";
    let message1 = parse_commit_message(message1_str).unwrap();
    let result1 = append_emoji_for_message(&message1, &message1_str);
    assert_eq!(result1, "fix: 🐛 resolve bug");
  }

  #[test]
  fn test_append_emoji_for_message_with_unknown_types() {
    let message_str = "unknown: update documentation";
    let message = parse_commit_message(message_str).unwrap();
    let result = append_emoji_for_message(&message, &message_str);
    assert_eq!(result, "unknown: 💡 update documentation");
  }
}
