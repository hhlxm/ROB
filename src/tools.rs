use anyhow::{anyhow, Context, Result};
use serde::Serialize;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use tokio::process::Command;
use tokio::time::{timeout, Duration};

const MAX_OUTPUT_BYTES: usize = 16 * 1024;

#[derive(Debug, Clone, Serialize)]
pub struct ToolSpec {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: ToolFunctionSpec,
}

#[derive(Debug, Clone, Serialize)]
pub struct ToolFunctionSpec {
    pub name: String,
    pub description: String,
    pub parameters: Value,
}

pub fn tool_specs() -> Vec<ToolSpec> {
    vec![
        tool(
            "pwd",
            "Return the current working directory.",
            json!({
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }),
        ),
        tool(
            "list_dir",
            "List files and directories under a path.",
            json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Directory path. Defaults to current directory." }
                },
                "additionalProperties": false
            }),
        ),
        tool(
            "read_file",
            "Read a UTF-8 text file with an output byte limit.",
            json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" },
                    "max_bytes": { "type": "integer", "minimum": 1, "maximum": 65536 }
                },
                "required": ["path"],
                "additionalProperties": false
            }),
        ),
        tool(
            "search_text",
            "Search for text with ripgrep when available.",
            json!({
                "type": "object",
                "properties": {
                    "pattern": { "type": "string" },
                    "path": { "type": "string", "description": "File or directory path. Defaults to current directory." },
                    "max_results": { "type": "integer", "minimum": 1, "maximum": 200 }
                },
                "required": ["pattern"],
                "additionalProperties": false
            }),
        ),
        tool(
            "shell_exec",
            "Run a small allowlisted Linux command without invoking a shell.",
            json!({
                "type": "object",
                "properties": {
                    "command": { "type": "string" },
                    "args": { "type": "array", "items": { "type": "string" } },
                    "timeout_ms": { "type": "integer", "minimum": 100, "maximum": 10000 }
                },
                "required": ["command"],
                "additionalProperties": false
            }),
        ),
    ]
}

pub async fn run_tool(name: &str, args: Value) -> Result<String> {
    match name {
        "pwd" => Ok(std::env::current_dir()?.display().to_string()),
        "list_dir" => list_dir(args),
        "read_file" => read_file(args),
        "search_text" => search_text(args).await,
        "shell_exec" => shell_exec(args).await,
        _ => Err(anyhow!("unknown tool `{name}`")),
    }
}

fn tool(name: &str, description: &str, parameters: Value) -> ToolSpec {
    ToolSpec {
        tool_type: "function".to_string(),
        function: ToolFunctionSpec {
            name: name.to_string(),
            description: description.to_string(),
            parameters,
        },
    }
}

fn list_dir(args: Value) -> Result<String> {
    let path = string_arg(&args, "path").unwrap_or_else(|| ".".to_string());
    let mut entries = Vec::new();
    for entry in std::fs::read_dir(&path).with_context(|| format!("failed to read dir {path}"))? {
        let entry = entry?;
        let metadata = entry.metadata()?;
        let kind = if metadata.is_dir() { "dir" } else { "file" };
        entries.push(format!("{kind}\t{}", entry.file_name().to_string_lossy()));
    }
    entries.sort();
    Ok(truncate(entries.join("\n"), MAX_OUTPUT_BYTES))
}

fn read_file(args: Value) -> Result<String> {
    let path = required_string_arg(&args, "path")?;
    let max_bytes = usize_arg(&args, "max_bytes").unwrap_or(MAX_OUTPUT_BYTES);
    let raw =
        std::fs::read_to_string(&path).with_context(|| format!("failed to read file {path}"))?;
    Ok(truncate(raw, max_bytes))
}

async fn search_text(args: Value) -> Result<String> {
    let pattern = required_string_arg(&args, "pattern")?;
    let path = string_arg(&args, "path").unwrap_or_else(|| ".".to_string());
    let max_results = usize_arg(&args, "max_results").unwrap_or(50).min(200);

    if command_exists("rg").await {
        let output = Command::new("rg")
            .arg("--line-number")
            .arg("--color=never")
            .arg("--max-count")
            .arg(max_results.to_string())
            .arg(&pattern)
            .arg(&path)
            .output()
            .await
            .context("failed to run rg")?;
        return command_output(output);
    }

    let mut matches = Vec::new();
    search_text_fallback(Path::new(&path), &pattern, max_results, &mut matches)?;
    Ok(truncate(matches.join("\n"), MAX_OUTPUT_BYTES))
}

