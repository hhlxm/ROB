use anyhow::{anyhow, Context, Result};
use serde::Serialize;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use tokio::process::Command;
use tokio::time::{timeout, Duration};

const MAX_OUTPUT_BYTES: usize = 16 * 1024;
const TOOL_TITLE_FIELD: &str = "tool_title";
const HOME_FLOORS: &[&str] = &[
    "一楼",
    "二楼",
    "三楼",
    "四楼",
    "五楼",
    "阁楼",
    "地下室",
    "夹层",
    "屋顶花园",
    "楼顶",
    "顶楼",
    "负一楼",
    "地下一层",
];
const HOME_ROOMS: &[&str] = &[
    "客厅",
    "餐厅",
    "主卧",
    "次卧",
    "儿童房",
    "长辈房",
    "书房",
    "厨房",
    "卫生间",
    "阳台",
    "衣帽间",
    "玄关",
    "走廊",
    "储藏室",
    "车库",
    "健身房",
    "影音室",
    "娱乐室",
    "洗衣房",
    "茶室",
    "棋牌室",
    "瑜伽室",
    "宠物房",
    "停车场",
    "电竞房",
    "全屋",
];
const LIGHT_DEVICE_NAMES: &[&str] = &[
    "灯",
    "主灯",
    "吸顶灯",
    "筒灯",
    "射灯",
    "灯带",
    "彩光灯带",
    "彩光灯",
    "客厅灯",
    "卧室灯",
    "灯泡",
    "球泡灯",
    "色温灯",
    "调光灯",
    "床头灯",
    "台灯",
    "夜灯",
];
const CURTAIN_DEVICE_NAMES: &[&str] = &[
    "窗帘",
    "窗帘电机",
    "智能窗帘",
    "智能窗帘电机",
    "开窗器",
    "卷帘",
    "卷帘电机",
    "开合帘",
    "百叶窗",
    "客厅窗帘",
    "卧室窗帘",
    "窗帘伴侣",
    "梦幻帘",
];
const POWER_DEVICE_NAMES: &[&str] = &[
    "插座",
    "智能插座",
    "插头",
    "开关插座",
    "计量插座",
    "墙壁插座",
    "开关",
    "一路开关",
    "二路开关",
    "三路开关",
    "一键开关",
    "二键开关",
    "三键开关",
    "单开",
    "双开",
    "三开",
    "墙壁开关",
    "零火开关",
    "单火开关",
    "场景开关",
    "无线开关",
    "通断器",
];
const SCENE_NAMES: &[&str] = &[
    "回家模式",
    "离家模式",
    "睡眠模式",
    "起床模式",
    "电影模式",
    "用餐模式",
    "会客模式",
    "阅读模式",
    "浪漫模式",
    "派对模式",
    "节能模式",
    "安防模式",
    "通风模式",
    "度假模式",
    "宠物模式",
];

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
            "Run a small allowlisted Linux command without invoking a shell. Always provide both `command` and `args`; use `args: []` when the command has no arguments. For system configuration, prefer concrete commands such as uname, whoami, env, df, ps, or ls.",
            json!({
                "type": "object",
                "properties": {
                    "command": {
                        "type": "string",
                        "enum": ["pwd", "ls", "cat", "head", "tail", "wc", "rg", "find", "date", "uname", "whoami", "df", "du", "ps", "env"]
                    },
                    "args": { "type": "array", "items": { "type": "string" }, "default": [] },
                    "timeout_ms": { "type": "integer", "minimum": 100, "maximum": 10000 }
                },
                "required": ["command", "args"],
                "additionalProperties": false
            }),
        ),
        tool(
            "smart_home_control_speaker",
            "Submit a normalized smart-home command for speaker volume control. Use this for volume up/down, mute, or setting an exact volume.",
            json!({
                "type": "object",
                "properties": {
                    "floor": { "type": "string", "enum": HOME_FLOORS, "description": "楼层。用户未说明时省略。" },
                    "room": { "type": "string", "enum": HOME_ROOMS, "description": "房间。全屋控制时使用“全屋”；用户未说明时省略。" },
                    "device_name": { "type": "string", "description": "设备名称或别名，例如“扬声器”“音箱”“NAS”。用户未说明时可省略。" },
                    "action": {
                        "type": "string",
                        "enum": ["increase_volume", "decrease_volume", "mute", "set_volume"],
                        "description": "音量调大、调小、静音或设置指定音量。"
                    },
                    "delta_percent": { "type": "integer", "minimum": 0, "maximum": 100, "description": "相对调节百分比。普通调大/调小默认填 10。" },
                    "volume_percent": { "type": "integer", "minimum": 0, "maximum": 100, "description": "目标音量百分比。静音时填 0；设置指定音量时必填。" }
                },
                "required": ["action"],
                "additionalProperties": false
            }),
        ),
        tool(
            "smart_home_control_light",
            "Submit a normalized smart-home command for light control, including power, brightness, color temperature, tone, and RGB color.",
            json!({
                "type": "object",
                "properties": {
                    "floor": { "type": "string", "enum": HOME_FLOORS, "description": "楼层。用户未说明时省略。" },
                    "room": { "type": "string", "enum": HOME_ROOMS, "description": "房间。全屋控制时使用“全屋”；用户未说明时省略。" },
                    "device_name": { "type": "string", "enum": LIGHT_DEVICE_NAMES, "description": "灯具名称，例如主灯、吸顶灯、筒灯、灯带、床头灯、台灯、夜灯。" },
                    "action": {
                        "type": "string",
                        "enum": [
                            "turn_on",
                            "turn_off",
                            "increase_brightness",
                            "decrease_brightness",
                            "set_brightness",
                            "increase_color_temperature",
                            "decrease_color_temperature",
                            "set_color_temperature",
                            "set_light_tone",
                            "set_color"
                        ],
                        "description": "灯光开关、亮度、色温、光色或颜色控制动作。"
                    },
                    "delta_percent": { "type": "integer", "minimum": 0, "maximum": 100, "description": "亮度相对调节百分比。普通调亮/调暗默认填 20。" },
                    "brightness_percent": { "type": "integer", "minimum": 0, "maximum": 100, "description": "目标亮度百分比。设置指定亮度时必填。" },
                    "delta_kelvin": { "type": "integer", "minimum": 0, "maximum": 6000, "description": "色温相对调节值。普通调冷/调暖默认填 500。" },
                    "color_temperature_kelvin": { "type": "integer", "minimum": 0, "maximum": 6000, "description": "目标色温 K。暖光 3000，中性光/自然光 4000，白光/冷光 6000。" },
                    "light_tone": {
                        "type": "string",
                        "enum": ["warm", "neutral", "natural", "white", "cool"],
                        "description": "用户要求调成暖光、中性光、自然光、白光或冷光时使用。"
                    },
                    "color_name": {
                        "type": "string",
                        "enum": ["红色", "橙色", "黄色", "绿色", "青色", "蓝色", "紫色"],
                        "description": "目标颜色名称。"
                    },
                    "rgb": {
                        "type": "object",
                        "properties": {
                            "r": { "type": "integer", "minimum": 0, "maximum": 255 },
                            "g": { "type": "integer", "minimum": 0, "maximum": 255 },
                            "b": { "type": "integer", "minimum": 0, "maximum": 255 }
                        },
                        "required": ["r", "g", "b"],
                        "additionalProperties": false,
                        "description": "目标 RGB 颜色值。"
                    }
                },
                "required": ["action"],
                "additionalProperties": false
            }),
        ),
        tool(
            "smart_home_control_curtain",
            "Submit a normalized smart-home command for curtains, blinds, rollers, and openers.",
            json!({
                "type": "object",
                "properties": {
                    "floor": { "type": "string", "enum": HOME_FLOORS, "description": "楼层。用户未说明时省略。" },
                    "room": { "type": "string", "enum": HOME_ROOMS, "description": "房间。全屋控制时使用“全屋”；用户未说明时省略。" },
                    "device_name": { "type": "string", "enum": CURTAIN_DEVICE_NAMES, "description": "窗帘设备名称，例如窗帘、卷帘、百叶窗、梦幻帘。" },
                    "action": {
                        "type": "string",
                        "enum": ["open", "close", "stop", "set_position", "set_angle"],
                        "description": "打开、关闭、停止、设置开合度或设置百叶窗角度。"
                    },
                    "position_percent": { "type": "integer", "minimum": 0, "maximum": 100, "description": "窗帘开合度百分比。设置开合度时必填。" },
                    "angle_degree": { "type": "integer", "minimum": 0, "maximum": 180, "description": "百叶窗角度。设置角度时必填。" }
                },
                "required": ["action"],
                "additionalProperties": false
            }),
        ),
        tool(
            "smart_home_control_power",
            "Submit a normalized smart-home command for outlets and wall switches.",
            json!({
                "type": "object",
                "properties": {
                    "floor": { "type": "string", "enum": HOME_FLOORS, "description": "楼层。用户未说明时省略。" },
                    "room": { "type": "string", "enum": HOME_ROOMS, "description": "房间。全屋控制时使用“全屋”；用户未说明时省略。" },
                    "device_category": {
                        "type": "string",
                        "enum": ["outlet", "wall_switch"],
                        "description": "插座/智能插头使用 outlet；墙壁开关/一路开关/双开/通断器等使用 wall_switch。"
                    },
                    "device_name": {
                        "type": "string",
                        "enum": POWER_DEVICE_NAMES,
                        "description": "设备名称，例如智能插座、计量插座、一路开关、双开、墙壁开关。"
                    },
                    "action": {
                        "type": "string",
                        "enum": ["turn_on", "turn_off"],
                        "description": "打开或关闭。"
                    }
                },
                "required": ["device_category", "action"],
                "additionalProperties": false
            }),
        ),
        tool(
            "smart_home_control_scene",
            "Submit a normalized smart-home command to activate or deactivate a predefined home scene mode.",
            json!({
                "type": "object",
                "properties": {
                    "scene_name": {
                        "type": "string",
                        "enum": SCENE_NAMES,
                        "description": "场景模式名称。"
                    },
                    "action": {
                        "type": "string",
                        "enum": ["activate", "deactivate"],
                        "description": "打开/设置/执行/启动场景使用 activate；关闭/退出/停止场景使用 deactivate。"
                    }
                },
                "required": ["scene_name", "action"],
                "additionalProperties": false
            }),
        ),
    ]
}

