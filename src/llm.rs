use crate::config::{ProviderProfile, ReasoningEffort};
use crate::tools::{ToolFunctionSpec, ToolSpec};
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
    messages: &'a [ChatMessage],
    model: &'a str,
    max_completion_tokens: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<usize>,
    temperature: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    stream: bool,
    stream_options: ChatCompletionStreamOptions,
    tools: &'a [ToolSpec],
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_choice: Option<&'a str>,
    parallel_tool_calls: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    functions: Option<Vec<ToolFunctionSpec>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    function_call: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reasoning_effort: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    enable_thinking: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    thinking: Option<ChatCompletionThinking>,
    #[serde(skip_serializing_if = "Option::is_none")]
    audio: Option<Value>,
}

#[derive(Debug, Serialize)]
struct ChatCompletionStreamOptions {
    include_usage: bool,
}

#[derive(Debug, Serialize)]
struct ChatCompletionThinking {
    #[serde(rename = "type")]
    thinking_type: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Usage {
    pub prompt_tokens: Option<usize>,
    pub completion_tokens: Option<usize>,
    pub total_tokens: Option<usize>,
}

#[derive(Debug, Default)]
struct ToolCallBuilder {
    id: Option<String>,
    call_type: Option<String>,
    name: String,
    arguments: String,
}

#[derive(Debug, Clone, Copy)]
enum ToolCallMergeMode {
    Delta,
    Full,
}

pub async fn chat_completion_stream<F>(
    profile: &ProviderProfile,
    messages: &[ChatMessage],
    tools: &[ToolSpec],
    tool_choice: Option<&str>,
    reasoning_effort: ReasoningEffort,
    mut on_delta: F,
) -> Result<ChatMessage>
where
    F: FnMut(&str) -> Result<()>,
{
    let api_key = profile.resolve_api_key()?;
    let endpoint = chat_completions_endpoint(&profile.base_url);
    let request = ChatCompletionRequest {
        messages,
        model: &profile.model,
        max_completion_tokens: 16_384,
        max_tokens: None,
        temperature: 0.2,
        top_p: None,
        stream: true,
        stream_options: ChatCompletionStreamOptions {
            include_usage: true,
        },
        tools,
        tool_choice,
        parallel_tool_calls: true,
        functions: None,
        function_call: None,
        reasoning_effort: reasoning_effort.as_request_value(),
        enable_thinking: reasoning_effort.enable_thinking_value(),
        thinking: None,
        audio: None,
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
    let mut current_tool_index = None;
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
                &mut current_tool_index,
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
            &mut current_tool_index,
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
    current_tool_index: &mut Option<usize>,
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

    for chunk in parse_stream_values(data)? {
        if let Some(next_usage) = chunk
            .get("usage")
            .filter(|value| !value.is_null())
            .and_then(|value| serde_json::from_value(value.clone()).ok())
        {
            *usage = Some(next_usage);
        }

        if let Some(choices) = chunk.get("choices").and_then(Value::as_array) {
            for (choice_index, choice) in choices.iter().enumerate() {
                consume_choice(
                    choice_index,
                    choice,
                    content,
                    reasoning,
                    tool_calls,
                    current_tool_index,
                    on_delta,
                )?;
            }
        } else {
            consume_top_level_chunk(
                &chunk,
                content,
                reasoning,
                tool_calls,
                current_tool_index,
                on_delta,
            )?;
        }
    }

    Ok(())
}

fn parse_stream_values(data: &str) -> Result<Vec<Value>> {
    if let Ok(value) = serde_json::from_str(data) {
        return Ok(vec![value]);
    }

    let chunks = split_composite_json_values(data);
    if chunks.len() <= 1 {
        let _: Value = serde_json::from_str(data)
            .with_context(|| format!("failed to decode stream chunk: {data}"))?;
    }

    chunks
        .into_iter()
        .map(|chunk| {
            serde_json::from_str(&chunk)
                .with_context(|| format!("failed to decode stream chunk: {chunk}"))
        })
        .collect()
}

fn split_composite_json_values(data: &str) -> Vec<String> {
    let mut values = Vec::new();
    let mut start = None;
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;

    for (index, ch) in data.char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }

        match ch {
            '"' => in_string = true,
            '{' | '[' => {
                if depth == 0 {
                    start = Some(index);
                }
                depth += 1;
            }
            '}' | ']' if depth > 0 => {
                depth -= 1;
                if depth == 0 {
                    if let Some(start_index) = start.take() {
                        let end = index + ch.len_utf8();
                        values.push(data[start_index..end].trim().to_string());
                    }
                }
            }
            _ => {}
        }
    }

