use crate::config::ProviderProfile;
use crate::tools::ToolSpec;
use anyhow::{Context, Result};
use futures_util::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub call_type: String,
    pub function: ToolCallFunction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallFunction {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Serialize)]
struct ChatCompletionRequest<'a> {
    model: &'a str,
    messages: &'a [ChatMessage],
    tools: &'a [ToolSpec],
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_choice: Option<&'a str>,
    parallel_tool_calls: bool,
    temperature: f32,
    stream: bool,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionStreamChunk {
    choices: Vec<ChatStreamChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatStreamChoice {
    delta: ChatStreamDelta,
}

#[derive(Debug, Deserialize)]
struct ChatStreamDelta {
    content: Option<String>,
    #[serde(default)]
    tool_calls: Vec<ToolCallDelta>,
}

#[derive(Debug, Deserialize)]
struct ToolCallDelta {
    index: usize,
    id: Option<String>,
    #[serde(rename = "type")]
    call_type: Option<String>,
    function: Option<ToolCallFunctionDelta>,
}

#[derive(Debug, Deserialize)]
struct ToolCallFunctionDelta {
    name: Option<String>,
    arguments: Option<String>,
}

#[derive(Debug, Default)]
struct ToolCallBuilder {
    id: Option<String>,
    call_type: Option<String>,
    name: String,
    arguments: String,
}

pub async fn chat_completion_stream<F>(
    profile: &ProviderProfile,
    messages: &[ChatMessage],
    tools: &[ToolSpec],
    mut on_delta: F,
) -> Result<ChatMessage>
where
    F: FnMut(&str) -> Result<()>,
{
    let api_key = profile.resolve_api_key()?;
    let endpoint = chat_completions_endpoint(&profile.base_url);
    let request = ChatCompletionRequest {
        model: &profile.model,
        messages,
        tools,
        tool_choice: None,
        parallel_tool_calls: false,
        temperature: 0.2,
        stream: true,
    };

    let response = Client::new()
        .post(endpoint)
        .bearer_auth(api_key)
        .json(&request)
        .send()
        .await
        .context("streaming chat completion request failed")?;

    let status = response.status();
    if !status.is_success() {
        let body = response
            .text()
            .await
            .context("failed to read model response")?;
        anyhow::bail!("model provider returned HTTP {status}: {body}");
    }

    let mut content = String::new();
    let mut tool_calls: BTreeMap<usize, ToolCallBuilder> = BTreeMap::new();
    let mut pending = String::new();
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.context("failed to read streaming response chunk")?;
        pending.push_str(&String::from_utf8_lossy(&chunk));

        while let Some(newline) = pending.find('\n') {
            let line = pending[..newline].trim_end_matches('\r').to_string();
            pending.drain(..=newline);
            process_sse_line(&line, &mut content, &mut tool_calls, &mut on_delta)?;
        }
    }

    if !pending.trim().is_empty() {
        let line = pending.trim_end_matches('\r').to_string();
        process_sse_line(&line, &mut content, &mut tool_calls, &mut on_delta)?;
    }

    let tool_calls = build_tool_calls(tool_calls);
    Ok(ChatMessage {
        role: "assistant".to_string(),
        content: if content.is_empty() {
            None
        } else {
            Some(content)
        },
        tool_calls: if tool_calls.is_empty() {
            None
        } else {
            Some(tool_calls)
        },
        tool_call_id: None,
        name: None,
    })
}

pub fn system_message() -> ChatMessage {
    ChatMessage {
        role: "system".to_string(),
        content: Some(
            "You are ROB, a Linux-native CLI agent migrated from OpenOmniBot concepts. \
Use tools when they help inspect the local Linux environment. Keep answers concise. \
When using shell_exec, pass a command and argv array; never assume shell expansion."
                .to_string(),
        ),
        tool_calls: None,
        tool_call_id: None,
        name: None,
    }
}

pub fn user_message(content: &str) -> ChatMessage {
    ChatMessage {
        role: "user".to_string(),
        content: Some(content.to_string()),
        tool_calls: None,
        tool_call_id: None,
        name: None,
    }
}

pub fn assistant_message(message: ChatMessage) -> ChatMessage {
    ChatMessage {
        role: "assistant".to_string(),
        content: message.content,
        tool_calls: message.tool_calls,
        tool_call_id: None,
        name: None,
    }
}

