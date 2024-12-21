use dirs_next::home_dir;
use serde::{Deserialize, Serialize};
use serde_yaml;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug)]
pub struct HooksConfig {
  pub enabled: bool,
  pub hooks: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StagedHooksConfig {
  pub enabled: bool,
  pub rules: std::collections::HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CommitLintConfig {
  pub enabled: bool,
  #[serde(rename = "valid_types")]
  pub valid_types: Option<Vec<String>>,
  #[serde(rename = "prependEmoji")]
  pub prepend_emoji: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct IgitConfig {
  pub hooks: HooksConfig,
  pub staged_hooks: StagedHooksConfig,
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

  let config = IgitConfig {
    hooks: HooksConfig {
      enabled: true,
      hooks: HashMap::new(),
    },
    staged_hooks: StagedHooksConfig {
      enabled: true,
      rules: [
        (
          "*.{js,jsx,ts,tsx}".to_string(),
          "biome check --write".to_string(),
        ),
        (
          "*.{css,scss,less,styl,stylus}".to_string(),
          "stylelint --fix".to_string(),
        ),
      ]
      .iter()
      .cloned()
      .collect(),
    },
    commit_msg: CommitLintConfig {
      enabled: true,
      valid_types: None,
      prepend_emoji: true,
    },
  };

  let mut yaml = serde_yaml::to_string(&config).unwrap();
  yaml = format!(
    "# yaml-language-server: $schema=https://igit.erguotou.me/schema/0.0.1/schema.json\n{}",
    yaml
  );

  fs::write(config_file, yaml)
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
