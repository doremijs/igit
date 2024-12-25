use dirs_next::home_dir;
use serde::{Deserialize, Serialize};
use serde_yaml;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum HookCommand {
    Single(String),
    Multiple(Vec<String>),
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct HooksConfig {
  #[serde(default)]
  pub enabled: bool,
  #[serde(default)]
  pub hooks: HashMap<String, HookCommand>,
}

// impl Default for HooksConfig {
//   fn default() -> Self {
//     Self {
//       enabled: false,
//       hooks: HashMap::new(),
//     }
//   }
// }

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct StagedHooksConfig {
  #[serde(default)]
  pub enabled: bool,
  #[serde(default)]
  pub rules: HashMap<String, HookCommand>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct CommitLintConfig {
  #[serde(default)]
  pub enabled: bool,
  #[serde(default, rename = "validTypes", skip_serializing_if = "Option::is_none")]
  pub valid_types: Option<Vec<String>>,
  #[serde(default, rename = "prependEmoji")]
  pub prepend_emoji: bool,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct IgitConfig {
  #[serde(default)]
  pub hooks: HooksConfig,
  #[serde(default)]
  pub staged_hooks: StagedHooksConfig,
  #[serde(default)]
  pub commit_msg: CommitLintConfig,
}

pub fn init() -> std::io::Result<()> {
  let config_file = Path::new(".config/igit.yaml");
  if config_file.exists() {
    return Ok(());
  }
  let config_dir = Path::new(".config");
  if !config_dir.exists() {
    fs::create_dir_all(config_dir)?;
  }

  fs::write(config_file, "# yaml-language-server: $schema=https://igit.erguotou.me/schema/0.0.1/schema.json
hooks:
  enabled: true
  hooks: {}
staged_hooks:
  enabled: true
  rules:
    '**/*.{css,scss,less,styl,stylus}': stylelint --fix
    '**/*.{js,jsx,ts,tsx}': biome check --write
commit_msg:
  enabled: true
  prependEmoji: true
")
}

pub fn parse() -> Result<IgitConfig, String> {
  let mut path: String = ".config/igit.yaml".to_string();
  if !Path::new(&path).exists() {
    path = "igit.yaml".to_string();
    if !Path::new(&path).exists() {
      let mut home_dir = home_dir().unwrap();
      home_dir.push(".config");
      home_dir.push("igit.yaml");
      if !Path::new(&home_dir).exists() {
        return Err(
          "Failed to find config file, please use `igit init` to create one.".to_string(),
        );
      }
      path = home_dir.to_str().unwrap().to_string();
    }
  }
  let ret = fs::read_to_string(path);
  if ret.is_err() {
    return Err("Failed to read config file".to_string());
  }
  let file_content = ret.unwrap();
  let config: IgitConfig = serde_yaml::from_str(&file_content)
    .expect("Failed to parse config, please check your config file.");
  Ok(config)
}
