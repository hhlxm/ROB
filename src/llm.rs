use crate::config::ProviderProfile;
use crate::tools::ToolSpec;
use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

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
    temperature: f32,
}

#[derive(Debug, Deserialize)]
pub struct ChatCompletionResponse {
    pub choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
pub struct ChatChoice {
    pub message: ChatMessage,
}

pub async fn chat_completion(
    profile: &ProviderProfile,
    messages: &[ChatMessage],
    tools: &[ToolSpec],
) -> Result<ChatMessage> {
    let api_key = profile.resolve_api_key()?;
    let endpoint = chat_completions_endpoint(&profile.base_url);
    let request = ChatCompletionRequest {
        model: &profile.model,
        messages,
        tools,
        tool_choice: None,
        temperature: 0.2,
    };

    let response = Client::new()
        .post(endpoint)
        .bearer_auth(api_key)
        .json(&request)
        .send()
        .await
        .context("chat completion request failed")?;

    let status = response.status();
    let body = response
        .text()
        .await
        .context("failed to read model response")?;
    if !status.is_success() {
        anyhow::bail!("model provider returned HTTP {status}: {body}");
    }

    let parsed: ChatCompletionResponse = serde_json::from_str(&body)
        .with_context(|| format!("failed to decode response: {body}"))?;
    parsed
        .choices
        .into_iter()
        .next()
        .map(|choice| choice.message)
        .context("model response did not contain choices")
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
}
