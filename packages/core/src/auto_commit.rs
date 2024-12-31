use std::io::Read;
use std::process::Command;
use serde_json::{json, Value};
use std::env;
use crate::config::IgitConfig;

pub struct CommitMessage {
    pub description: String,
    pub body: Option<String>,
    pub footers: Option<Vec<String>>
}

pub fn generate_commit_message(config: &IgitConfig) -> Result<CommitMessage, String> {
    // 获取git diff内容
    let diff = get_git_diff()?;
    if diff.is_empty() {
        return Err("No changes to commit".to_string());
    }

    // Build prompt
    let mut system_prompt = "You are a git commit message generator. Please generate a standardized commit message based on the git diff, with the following information:\n\
        - type: build, chore, ci, docs, feat, fix, perf, refactor, revert, style, test\n\
        - scope: Optional scope of the commit, like 'cli', 'core', 'ui', 'api', etc.\n\
        - description: A short one-line description starting with a present-tense verb\n\
        - body: Optional detailed explanation\n\
        - footers: Optional footers like 'BREAKING CHANGE: `extends` key in config file is now used for extending other config files'";

    if let Some(lang) = config.ai.respond_in {
        system_prompt = &format!("{}\nPlease always generate the commit message in {} language", system_prompt, lang);
    }

    let messages = vec![
        json!({
            "role": "system",
            "content": system_prompt
        }),
        json!({
            "role": "user",
            "content": diff
        })
    ];

    // 调用API
    let api_key = config.ai.api_key.as_ref()
        .map(|k| k.to_string())
        .or_else(|| env::var("OPENAI_API_KEY").ok())
        .ok_or("API key not found")?;

    let base_url = config.ai.base_url.as_ref()
        .map(|u| u.to_string())
        .unwrap_or("https://api.openai.com".to_string());

    let model = config.ai.model.as_ref()
        .map(|m| m.to_string())
        .unwrap_or("gpt-3.5-turbo".to_string());

    let client = std::net::TcpStream::connect(base_url.replace("https://", ""))?;
    let mut stream = std::io::BufWriter::new(client);

    let request = json!({
        "model": model,
        "messages": messages,
        "temperature": 0.7
    });

    let request_str = format!(
        "POST /v1/chat/completions HTTP/1.1\r\n\
         Host: {}\r\n\
         Content-Type: application/json\r\n\
         Authorization: Bearer {}\r\n\
         Content-Length: {}\r\n\
         \r\n\
         {}",
        base_url.replace("https://", ""),
        api_key,
        request.to_string().len(),
        request.to_string()
    );

    stream.write_all(request_str.as_bytes())?;
    stream.flush()?;

    let mut response = String::new();
    stream.get_mut().read_to_string(&mut response)?;

    // 解析响应
    let response: Value = serde_json::from_str(&response)?;
    let content = response["choices"][0]["message"]["content"]
        .as_str()
        .ok_or("Invalid response format")?;

    // 解析生成的commit message
    let lines: Vec<&str> = content.lines().collect();
    let mut description = String::new();
    let mut body = None;
    let mut footers = None;

    for line in lines {
        if line.starts_with("description:") {
            description = line.replace("description:", "").trim().to_string();
        } else if line.starts_with("body:") {
            body = Some(line.replace("body:", "").trim().to_string());
        } else if line.starts_with("footers:") {
            footers = Some(vec![line.replace("footers:", "").trim().to_string()]);
        }
    }

    Ok(CommitMessage {
        description,
        body,
        footers
    })
}

fn get_git_diff() -> Result<String, String> {
    let output = Command::new("git")
        .args(&["diff",
            "--staged",
            "--ignore-all-space",
            "--diff-algorithm=minimal",
            "--function-context",
            "--no-ext-diff",
            "--no-color"])
        .output()
        .map_err(|e| format!("Failed to get git diff: {}", e))?;

    if !output.status.success() {
        return Err("Git diff command failed".to_string());
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