pub fn tool_specs_by_name(names: &[&str]) -> Result<Vec<ToolSpec>> {
    let all = tool_specs();
    names
        .iter()
        .map(|name| {
            all.iter()
                .find(|spec| spec.function.name == *name)
                .cloned()
                .ok_or_else(|| anyhow!("unknown tool `{name}` in agent definition"))
        })
        .collect()
}

pub async fn run_tool(name: &str, args: Value) -> Result<String> {
    match name {
        "pwd" => Ok(std::env::current_dir()?.display().to_string()),
        "list_dir" => list_dir(args),
        "read_file" => read_file(args),
        "search_text" => search_text(args).await,
        "shell_exec" => shell_exec(args).await,
        "smart_home_control_speaker"
        | "smart_home_control_light"
        | "smart_home_control_curtain"
        | "smart_home_control_power"
        | "smart_home_control_scene" => smart_home_command(name, args),
        _ => Err(anyhow!("unknown tool `{name}`")),
    }
}

pub fn tool_context_prompt(specs: &[ToolSpec]) -> String {
    let mut prompt = String::from(
        "Tool usage context for this turn:\n\
- Use only tools provided in the current `tools` request field.\n\
- Tool arguments must strictly match each tool JSON schema; include every required field.\n\
- Every tool call must include `tool_title`, a short 4-12 word title in the same language as the user.\n\
- Prefer specialized tools before `shell_exec`: use `pwd`, `list_dir`, `read_file`, or `search_text` when they fit.\n\
- Use at most one tool call per model round unless calls are independent.\n\
- After a tool result, inspect it before deciding whether another tool is needed.\n\
- For `shell_exec`, always provide both `command` and `args`; use `args: []` when there are no arguments. Example: {\"command\":\"uname\",\"args\":[\"-a\"],\"timeout_ms\":3000}.\n\n\
Available tools:\n",
    );

    for spec in specs {
        prompt.push_str("- `");
        prompt.push_str(&spec.function.name);
        prompt.push_str("`: ");
        prompt.push_str(&spec.function.description);
        let required = required_fields(&spec.function.parameters);
        if !required.is_empty() {
            prompt.push_str(" Required: ");
            prompt.push_str(&required.join(", "));
            prompt.push('.');
        }
        prompt.push('\n');
    }

    prompt
}