    values
}

fn consume_choice<F>(
    choice_index: usize,
    choice: &Value,
    content: &mut String,
    reasoning: &mut String,
    tool_calls: &mut BTreeMap<usize, ToolCallBuilder>,
    current_tool_index: &mut Option<usize>,
    on_delta: &mut F,
) -> Result<()>
where
    F: FnMut(&str) -> Result<()>,
{
    if let Some(delta) = choice.get("delta") {
        consume_message_like(
            choice_index,
            delta,
            ToolCallMergeMode::Delta,
            content,
            reasoning,
            tool_calls,
            current_tool_index,
            on_delta,
        )?;
    }

    if let Some(message) = choice.get("message") {
        consume_message_field(
            choice_index,
            message,
            content,
            reasoning,
            tool_calls,
            current_tool_index,
            on_delta,
        )?;
    }

    consume_text_fields(choice, content, on_delta)?;
    consume_reasoning_fields(choice, reasoning);
    consume_tool_calls(
        choice.get("tool_calls"),
        ToolCallMergeMode::Full,
        tool_calls,
        current_tool_index,
    );
    consume_legacy_function_call(
        choice_index,
        choice.get("function_call"),
        ToolCallMergeMode::Full,
        tool_calls,
        current_tool_index,
    );

    Ok(())
}

fn consume_top_level_chunk<F>(
    chunk: &Value,
    content: &mut String,
    reasoning: &mut String,
    tool_calls: &mut BTreeMap<usize, ToolCallBuilder>,
    current_tool_index: &mut Option<usize>,
    on_delta: &mut F,
) -> Result<()>
where
    F: FnMut(&str) -> Result<()>,
{
    consume_message_like(
        0,
        chunk,
        ToolCallMergeMode::Delta,
        content,
        reasoning,
        tool_calls,
        current_tool_index,
        on_delta,
    )?;
    if let Some(message) = chunk.get("message").filter(|value| value.is_object()) {
        consume_message_field(
            0,
            message,
            content,
            reasoning,
            tool_calls,
            current_tool_index,
            on_delta,
        )?;
    }
    Ok(())
}

fn consume_message_field<F>(
    choice_index: usize,
    message: &Value,
    content: &mut String,
    reasoning: &mut String,
    tool_calls: &mut BTreeMap<usize, ToolCallBuilder>,
    current_tool_index: &mut Option<usize>,
    on_delta: &mut F,
) -> Result<()>
where
    F: FnMut(&str) -> Result<()>,
{
    if message.is_object() {
        consume_message_like(
            choice_index,
            message,
            ToolCallMergeMode::Full,
            content,
            reasoning,
            tool_calls,
            current_tool_index,
            on_delta,
        )
    } else {
        append_text(message, content, on_delta)
    }
}

fn consume_message_like<F>(
    choice_index: usize,
    message: &Value,
    mode: ToolCallMergeMode,
    content: &mut String,
    reasoning: &mut String,
    tool_calls: &mut BTreeMap<usize, ToolCallBuilder>,
    current_tool_index: &mut Option<usize>,
    on_delta: &mut F,
) -> Result<()>
where
    F: FnMut(&str) -> Result<()>,
{
    consume_text_fields(message, content, on_delta)?;
    consume_reasoning_fields(message, reasoning);
    consume_tool_calls(
        message.get("tool_calls"),
        mode,
        tool_calls,
        current_tool_index,
    );
    consume_tool_calls(
        message.get("tool_call"),
        mode,
        tool_calls,
        current_tool_index,
    );
    consume_legacy_function_call(
        choice_index,
        message.get("function_call"),
        mode,
        tool_calls,
        current_tool_index,
    );
    Ok(())
}

