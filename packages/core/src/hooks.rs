use crate::command::ShellCommand;
use crate::config;
use crate::config::HookCommand;
use crate::log::LOG_PREFIX;
use fast_glob::glob_match;
use std::error::Error;
use std::fs;
use std::path::Path;
use std::process::Command;

pub fn install() -> Result<(), Box<dyn Error>> {
  let config = config::check()?;
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

  let mut target_hooks = Vec::new();
  if config.commit_msg.enabled {
    target_hooks.push("commit-msg");
  }
  if config.staged_hooks.enabled {
    target_hooks.push("pre-commit");
  }
  if config.hooks.enabled {
    // all hooks
    for hook_name in config.hooks.hooks.keys() {
      if !target_hooks.contains(&hook_name.as_str()) {
        target_hooks.push(hook_name);
      }
    }
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
  for hook_name in target_hooks.iter() {
    let hook_path = hooks_dir.join(hook_name);
    let hook_content = format!(
      r#"#!/usr/bin/env sh
npx igit run "{}" "$@"
"#,
      hook_name
    );
    fs::write(&hook_path, hook_content)?;
    Command::new("chmod").arg("+x").arg(&hook_path).output()?;
  }
  println!("{}Hooks installed", LOG_PREFIX);
  Ok(())
}

#[derive(Debug)]
#[allow(dead_code)]
struct CommitMessage {
  commit_type: String,
  scope: Option<String>,
  is_breaking: bool,
  description: String,
  body: Option<String>,
  footers: Option<Vec<CommitMessageFooter>>,
}

#[derive(Debug)]
#[allow(dead_code)]
struct CommitMessageFooter {
  key: String,
  value: String,
}

static DEFAULT_VALID_TYPES: &[&str] = &[
  "build", "chore", "ci", "docs", "feat", "fix", "perf", "refactor", "revert", "style", "test",
];
static DEFAULT_TYPES_EMOJI: &[&str] = &[
  "📦", "🔧", "👷", "📚", "✨", "🐛", "⚡️", "♻️", "⏪", "🎨", "🚨",
];
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
  original_message.replace(
    &format!("{}:", commit_type_with_scope),
    &format!("{}: {}", commit_type_with_scope, emoji),
  )
}

/**
 * glob match
 */
fn get_matched_files<'a>(pattern: &'a str, files: &Vec<&'a str>) -> Vec<&'a str> {
  let mut matched_files = Vec::new();
  for file in files.iter() {
    if glob_match(&pattern, file) {
      matched_files.push(*file);
    }
  }
  matched_files
}

fn get_commands(command: &HookCommand) -> Vec<&str> {
  match command {
    HookCommand::Single(_command) => vec![_command.as_str()],
    HookCommand::Multiple(_commands) => _commands.iter().map(|s| s.as_str()).collect::<Vec<&str>>(),
  }
}

/**
 * collect staged commands
 */
pub fn collect_staged_commands() -> Result<Vec<ShellCommand>, Box<dyn Error>> {
  let config = config::check()?;
  // no need to collect staged commands
  if !config.staged_hooks.enabled || config.staged_hooks.rules.is_empty() {
    return Ok(vec![]);
  }
  let mut staged_commands: Vec<ShellCommand> = vec![];
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
    let matched_files = get_matched_files(&pattern, &staged_files);
    if !matched_files.is_empty() {
      let commands = get_commands(command);
      for command in commands {
        println!("{}\x1b[90mRunning staged command:\x1b[0m \x1b[32m{}\x1b[0m \x1b[90mfor\x1b[0m \x1b[32m{}\x1b[0m \x1b[90mfiles that match pattern(\x1b[0m\x1b[34m{}\x1b[0m\x1b[90m)\x1b[0m", LOG_PREFIX, command, matched_files.len(), pattern);
        staged_commands.push(ShellCommand::with_args(command.to_string(), matched_files.to_vec()));
      }
    }
  }
  Ok(staged_commands)
}

/**
 * collect hook commands
 */