fn tool(name: &str, description: &str, parameters: Value) -> ToolSpec {
    ToolSpec {
        tool_type: "function".to_string(),
        function: ToolFunctionSpec {
            name: name.to_string(),
            description: description.to_string(),
            parameters: decorate_parameter_schema(parameters),
        },
    }
}

fn required_fields(parameters: &Value) -> Vec<String> {
    parameters
        .get("required")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(ToString::to_string)
        .collect()
}

fn decorate_parameter_schema(mut parameters: Value) -> Value {
    let Some(object) = parameters.as_object_mut() else {
        return parameters;
    };

    let properties = object.entry("properties").or_insert_with(|| json!({}));
    if let Some(properties) = properties.as_object_mut() {
        properties.insert(
            TOOL_TITLE_FIELD.to_string(),
            json!({
                "type": "string",
                "description": "A short 4-12 word title for this tool call, in the same language as the user."
            }),
        );
    }

    let required = object
        .entry("required")
        .or_insert_with(|| Value::Array(Vec::new()));
    if let Some(required) = required.as_array_mut() {
        let has_tool_title = required.iter().any(|value| value == TOOL_TITLE_FIELD);
        if !has_tool_title {
            required.insert(0, Value::String(TOOL_TITLE_FIELD.to_string()));
        }
    }

    parameters
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

fn smart_home_command(name: &str, args: Value) -> Result<String> {
    Ok(serde_json::to_string_pretty(&json!({
        "status": "accepted",
        "tool": name,
        "command": args,
        "note": "normalized smart-home command payload; map this to the real home gateway integration"
    }))?)
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

    #[test]
    fn shell_exec_schema_requires_command_and_args() {
        let specs = tool_specs();
        let shell = specs
            .iter()
            .find(|spec| spec.function.name == "shell_exec")
            .unwrap();
        let required = shell.function.parameters["required"].as_array().unwrap();

        assert!(required.iter().any(|value| value == "tool_title"));
        assert!(required.iter().any(|value| value == "command"));
        assert!(required.iter().any(|value| value == "args"));
    }

    #[test]
    fn smart_home_light_schema_exposes_locations_and_actions() {
        let specs = tool_specs();
        let light = specs
            .iter()
            .find(|spec| spec.function.name == "smart_home_control_light")
            .unwrap();

        assert!(light.function.parameters["properties"]["floor"]["enum"]
            .as_array()
            .unwrap()
            .iter()
            .any(|value| value == "一楼"));
        assert!(light.function.parameters["properties"]["room"]["enum"]
            .as_array()
            .unwrap()
            .iter()
            .any(|value| value == "全屋"));
        assert!(light.function.parameters["properties"]["action"]["enum"]
            .as_array()
            .unwrap()
            .iter()
            .any(|value| value == "set_color"));
    }

    #[test]
    fn tool_context_prompt_includes_schema_guidance() {
        let prompt = tool_context_prompt(&tool_specs());

        assert!(prompt.contains("Tool arguments must strictly match"));
        assert!(prompt.contains("tool_title"));
        assert!(prompt.contains("`shell_exec`"));
        assert!(prompt.contains("Required: tool_title, command, args"));
        assert!(prompt.contains(r#""command":"uname""#));
    }

    #[tokio::test]
    async fn pwd_tool_returns_current_directory() {
        let result = run_tool("pwd", json!({})).await.unwrap();

        assert!(result.contains("ROB"));
    }

    #[tokio::test]
    async fn smart_home_tool_returns_normalized_payload() {
        let result = run_tool(
            "smart_home_control_scene",
            json!({
                "tool_title": "打开回家模式",
                "scene_name": "回家模式",
                "action": "activate"
            }),
        )
        .await
        .unwrap();

        assert!(result.contains("smart_home_control_scene"));
        assert!(result.contains("回家模式"));
    }
}