fn consume_text_fields<F>(value: &Value, content: &mut String, on_delta: &mut F) -> Result<()>
where
    F: FnMut(&str) -> Result<()>,
{
    for field in ["content", "text", "output_text", "output", "message"] {
        if let Some(text) = value.get(field) {
            append_text(text, content, on_delta)?;
        }
    }
    Ok(())
}

fn append_text<F>(value: &Value, content: &mut String, on_delta: &mut F) -> Result<()>
where
    F: FnMut(&str) -> Result<()>,
{
    if let Some(text) = value.as_str().filter(|text| !text.is_empty()) {
        content.push_str(text);
        on_delta(text)?;
    }
    Ok(())
}

fn consume_reasoning_fields(value: &Value, reasoning: &mut String) {
    for field in ["reasoning_content", "reasoning", "thinking"] {
        if let Some(text) = value
            .get(field)
            .and_then(Value::as_str)
            .filter(|text| !text.is_empty())
        {
            reasoning.push_str(text);
        }
    }
}

fn consume_tool_calls(
    value: Option<&Value>,
    mode: ToolCallMergeMode,
    tool_calls: &mut BTreeMap<usize, ToolCallBuilder>,
    current_tool_index: &mut Option<usize>,
) {
    let Some(value) = value else {
        return;
    };

    if let Some(calls) = value.as_array() {
        for (array_index, call) in calls.iter().enumerate() {
            merge_tool_call(array_index, call, mode, tool_calls, current_tool_index);
        }
    } else if value.is_object() {
        merge_tool_call(0, value, mode, tool_calls, current_tool_index);
    }
}

fn merge_tool_call(
    array_index: usize,
    call: &Value,
    mode: ToolCallMergeMode,
    tool_calls: &mut BTreeMap<usize, ToolCallBuilder>,
    current_tool_index: &mut Option<usize>,
) {
    let declared_index = call
        .get("index")
        .and_then(Value::as_u64)
        .and_then(|index| usize::try_from(index).ok())
        .unwrap_or(array_index);
    let index = tool_call_merge_index(declared_index, call, tool_calls, current_tool_index);
    let builder = tool_calls.entry(index).or_default();

    if let Some(id) = string_field(call, "id") {
        builder.id = Some(id.to_string());
        *current_tool_index = Some(index);
    }
    if let Some(call_type) = string_field(call, "type") {
        builder.call_type = Some(call_type.to_string());
        *current_tool_index = Some(index);
    }

    if let Some(function) = call.get("function").filter(|function| function.is_object()) {
        if let Some(name) = string_field(function, "name") {
            merge_tool_name(builder, name, mode);
            *current_tool_index = Some(index);
        }
        if let Some(arguments) = function.get("arguments") {
            merge_tool_arguments(builder, arguments, mode);
        }
    }

    if let Some(name) = string_field(call, "name") {
        merge_tool_name(builder, name, mode);
        *current_tool_index = Some(index);
    }
    if let Some(arguments) = call.get("arguments") {
        merge_tool_arguments(builder, arguments, mode);
    }
}

fn tool_call_merge_index(
    declared_index: usize,
    call: &Value,
    tool_calls: &BTreeMap<usize, ToolCallBuilder>,
    current_tool_index: &Option<usize>,
) -> usize {
    let has_identity = string_field(call, "id").is_some()
        || string_field(call, "type").is_some()
        || string_field(call, "name").is_some()
        || call
            .get("function")
            .and_then(|function| string_field(function, "name"))
            .is_some();
    let has_arguments = call.get("arguments").is_some()
        || call
            .get("function")
            .and_then(|function| function.get("arguments"))
            .is_some();

    if has_identity || !has_arguments {
        return declared_index;
    }

    if tool_calls
        .get(&declared_index)
        .is_some_and(|builder| !builder.name.is_empty())
    {
        return declared_index;
    }

    current_tool_index
        .and_then(|index| {
            tool_calls
                .get(&index)
                .filter(|builder| !builder.name.is_empty())
                .map(|_| index)
        })
        .or_else(|| {
            tool_calls
                .iter()
                .rev()
                .find(|(_, builder)| !builder.name.is_empty() && builder.arguments.is_empty())
                .map(|(index, _)| *index)
        })
        .unwrap_or(declared_index)
}