pub fn collect_hook_commands(hook_name: &str, args: &Vec<String>) -> Result<Vec<ShellCommand>, Box<dyn Error>> {
  let config = config::check()?;
  let mut collected_commands: Vec<ShellCommand> = vec![];
  // commit message hook
  if hook_name == "commit-msg" {
    if config.commit_msg.enabled {
      let commit_message_path = &args[0];
      let mut commit_message_str = fs::read_to_string(commit_message_path).map_err(|e| {
        format!(
          "Failed to read commit message from {}: {}",
          commit_message_path, e
        )
      })?;
      commit_message_str = commit_message_str.trim().to_string();
      let commit_message = parse_commit_message(&commit_message_str)?;
      // is valid commit type
      let valid_types = if config.commit_msg.valid_types.is_some() {
        config.commit_msg.valid_types.unwrap()
      } else {
        // default valid types
        DEFAULT_VALID_TYPES
          .iter()
          .map(|&s| s.to_string())
          .collect::<Vec<String>>()
      };
      if !valid_types.contains(&commit_message.commit_type) {
        return Err(format!("Invalid commit type: {}", commit_message.commit_type).into());
      }
      // prepend emoji
      if config.commit_msg.prepend_emoji {
        commit_message_str = append_emoji_for_message(&commit_message, &mut commit_message_str);
        println!("{}\x1b[32mAppend emoji for commit message.\x1b[0m", LOG_PREFIX);
        fs::write(commit_message_path, commit_message_str)?;
      }
    }
  }
  if let Some(script) = config.hooks.hooks.get(hook_name) {
    // hooks
    if config.hooks.enabled {
      let commands = get_commands(script);
      for command in commands {
        println!(
          "{}\x1b[90mRunning\x1b[0m \x1b[34m{}\x1b[0m \x1b[90mhook:\x1b[0m \x1b[32m{}\x1b[0m",
          LOG_PREFIX, hook_name, command
        );
        collected_commands.push(ShellCommand::with_args(command, args.to_vec()));
      }
    }
  }
  Ok(collected_commands)
}

/* ------------ test ------------ */
#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_color_print() {
    println!("\x1b[32mAppend emoji for message\x1b[0m");
    println!("\x1b[90mRunning staged command:\x1b[0m \x1b[32m{}\x1b[0m \x1b[90mfor\x1b[0m \x1b[32m{}\x1b[0m \x1b[90mfiles that match pattern(\x1b[0m\x1b[34m{}\x1b[0m\x1b[90m)\x1b[0m", "biome check --write", 4, "**/*.{ts,tsx}");
    println!(
      "\x1b[90mRunning\x1b[0m \x1b[34m{}\x1b[0m \x1b[90mhook:\x1b[0m \x1b[32m{}\x1b[0m",
      "pre-commit", "echo hello"
    );
    assert!(true);
  }

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

  #[test]
  fn test_get_matched_files() {
    let files = vec![
      "foo/bar/baz.css",
      "src/main.rs",
      "src/lib.rs",
      "tests/test.rs",
      "README.md",
    ];

    let matched = get_matched_files("**/*.rs", &files);
    assert_eq!(matched.len(), 3);
    assert!(matched.contains(&"src/main.rs"));
    assert!(matched.contains(&"src/lib.rs"));
    assert!(matched.contains(&"tests/test.rs"));

    let matched = get_matched_files("**/*.{rs,md}", &files);
    assert_eq!(matched.len(), 4);
    assert!(matched.contains(&"src/main.rs"));
    assert!(matched.contains(&"src/lib.rs"));
    assert!(matched.contains(&"tests/test.rs"));
    assert!(matched.contains(&"README.md"));

    let matched = get_matched_files("*/*.css", &files);
    assert_eq!(matched.len(), 0);

    let matched = get_matched_files("!**/*.rs", &files);
    assert_eq!(matched.len(), 2);
    assert!(matched.contains(&"foo/bar/baz.css"));
    assert!(matched.contains(&"README.md"));
  }
}
