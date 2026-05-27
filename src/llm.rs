use crate::config::{ProviderProfile, ReasoningEffort};
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
    pub reasoning_content: Option<String>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    reasoning_effort: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    enable_thinking: Option<bool>,
    temperature: f32,
    stream: bool,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionStreamChunk {
    choices: Vec<ChatStreamChoice>,
    usage: Option<Usage>,
}

#[derive(Debug, Deserialize)]
struct ChatStreamChoice {
    delta: ChatStreamDelta,
}

#[derive(Debug, Deserialize)]
struct ChatStreamDelta {
    content: Option<String>,
    reasoning_content: Option<String>,
    reasoning: Option<String>,
    thinking: Option<String>,
    #[serde(default)]
    tool_calls: Vec<ToolCallDelta>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Usage {
    pub prompt_tokens: Option<usize>,
    pub completion_tokens: Option<usize>,
    pub total_tokens: Option<usize>,
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
    reasoning_effort: ReasoningEffort,
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
        reasoning_effort: reasoning_effort.as_request_value(),
        enable_thinking: reasoning_effort.enable_thinking_value(),
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
    let mut reasoning = String::new();
    let mut usage = None;
    let mut tool_calls: BTreeMap<usize, ToolCallBuilder> = BTreeMap::new();
    let mut pending = String::new();
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.context("failed to read streaming response chunk")?;
        pending.push_str(&String::from_utf8_lossy(&chunk));

        while let Some(newline) = pending.find('\n') {
            let line = pending[..newline].trim_end_matches('\r').to_string();
            pending.drain(..=newline);
            process_sse_line(
                &line,
                &mut content,
                &mut reasoning,
                &mut tool_calls,
                &mut usage,
                &mut on_delta,
            )?;
        }
    }

    if !pending.trim().is_empty() {
        let line = pending.trim_end_matches('\r').to_string();
        process_sse_line(
            &line,
            &mut content,
            &mut reasoning,
            &mut tool_calls,
            &mut usage,
            &mut on_delta,
        )?;
    }

    let tool_calls = build_tool_calls(tool_calls);
    Ok(ChatMessage {
        role: "assistant".to_string(),
        content: if content.is_empty() {
            None
        } else {
            Some(content)
        },
        reasoning_content: if reasoning.is_empty() {
            None
        } else {
            Some(reasoning)
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

impl ReasoningEffort {
    fn as_request_value(&self) -> Option<&'static str> {
        match self {
            Self::Auto | Self::No => None,
            Self::Low => Some("low"),
            Self::Medium => Some("medium"),
            Self::High => Some("high"),
        }
    }

    fn enable_thinking_value(&self) -> Option<bool> {
        match self {
            Self::No => Some(false),
            Self::Auto | Self::Low | Self::Medium | Self::High => None,
        }
    }
}

pub fn system_message() -> ChatMessage {
    system_text_message(
        "You are ROB, a Linux-native CLI agent migrated from OpenOmniBot concepts. \
Use tools when they help inspect the local Linux environment. Keep answers concise. \
When using shell_exec, pass a command and argv array; never assume shell expansion.",
    )
}

pub fn system_text_message(content: &str) -> ChatMessage {
    ChatMessage {
        role: "system".to_string(),
        content: Some(content.to_string()),
        reasoning_content: None,
        tool_calls: None,
        tool_call_id: None,
        name: None,
    }
}

pub fn user_message(content: &str) -> ChatMessage {
    ChatMessage {
        role: "user".to_string(),
        content: Some(content.to_string()),
        reasoning_content: None,
        tool_calls: None,
        tool_call_id: None,
        name: None,
    }
}

pub fn assistant_message(message: ChatMessage) -> ChatMessage {
    ChatMessage {
        role: "assistant".to_string(),
        content: message.content,
        reasoning_content: message.reasoning_content,
        tool_calls: message.tool_calls,
        tool_call_id: None,
        name: None,
    }
}

pub fn assistant_text_message(content: &str) -> ChatMessage {
    ChatMessage {
        role: "assistant".to_string(),
        content: Some(content.to_string()),
        reasoning_content: None,
        tool_calls: None,
        tool_call_id: None,
        name: None,
    }
}

pub fn tool_message(tool_call_id: String, name: String, content: String) -> ChatMessage {
    ChatMessage {
        role: "tool".to_string(),
        content: Some(content),
        reasoning_content: None,
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
    reasoning: &mut String,
    tool_calls: &mut BTreeMap<usize, ToolCallBuilder>,
    usage: &mut Option<Usage>,
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
    if chunk.usage.is_some() {
        *usage = chunk.usage;
    }

    for choice in chunk.choices {
        if let Some(delta) = choice.delta.content {
            if !delta.is_empty() {
                content.push_str(&delta);
                on_delta(&delta)?;
            }
        }
        for delta in [
            choice.delta.reasoning_content,
            choice.delta.reasoning,
            choice.delta.thinking,
        ]
        .into_iter()
        .flatten()
        {
            reasoning.push_str(&delta);
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
        let mut reasoning = String::new();
        let mut tool_calls = BTreeMap::new();
        let mut usage = None;
        let mut deltas = Vec::new();

        process_sse_line(
            r#"data: {"choices":[{"delta":{"content":"hel"}}]}"#,
            &mut content,
            &mut reasoning,
            &mut tool_calls,
            &mut usage,
            &mut |delta| {
                deltas.push(delta.to_string());
                Ok(())
            },
        )
        .unwrap();
        process_sse_line(
            r#"data: {"choices":[{"delta":{"tool_calls":[{"index":0,"id":"call_1","type":"function","function":{"name":"pwd","arguments":"{}"}}]}}]}"#,
            &mut content,
            &mut reasoning,
            &mut tool_calls,
            &mut usage,
            &mut |_| Ok(()),
        )
        .unwrap();

        let tool_calls = build_tool_calls(tool_calls);
        assert_eq!(content, "hel");
        assert_eq!(deltas, vec!["hel"]);
        assert_eq!(tool_calls[0].function.name, "pwd");
        assert_eq!(tool_calls[0].function.arguments, "{}");
    }

    #[test]
    fn stream_parser_accumulates_reasoning() {
        let mut content = String::new();
        let mut reasoning = String::new();
        let mut tool_calls = BTreeMap::new();
        let mut usage = None;

        process_sse_line(
            r#"data: {"choices":[{"delta":{"reasoning_content":"think"}}],"usage":{"prompt_tokens":12}}"#,
            &mut content,
            &mut reasoning,
            &mut tool_calls,
            &mut usage,
            &mut |_| Ok(()),
        )
        .unwrap();

        assert_eq!(reasoning, "think");
        assert_eq!(usage.unwrap().prompt_tokens, Some(12));
    }
}