fn consume_legacy_function_call(
    choice_index: usize,
    value: Option<&Value>,
    mode: ToolCallMergeMode,
    tool_calls: &mut BTreeMap<usize, ToolCallBuilder>,
    current_tool_index: &mut Option<usize>,
) {
    let Some(function_call) = value.filter(|value| value.is_object()) else {
        return;
    };
    let builder = tool_calls.entry(choice_index).or_default();
    builder
        .call_type
        .get_or_insert_with(|| "function".to_string());

    if let Some(name) = string_field(function_call, "name") {
        merge_tool_name(builder, name, mode);
        *current_tool_index = Some(choice_index);
    }
    if let Some(arguments) = function_call.get("arguments") {
        merge_tool_arguments(builder, arguments, mode);
    }
}

fn string_field<'a>(value: &'a Value, field: &str) -> Option<&'a str> {
    value
        .get(field)
        .and_then(Value::as_str)
        .filter(|text| !text.is_empty())
}

fn merge_tool_name(builder: &mut ToolCallBuilder, name: &str, mode: ToolCallMergeMode) {
    match mode {
        ToolCallMergeMode::Full => builder.name = name.to_string(),
        ToolCallMergeMode::Delta => append_or_replace_fragment(&mut builder.name, name),
    }
}

fn merge_tool_arguments(builder: &mut ToolCallBuilder, arguments: &Value, mode: ToolCallMergeMode) {
    let Some((arguments, is_string_fragment)) = json_argument_text(arguments) else {
        return;
    };

    match mode {
        ToolCallMergeMode::Full => builder.arguments = arguments,
        ToolCallMergeMode::Delta if !is_string_fragment => builder.arguments = arguments,
        ToolCallMergeMode::Delta => append_or_replace_fragment(&mut builder.arguments, &arguments),
    }
}

fn json_argument_text(value: &Value) -> Option<(String, bool)> {
    if value.is_null() {
        None
    } else if let Some(text) = value.as_str() {
        Some((text.to_string(), true))
    } else {
        serde_json::to_string(value).ok().map(|text| (text, false))
    }
}

