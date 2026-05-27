use crate::llm::{system_text_message, ChatMessage, ToolCall};

const DEFAULT_CONTEXT_TOKEN_THRESHOLD: usize = 32_000;
const DEFAULT_CONTEXT_RECENT_MESSAGES: usize = 12;
const CHARS_PER_TOKEN_ESTIMATE: usize = 4;
const SUMMARY_MAX_CHARS: usize = 8_000;

#[derive(Debug, Clone)]
pub struct ContextWindowConfig {
    pub token_threshold: usize,
    pub recent_messages: usize,
}

impl Default for ContextWindowConfig {
    fn default() -> Self {
        Self {
            token_threshold: DEFAULT_CONTEXT_TOKEN_THRESHOLD,
            recent_messages: DEFAULT_CONTEXT_RECENT_MESSAGES,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PreparedContext {
    pub messages: Vec<ChatMessage>,
    pub estimated_tokens: usize,
    pub compacted: bool,
}

pub fn prepare_context(messages: &[ChatMessage], config: &ContextWindowConfig) -> PreparedContext {
    let estimated_tokens = estimate_messages_tokens(messages);
    let token_threshold = config.token_threshold.max(1);
    let recent_messages = config.recent_messages.max(1);

    if estimated_tokens <= token_threshold || messages.len() <= recent_messages + 1 {
        return PreparedContext {
            messages: messages.to_vec(),
            estimated_tokens,
            compacted: false,
        };
    }

    let mut prepared = Vec::new();
    let system = messages
        .first()
        .filter(|message| message.role == "system")
        .cloned();
    if let Some(system) = system {
        prepared.push(system);
    }

    let start = usize::from(
        messages
            .first()
            .is_some_and(|message| message.role == "system"),
    );
    let compactable_end = recent_window_start(messages, recent_messages);
    let compactable = &messages[start..compactable_end];
    let recent = &messages[compactable_end..];

    if !compactable.is_empty() {
        prepared.push(ChatMessage {
            role: "system".to_string(),
            content: Some(build_context_summary(compactable)),
            reasoning_content: None,
            tool_calls: None,
            tool_call_id: None,
            name: None,
        });
    }
    prepared.extend_from_slice(recent);

    PreparedContext {
        messages: prepared,
        estimated_tokens,
        compacted: true,
    }
}

pub fn inject_runtime_context(messages: &[ChatMessage], runtime_context: &str) -> Vec<ChatMessage> {
    let context = runtime_context.trim();
    if context.is_empty() {
        return messages.to_vec();
    }

    let mut prepared = Vec::with_capacity(messages.len() + 1);
    if let Some(system) = messages
        .first()
        .filter(|message| message.role == "system")
        .cloned()
    {
        prepared.push(system);
        prepared.push(system_text_message(context));
        prepared.extend_from_slice(&messages[1..]);
    } else {
        prepared.push(system_text_message(context));
        prepared.extend_from_slice(messages);
    }
    prepared
}

pub fn estimate_messages_tokens(messages: &[ChatMessage]) -> usize {
    let chars = messages
        .iter()
        .map(message_char_count)
        .sum::<usize>()
        .max(1);
    chars.div_ceil(CHARS_PER_TOKEN_ESTIMATE)
}

fn build_context_summary(messages: &[ChatMessage]) -> String {
    let mut summary = String::from(
        "Context summary replacing earlier conversation messages. Preserve these facts while continuing:\n",
    );

    for message in messages {
        let line = summarize_message(message);
        if line.trim().is_empty() {
            continue;
        }
        summary.push_str("- ");
        summary.push_str(&line);
        summary.push('\n');
        if summary.len() >= SUMMARY_MAX_CHARS {
            summary.truncate(SUMMARY_MAX_CHARS);
            summary.push_str("\n[summary truncated]");
            break;
        }
    }

    summary
}

fn summarize_message(message: &ChatMessage) -> String {
    match message.role.as_str() {
        "assistant" => {
            let mut parts = Vec::new();
            if let Some(content) = message
                .content
                .as_deref()
                .filter(|content| !content.is_empty())
            {
                parts.push(format!("assistant said: {}", compact_text(content, 700)));
            }
            if let Some(tool_calls) = &message.tool_calls {
                for call in tool_calls {
                    parts.push(format!("assistant called {}", summarize_tool_call(call)));
                }
            }
            parts.join("; ")
        }
        "tool" => {
            let name = message.name.as_deref().unwrap_or("tool");
            let output = message.content.as_deref().unwrap_or_default();
            format!("{name} result: {}", compact_text(output, 700))
        }
        role => {
            let content = message.content.as_deref().unwrap_or_default();
            format!("{role}: {}", compact_text(content, 700))
        }
    }
}

fn summarize_tool_call(call: &ToolCall) -> String {
    format!(
        "{}({})",
        call.function.name,
        compact_text(&call.function.arguments, 360)
    )
}

fn message_char_count(message: &ChatMessage) -> usize {
    let content = message.content.as_deref().map(str::len).unwrap_or_default();
    let tool_calls = message
        .tool_calls
        .as_deref()
        .unwrap_or_default()
        .iter()
        .map(|call| call.id.len() + call.function.name.len() + call.function.arguments.len())
        .sum::<usize>();
    content
        + tool_calls
        + message
            .tool_call_id
            .as_deref()
            .map(str::len)
            .unwrap_or_default()
        + message.name.as_deref().map(str::len).unwrap_or_default()
        + message.role.len()
}

fn recent_window_start(messages: &[ChatMessage], recent_messages: usize) -> usize {
    let mut start = messages.len().saturating_sub(recent_messages);
    while start > 0 && messages[start].role == "tool" {
        start -= 1;
    }
    start
}

fn compact_text(value: &str, max_chars: usize) -> String {
    let normalized = value.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.chars().count() <= max_chars {
        return normalized;
    }
    let mut compacted = normalized.chars().take(max_chars).collect::<String>();
    compacted.push_str("...");
    compacted
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::{
        assistant_text_message, system_message, tool_message, user_message, ToolCall,
        ToolCallFunction,
    };

    #[test]
    fn prepare_context_compacts_old_messages() {
        let mut messages = vec![system_message()];
        for index in 0..20 {
            messages.push(user_message(&format!("message {index} {}", "x".repeat(80))));
            messages.push(assistant_text_message(&format!("answer {index}")));
        }

        let prepared = prepare_context(
            &messages,
            &ContextWindowConfig {
                token_threshold: 20,
                recent_messages: 4,
            },
        );

        assert!(prepared.compacted);
        assert_eq!(prepared.messages.first().unwrap().role, "system");
        assert!(prepared.messages[1]
            .content
            .as_deref()
            .unwrap()
            .contains("Context summary"));
        assert_eq!(prepared.messages.len(), 6);
    }

    #[test]
    fn prepare_context_keeps_tool_call_pair_together() {
        let messages = vec![
            system_message(),
            user_message("old"),
            assistant_text_message("old answer"),
            user_message("inspect cwd"),
            ChatMessage {
                role: "assistant".to_string(),
                content: None,
                reasoning_content: None,
                tool_calls: Some(vec![ToolCall {
                    id: "call_1".to_string(),
                    call_type: "function".to_string(),
                    function: ToolCallFunction {
                        name: "pwd".to_string(),
                        arguments: "{}".to_string(),
                    },
                }]),
                tool_call_id: None,
                name: None,
            },
            tool_message(
                "call_1".to_string(),
                "pwd".to_string(),
                "/mnt/emmc/lxm/ROB".to_string(),
            ),
        ];

        let prepared = prepare_context(
            &messages,
            &ContextWindowConfig {
                token_threshold: 1,
                recent_messages: 1,
            },
        );

        assert_eq!(prepared.messages[2].role, "assistant");
        assert!(prepared.messages[2].tool_calls.is_some());
        assert_eq!(prepared.messages[3].role, "tool");
    }

    #[test]
    fn inject_runtime_context_keeps_primary_system_first() {
        let messages = vec![system_message(), user_message("hello")];

        let prepared = inject_runtime_context(&messages, "runtime tools");

        assert_eq!(prepared[0].role, "system");
        assert!(prepared[0]
            .content
            .as_deref()
            .unwrap()
            .contains("You are ROB"));
        assert_eq!(prepared[1].role, "system");
        assert_eq!(prepared[1].content.as_deref(), Some("runtime tools"));
        assert_eq!(prepared[2].role, "user");
    }
}