pub fn assistant_text_message(content: &str) -> ChatMessage {
    ChatMessage {
        role: "assistant".to_string(),
        content: Some(content.to_string()),
        tool_calls: None,
        tool_call_id: None,
        name: None,
    }
}

pub fn tool_message(tool_call_id: String, name: String, content: String) -> ChatMessage {
    ChatMessage {
        role: "tool".to_string(),
        content: Some(content),
        tool_calls: None,
        tool_call_id: Some(tool_call_id),
        name: Some(name),
    }
}

pub fn content_as_text(content: Option<String>) -> String {
    content.unwrap_or_default()
}

pub fn parse_tool_arguments(raw: &str) -> Value {
    serde_json::from_str(raw).unwrap_or_else(|_| Value::Object(Default::default()))
}

fn chat_completions_endpoint(base_url: &str) -> String {
    let normalized = base_url.trim_end_matches('/');
    if normalized.ends_with("/chat/completions") {
        normalized.to_string()
    } else {
        format!("{normalized}/chat/completions")
    }
}

fn process_sse_line<F>(
    line: &str,
    content: &mut String,
    tool_calls: &mut BTreeMap<usize, ToolCallBuilder>,
    on_delta: &mut F,
) -> Result<()>
where
    F: FnMut(&str) -> Result<()>,
{
    let line = line.trim();
    if line.is_empty() || line.starts_with(':') {
        return Ok(());
    }

    let Some(data) = line.strip_prefix("data:") else {
        return Ok(());
    };
    let data = data.trim();
    if data == "[DONE]" {
        return Ok(());
    }

    let chunk: ChatCompletionStreamChunk = serde_json::from_str(data)
        .with_context(|| format!("failed to decode stream chunk: {data}"))?;

    for choice in chunk.choices {
        if let Some(delta) = choice.delta.content {
            if !delta.is_empty() {
                content.push_str(&delta);
                on_delta(&delta)?;
            }
        }

        for delta in choice.delta.tool_calls {
            let builder = tool_calls.entry(delta.index).or_default();
            if let Some(id) = delta.id {
                builder.id = Some(id);
            }
            if let Some(call_type) = delta.call_type {
                builder.call_type = Some(call_type);
            }
            if let Some(function) = delta.function {
                if let Some(name) = function.name {
                    builder.name.push_str(&name);
                }
                if let Some(arguments) = function.arguments {
                    builder.arguments.push_str(&arguments);
                }
            }
        }
    }

    Ok(())
}

fn build_tool_calls(builders: BTreeMap<usize, ToolCallBuilder>) -> Vec<ToolCall> {
    builders
        .into_values()
        .filter(|builder| !builder.name.is_empty())
        .map(|builder| ToolCall {
            id: builder.id.unwrap_or_else(|| "tool_call".to_string()),
            call_type: builder.call_type.unwrap_or_else(|| "function".to_string()),
            function: ToolCallFunction {
                name: builder.name,
                arguments: builder.arguments,
            },
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chat_endpoint_appends_chat_completions() {
        assert_eq!(
            chat_completions_endpoint("https://api.example.com/v1"),
            "https://api.example.com/v1/chat/completions"
        );
    }

    #[test]
    fn chat_endpoint_keeps_explicit_chat_completions() {
        assert_eq!(
            chat_completions_endpoint("https://api.example.com/v1/chat/completions"),
            "https://api.example.com/v1/chat/completions"
        );
    }

    #[test]
    fn stream_parser_accumulates_content_and_tool_calls() {
        let mut content = String::new();
        let mut tool_calls = BTreeMap::new();
        let mut deltas = Vec::new();

        process_sse_line(
            r#"data: {"choices":[{"delta":{"content":"hel"}}]}"#,
            &mut content,
            &mut tool_calls,
            &mut |delta| {
                deltas.push(delta.to_string());
                Ok(())
            },
        )
        .unwrap();
        process_sse_line(
            r#"data: {"choices":[{"delta":{"tool_calls":[{"index":0,"id":"call_1","type":"function","function":{"name":"pwd","arguments":"{}"}}]}}]}"#,
            &mut content,
            &mut tool_calls,
            &mut |_| Ok(()),
        )
        .unwrap();

        let tool_calls = build_tool_calls(tool_calls);
        assert_eq!(content, "hel");
        assert_eq!(deltas, vec!["hel"]);
        assert_eq!(tool_calls[0].function.name, "pwd");
        assert_eq!(tool_calls[0].function.arguments, "{}");
    }
}