fn append_or_replace_fragment(target: &mut String, fragment: &str) {
    if fragment.is_empty() || target.ends_with(fragment) {
        return;
    }
    if fragment.starts_with(target.as_str()) {
        *target = fragment.to_string();
    } else {
        target.push_str(fragment);
    }
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
    use crate::tools::tool_specs;

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
    fn request_body_matches_openomnibot_chat_shape() {
        let messages = vec![system_message(), user_message("hello")];
        let tools = tool_specs();
        let request = ChatCompletionRequest {
            messages: &messages,
            model: "model",
            max_completion_tokens: 16_384,
            max_tokens: None,
            temperature: 0.2,
            top_p: None,
            stream: true,
            stream_options: ChatCompletionStreamOptions {
                include_usage: true,
            },
            tools: &tools,
            tool_choice: Some("auto"),
            parallel_tool_calls: true,
            functions: None,
            function_call: None,
            reasoning_effort: Some("high"),
            enable_thinking: None,
            thinking: None,
            audio: None,
        };

        let value = serde_json::to_value(request).unwrap();

        assert_eq!(value["max_completion_tokens"], 16_384);
        assert_eq!(value["stream_options"]["include_usage"], true);
        assert_eq!(value["tool_choice"], "auto");
        assert_eq!(value["parallel_tool_calls"], true);
        assert_eq!(value["reasoning_effort"], "high");
        assert!(value.get("functions").is_none());
        assert!(value.get("function_call").is_none());
    }

    #[test]
    fn stream_parser_accumulates_content_and_tool_calls() {
        let mut content = String::new();
        let mut reasoning = String::new();
        let mut tool_calls = BTreeMap::new();
        let mut current_tool_index = None;
        let mut usage = None;
        let mut deltas = Vec::new();

        process_sse_line(
            r#"data: {"choices":[{"delta":{"content":"hel"}}]}"#,
            &mut content,
            &mut reasoning,
            &mut tool_calls,
            &mut current_tool_index,
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
            &mut current_tool_index,
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
    fn stream_parser_reads_message_tool_calls() {
        let mut content = String::new();
        let mut reasoning = String::new();
        let mut tool_calls = BTreeMap::new();
        let mut current_tool_index = None;
        let mut usage = None;

        process_sse_line(
            r#"data: {"choices":[{"message":{"tool_calls":[{"id":"call_1","type":"function","function":{"name":"shell_exec","arguments":"{\"command\":\"uname\",\"args\":[\"-a\"]}"}}]}}]}"#,
            &mut content,
            &mut reasoning,
            &mut tool_calls,
            &mut current_tool_index,
            &mut usage,
            &mut |_| Ok(()),
        )
        .unwrap();

        let tool_calls = build_tool_calls(tool_calls);
        assert_eq!(tool_calls[0].function.name, "shell_exec");
        assert_eq!(
            tool_calls[0].function.arguments,
            r#"{"command":"uname","args":["-a"]}"#
        );
    }

    #[test]
    fn stream_parser_reads_direct_choice_tool_calls() {
        let mut content = String::new();
        let mut reasoning = String::new();
        let mut tool_calls = BTreeMap::new();
        let mut current_tool_index = None;
        let mut usage = None;

        process_sse_line(
            r#"data: {"choices":[{"tool_calls":[{"id":"call_1","type":"function","function":{"name":"shell_exec","arguments":"{\"command\":\"env\",\"args\":[]}"}}]}]}"#,
            &mut content,
            &mut reasoning,
            &mut tool_calls,
            &mut current_tool_index,
            &mut usage,
            &mut |_| Ok(()),
        )
        .unwrap();

        let tool_calls = build_tool_calls(tool_calls);
        assert_eq!(tool_calls[0].function.name, "shell_exec");
        assert_eq!(
            tool_calls[0].function.arguments,
            r#"{"command":"env","args":[]}"#
        );
    }

    #[test]
    fn stream_parser_reads_legacy_function_call() {
        let mut content = String::new();
        let mut reasoning = String::new();
        let mut tool_calls = BTreeMap::new();
        let mut current_tool_index = None;
        let mut usage = None;

        process_sse_line(
            r#"data: {"choices":[{"delta":{"function_call":{"name":"shell_exec","arguments":"{\"command\":\"whoami\",\"args\":[]}"}}}]}"#,
            &mut content,
            &mut reasoning,
            &mut tool_calls,
            &mut current_tool_index,
            &mut usage,
            &mut |_| Ok(()),
        )
        .unwrap();

        let tool_calls = build_tool_calls(tool_calls);
        assert_eq!(tool_calls[0].call_type, "function");
        assert_eq!(tool_calls[0].function.name, "shell_exec");
        assert_eq!(
            tool_calls[0].function.arguments,
            r#"{"command":"whoami","args":[]}"#
        );
    }

    #[test]
    fn stream_parser_serializes_object_arguments() {
        let mut content = String::new();
        let mut reasoning = String::new();
        let mut tool_calls = BTreeMap::new();
        let mut current_tool_index = None;
        let mut usage = None;

        process_sse_line(
            r#"data: {"choices":[{"message":{"tool_calls":[{"id":"call_1","type":"function","function":{"name":"shell_exec","arguments":{"command":"df","args":["-h"]}}}]}}]}"#,
            &mut content,
            &mut reasoning,
            &mut tool_calls,
            &mut current_tool_index,
            &mut usage,
            &mut |_| Ok(()),
        )
        .unwrap();

        let tool_calls = build_tool_calls(tool_calls);
        assert_eq!(
            tool_calls[0].function.arguments,
            r#"{"args":["-h"],"command":"df"}"#
        );
    }

    #[test]
    fn stream_parser_appends_fragmented_arguments() {
        let mut content = String::new();
        let mut reasoning = String::new();
        let mut tool_calls = BTreeMap::new();
        let mut current_tool_index = None;
        let mut usage = None;

        process_sse_line(
            r#"data: {"choices":[{"delta":{"tool_calls":[{"index":0,"id":"call_1","type":"function","function":{"name":"shell_","arguments":"{\"command\":\"uname\""}}]}}]}"#,
            &mut content,
            &mut reasoning,
            &mut tool_calls,
            &mut current_tool_index,
            &mut usage,
            &mut |_| Ok(()),
        )
        .unwrap();
        process_sse_line(
            r#"data: {"choices":[{"delta":{"tool_calls":[{"index":0,"function":{"name":"exec","arguments":",\"args\":[\"-a\"]}"}}]}}]}"#,
            &mut content,
            &mut reasoning,
            &mut tool_calls,
            &mut current_tool_index,
            &mut usage,
            &mut |_| Ok(()),
        )
        .unwrap();

        let tool_calls = build_tool_calls(tool_calls);
        assert_eq!(tool_calls[0].function.name, "shell_exec");
        assert_eq!(
            tool_calls[0].function.arguments,
            r#"{"command":"uname","args":["-a"]}"#
        );
    }

    #[test]
    fn stream_parser_routes_argument_only_chunks_to_current_tool() {
        let mut content = String::new();
        let mut reasoning = String::new();
        let mut tool_calls = BTreeMap::new();
        let mut current_tool_index = None;
        let mut usage = None;

        process_sse_line(
            r#"data: {"choices":[{"delta":{"tool_calls":[{"function":{"arguments":"","name":"shell_exec"},"id":"call_1","index":0,"type":"function"}]}}]}"#,
            &mut content,
            &mut reasoning,
            &mut tool_calls,
            &mut current_tool_index,
            &mut usage,
            &mut |_| Ok(()),
        )
        .unwrap();
        process_sse_line(
            r#"data: {"choices":[{"delta":{"tool_calls":[{"function":{"arguments":"{\"args\":[\"-a\"],\"command\":\"uname\"}"},"index":1}]}}]}"#,
            &mut content,
            &mut reasoning,
            &mut tool_calls,
            &mut current_tool_index,
            &mut usage,
            &mut |_| Ok(()),
        )
        .unwrap();
        process_sse_line(
            r#"data: {"choices":[{"delta":{"tool_calls":[{"function":{"arguments":"","name":"shell_exec"},"id":"call_2","index":2,"type":"function"}]}}]}"#,
            &mut content,
            &mut reasoning,
            &mut tool_calls,
            &mut current_tool_index,
            &mut usage,
            &mut |_| Ok(()),
        )
        .unwrap();
        process_sse_line(
            r#"data: {"choices":[{"delta":{"tool_calls":[{"function":{"arguments":"{\"args\":[],\"command\":\"whoami\"}"},"index":1}]}}]}"#,
            &mut content,
            &mut reasoning,
            &mut tool_calls,
            &mut current_tool_index,
            &mut usage,
            &mut |_| Ok(()),
        )
        .unwrap();

        let tool_calls = build_tool_calls(tool_calls);
        assert_eq!(tool_calls.len(), 2);
        assert_eq!(tool_calls[0].id, "call_1");
        assert_eq!(
            tool_calls[0].function.arguments,
            r#"{"args":["-a"],"command":"uname"}"#
        );
        assert_eq!(tool_calls[1].id, "call_2");
        assert_eq!(
            tool_calls[1].function.arguments,
            r#"{"args":[],"command":"whoami"}"#
        );
    }

    #[test]
    fn stream_parser_accumulates_reasoning() {
        let mut content = String::new();
        let mut reasoning = String::new();
        let mut tool_calls = BTreeMap::new();
        let mut current_tool_index = None;
        let mut usage = None;

        process_sse_line(
            r#"data: {"choices":[{"delta":{"reasoning_content":"think"}}],"usage":{"prompt_tokens":12}}"#,
            &mut content,
            &mut reasoning,
            &mut tool_calls,
            &mut current_tool_index,
            &mut usage,
            &mut |_| Ok(()),
        )
        .unwrap();

        assert_eq!(reasoning, "think");
        assert_eq!(usage.unwrap().prompt_tokens, Some(12));
    }
}