async fn shell_exec(args: Value) -> Result<String> {
    let command = required_string_arg(&args, "command")?;
    let argv = string_array_arg(&args, "args")?;
    let timeout_ms = usize_arg(&args, "timeout_ms")
        .unwrap_or(3000)
        .clamp(100, 10000);

    let allowed = [
        "pwd", "ls", "cat", "head", "tail", "wc", "rg", "find", "date", "uname", "whoami", "df",
        "du", "ps", "env",
    ];
    if !allowed.contains(&command.as_str()) {
        return Err(anyhow!("command `{command}` is not in the allowlist"));
    }

    let child = Command::new(&command).args(argv).output();
    let output = timeout(Duration::from_millis(timeout_ms as u64), child)
        .await
        .context("command timed out")?
        .with_context(|| format!("failed to run {command}"))?;
    command_output(output)
}

fn command_output(output: std::process::Output) -> Result<String> {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let status = output.status.code().unwrap_or(-1);
    Ok(truncate(
        format!("exit_code: {status}\nstdout:\n{stdout}\nstderr:\n{stderr}"),
        MAX_OUTPUT_BYTES,
    ))
}

async fn command_exists(command: &str) -> bool {
    Command::new(command)
        .arg("--version")
        .output()
        .await
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn search_text_fallback(
    path: &Path,
    pattern: &str,
    max_results: usize,
    matches: &mut Vec<String>,
) -> Result<()> {
    if matches.len() >= max_results {
        return Ok(());
    }
    if path.is_file() {
        search_file(path, pattern, max_results, matches)?;
        return Ok(());
    }
    if !path.is_dir() {
        return Ok(());
    }

    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let child = entry.path();
        if should_skip(&child) {
            continue;
        }
        if child.is_dir() {
            search_text_fallback(&child, pattern, max_results, matches)?;
        } else {
            search_file(&child, pattern, max_results, matches)?;
        }
        if matches.len() >= max_results {
            break;
        }
    }
    Ok(())
}

fn search_file(
    path: &Path,
    pattern: &str,
    max_results: usize,
    matches: &mut Vec<String>,
) -> Result<()> {
    let Ok(content) = std::fs::read_to_string(path) else {
        return Ok(());
    };
    for (index, line) in content.lines().enumerate() {
        if line.contains(pattern) {
            matches.push(format!("{}:{}:{}", path.display(), index + 1, line));
            if matches.len() >= max_results {
                break;
            }
        }
    }
    Ok(())
}

fn should_skip(path: &PathBuf) -> bool {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    matches!(name, ".git" | "target" | "node_modules" | ".gradle")
}

fn string_arg(args: &Value, key: &str) -> Option<String> {
    args.get(key)?.as_str().map(ToString::to_string)
}

fn required_string_arg(args: &Value, key: &str) -> Result<String> {
    string_arg(args, key).ok_or_else(|| anyhow!("missing string argument `{key}`"))
}

fn usize_arg(args: &Value, key: &str) -> Option<usize> {
    args.get(key)?.as_u64().map(|value| value as usize)
}

fn string_array_arg(args: &Value, key: &str) -> Result<Vec<String>> {
    let Some(value) = args.get(key) else {
        return Ok(Vec::new());
    };
    let array = value
        .as_array()
        .ok_or_else(|| anyhow!("argument `{key}` must be an array"))?;
    array
        .iter()
        .map(|item| {
            item.as_str()
                .map(ToString::to_string)
                .ok_or_else(|| anyhow!("argument `{key}` items must be strings"))
        })
        .collect()
}

fn truncate(mut value: String, max_bytes: usize) -> String {
    if value.len() <= max_bytes {
        return value;
    }
    value.truncate(max_bytes);
    value.push_str("\n[truncated]");
    value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn shell_exec_rejects_non_allowlisted_command() {
        let result = run_tool(
            "shell_exec",
            json!({
                "command": "sh",
                "args": ["-c", "echo no"]
            }),
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn pwd_tool_returns_current_directory() {
        let result = run_tool("pwd", json!({})).await.unwrap();

        assert!(result.contains("ROB"));
    }
}
