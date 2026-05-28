use crate::agents::{agent_for_system_prompt, resolve_agent, AgentDefinition};
use crate::config::{ApprovalPolicy, RobConfig};
use crate::context::{inject_runtime_context, prepare_context, ContextWindowConfig};
use crate::llm::{
    assistant_message, assistant_text_message, chat_completion_stream, content_as_text,
    parse_tool_arguments, system_text_message, tool_message, user_message, ChatMessage, ToolCall,
};
use crate::state;
use crate::tools::{run_tool, tool_context_prompt};
use anyhow::Result;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use serde_json::Value;
use std::collections::HashMap;
use std::io::{self, Write};
use uuid::Uuid;

const TOOL_DENIED_MESSAGE: &str = "tool denied by approval policy";
const INVALID_TOOL_REPEAT_LIMIT: usize = 2;

#[derive(Debug, Clone)]
pub enum AgentEvent {
    AssistantDelta(String),
    ContextWindow {
        estimated_tokens: usize,
        threshold: usize,
        compacted: bool,
    },
    ToolCallStarted {
        id: String,
        name: String,
        arguments: Value,
    },
    ToolCallCompleted {
        id: String,
        name: String,
        output: String,
    },
    ToolCallFailed {
        id: String,
        name: String,
        error: String,
    },
    ToolCallDenied {
        id: String,
        name: String,
    },
}

pub struct AgentSession {
    id: String,
    agent: AgentDefinition,
    config: RobConfig,
    messages: Vec<ChatMessage>,
    approval_policy: ApprovalPolicy,
    interactive_approval: bool,
}

impl AgentSession {
    pub fn new(
        config: RobConfig,
        agent_override: Option<String>,
        model_override: Option<String>,
        resume_id: Option<String>,
        approval_override: Option<ApprovalPolicy>,
    ) -> Result<Self> {
        Self::new_inner(
            config,
            agent_override,
            model_override,
            resume_id,
            approval_override,
            false,
        )
    }

    fn new_interactive(
        config: RobConfig,
        agent_override: Option<String>,
        model_override: Option<String>,
        resume_id: Option<String>,
        approval_override: Option<ApprovalPolicy>,
    ) -> Result<Self> {
        Self::new_inner(
            config,
            agent_override,
            model_override,
            resume_id,
            approval_override,
            true,
        )
    }

    fn new_inner(
        mut config: RobConfig,
        agent_override: Option<String>,
        model_override: Option<String>,
        resume_id: Option<String>,
        approval_override: Option<ApprovalPolicy>,
        interactive_approval: bool,
    ) -> Result<Self> {
        if let Some(model) = model_override {
            let mut profile = config.active_profile()?.clone();
            profile.model = model;
            config.set_active_profile(profile);
        }

        let approval_policy = approval_override.unwrap_or(config.tool_approval);
        let requested_agent = resolve_agent(agent_override.as_deref())?;
        let (id, agent, messages) = if let Some(id) = resume_id {
            let record = state::load_session(&id)?;
            let agent = if agent_override.is_some() {
                requested_agent
            } else {
                agent_for_system_prompt(
                    record
                        .messages
                        .first()
                        .and_then(|message| message.content.as_deref()),
                )
            };
            (record.id, agent, record.messages)
        } else {
            (
                Uuid::new_v4().to_string(),
                requested_agent.clone(),
                vec![system_text_message(requested_agent.system_prompt)],
            )
        };

        Ok(Self {
            id,
            agent,
            config,
            messages,
            approval_policy,
            interactive_approval,
        })
    }

    pub async fn send_user_message_streaming<F>(
        &mut self,
        input: &str,
        mut on_delta: F,
    ) -> Result<String>
    where
        F: FnMut(&str) -> Result<()>,
    {
        self.send_user_message_events(input, |event| {
            match event {
                AgentEvent::AssistantDelta(delta) => on_delta(&delta)?,
                AgentEvent::ContextWindow { .. }
                | AgentEvent::ToolCallStarted { .. }
                | AgentEvent::ToolCallCompleted { .. }
                | AgentEvent::ToolCallFailed { .. }
                | AgentEvent::ToolCallDenied { .. } => {}
            }
            Ok(())
        })
        .await
    }

    pub async fn send_user_message_events<F>(
        &mut self,
        input: &str,
        mut on_event: F,
    ) -> Result<String>
    where
        F: FnMut(AgentEvent) -> Result<()>,
    {
        self.messages.push(user_message(input));
        self.persist()?;
        let mut invalid_tool_counts = HashMap::new();

        loop {
            let profile = self.config.active_profile()?;
            let tools = self.agent.tools()?;
            let tool_context = tool_context_prompt(&tools);
            let context_config = ContextWindowConfig {
                token_threshold: self.config.context.token_threshold,
                recent_messages: self.config.context.recent_messages,
            };
            let prepared_context = prepare_context(&self.messages, &context_config);
            let request_messages =
                inject_runtime_context(&prepared_context.messages, &tool_context);
            on_event(AgentEvent::ContextWindow {
                estimated_tokens: prepared_context.estimated_tokens,
                threshold: context_config.token_threshold,
                compacted: prepared_context.compacted,
            })?;
            let model_message = chat_completion_stream(
                profile,
                &request_messages,
                &tools,
                tool_choice_for_round(&self.messages),
                self.config.reasoning.effort,
                |delta| on_event(AgentEvent::AssistantDelta(delta.to_string())),
            )
            .await?;
            let tool_calls = model_message.tool_calls.clone().unwrap_or_default();
            let assistant = assistant_message(model_message);
            let answer = content_as_text(assistant.content.clone());
            self.messages.push(assistant);
            self.persist()?;

            if tool_calls.is_empty() {
                return Ok(answer);
            }

            for call in tool_calls {
                let outcome = self
                    .run_and_append_tool_call(call, &mut on_event, &mut invalid_tool_counts)
                    .await?;
                if let ToolOutcome::InvalidRepeated(message) = outcome {
                    self.messages.push(assistant_text_message(&message));
                    self.persist()?;
                    on_event(AgentEvent::AssistantDelta(message.clone()))?;
                    return Ok(message);
                }
            }

            invalid_tool_counts.clear();
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn transcript(&self) -> &[ChatMessage] {
        &self.messages
    }

    pub fn agent_tools(&self) -> Result<Vec<crate::tools::ToolSpec>> {
        self.agent.tools()
    }

    pub fn config_summary(&self) -> Result<String> {
        let profile = self.config.active_profile()?;
        Ok(format!(
            "agent={} | {} | {} | {} | {} | approval={} | reasoning={} | context={} tokens/{} msgs",
            self.agent.name,
            profile.name,
            profile.base_url,
            profile.model,
            profile.protocol,
            self.approval_policy,
            self.config.reasoning.effort,
            self.config.context.token_threshold,
            self.config.context.recent_messages
        ))
    }

    fn approve_tool(&self, name: &str, args: &Value) -> Result<bool> {
        match self.approval_policy {
            ApprovalPolicy::Auto => Ok(true),
            ApprovalPolicy::OnRequest if self.interactive_approval => {
                print!("Approve tool `{name}` with args {args}? [y/N] ");
                io::stdout().flush()?;
                let mut answer = String::new();
                io::stdin().read_line(&mut answer)?;
                Ok(matches!(answer.trim(), "y" | "Y" | "yes" | "YES"))
            }
            ApprovalPolicy::OnRequest => Ok(false),
        }
    }

    fn persist(&self) -> Result<()> {
        state::save_session(&self.id, &self.messages)
    }

    async fn run_and_append_tool_call<F>(
        &mut self,
        call: ToolCall,
        on_event: &mut F,
        invalid_tool_counts: &mut HashMap<String, usize>,
    ) -> Result<ToolOutcome>
    where
        F: FnMut(AgentEvent) -> Result<()>,
    {
        let args = parse_tool_arguments(&call.function.arguments);
        on_event(AgentEvent::ToolCallStarted {
            id: call.id.clone(),
            name: call.function.name.clone(),
            arguments: args.clone(),
        })?;

        let outcome = if !self
            .agent
            .tool_names()
            .contains(&call.function.name.as_str())
        {
            let error = format!(
                "tool `{}` is not available to agent `{}`",
                call.function.name, self.agent.name
            );
            invalid_tool_call_outcome(
                &call.function.name,
                &error,
                &format!(
                    "use only these tools: {}",
                    self.agent.tool_names().join(", ")
                ),
                invalid_tool_counts,
            )
        } else if let Some(error) = validate_tool_arguments(&call.function.name, &args) {
            invalid_tool_call_outcome(
                &call.function.name,
                &error,
                &invalid_tool_example(&call.function.name),
                invalid_tool_counts,
            )
        } else if self.approve_tool(&call.function.name, &args)? {
            match run_tool(&call.function.name, args).await {
                Ok(output) => ToolOutcome::Completed(output),
                Err(error) => ToolOutcome::Failed(format!("tool error: {error:#}")),
            }
        } else {
            ToolOutcome::Denied
        };

        let output = outcome.output().to_string();
        self.messages.push(tool_message(
            call.id.clone(),
            call.function.name.clone(),
            output.clone(),
        ));
        self.persist()?;

        match outcome {
            ToolOutcome::Completed(_) => on_event(AgentEvent::ToolCallCompleted {
                id: call.id,
                name: call.function.name,
                output,
            })?,
            ToolOutcome::Failed(_) | ToolOutcome::InvalidRepeated(_) => {
                on_event(AgentEvent::ToolCallFailed {
                    id: call.id,
                    name: call.function.name,
                    error: output,
                })?
            }
            ToolOutcome::Denied => on_event(AgentEvent::ToolCallDenied {
                id: call.id,
                name: call.function.name,
            })?,
        }

        Ok(outcome)
    }
}

enum ToolOutcome {
    Completed(String),
    Failed(String),
    Denied,
    InvalidRepeated(String),
}

impl ToolOutcome {
    fn output(&self) -> &str {
        match self {
            Self::Completed(output) | Self::Failed(output) | Self::InvalidRepeated(output) => {
                output
            }
            Self::Denied => TOOL_DENIED_MESSAGE,
        }
    }
}

fn validate_tool_arguments(name: &str, args: &Value) -> Option<String> {
    match name {
        "shell_exec" => {
            let command = args
                .get("command")
                .and_then(Value::as_str)
                .unwrap_or_default();
            if command.trim().is_empty() {
                return Some("missing required string argument `command`".to_string());
            }
            if !args.get("args").is_some_and(Value::is_array) {
                return Some(
                    "missing required array argument `args`; use [] for no arguments".to_string(),
                );
            }
            None
        }
        "read_file" => missing_string(args, "path"),
        "search_text" => missing_string(args, "pattern"),
        _ => None,
    }
}

fn missing_string(args: &Value, key: &str) -> Option<String> {
    args.get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(|_| ())
        .ok_or_else(|| format!("missing required string argument `{key}`"))
        .err()
}

fn invalid_tool_example(name: &str) -> String {
    match name {
        "shell_exec" => r#"example: {"command":"uname","args":["-a"],"timeout_ms":3000}"#,
        "read_file" => r#"example: {"path":"README.md"}"#,
        "search_text" => r#"example: {"pattern":"TODO","path":"."}"#,
        _ => "check the tool schema and provide all required arguments",
    }
    .to_string()
}

fn invalid_tool_call_outcome(
    name: &str,
    error: &str,
    hint: &str,
    invalid_tool_counts: &mut HashMap<String, usize>,
) -> ToolOutcome {
    let count = invalid_tool_counts
        .entry(format!("{name}:{error}"))
        .and_modify(|count| *count += 1)
        .or_insert(1);
    if *count >= INVALID_TOOL_REPEAT_LIMIT {
        ToolOutcome::InvalidRepeated(format!(
            "Stopped because the model repeatedly produced an invalid `{name}` tool call: {error}.",
        ))
    } else {
        ToolOutcome::Failed(invalid_tool_guidance_with_hint(name, error, hint, *count))
    }
}

fn invalid_tool_guidance_with_hint(name: &str, error: &str, hint: &str, count: usize) -> String {
    format!(
        "tool argument error ({count}/{INVALID_TOOL_REPEAT_LIMIT}) for `{name}`: {error}. Retry with a valid tool call; {hint}"
    )
}

fn tool_choice_for_round(messages: &[ChatMessage]) -> Option<&'static str> {
    match messages.last().map(|message| message.role.as_str()) {
        Some("tool") => None,
        _ => Some("auto"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn validates_empty_shell_exec_arguments_before_running_tool() {
        let error = validate_tool_arguments("shell_exec", &json!({})).unwrap();

        assert!(error.contains("command"));
    }

    #[test]
    fn accepts_valid_shell_exec_arguments() {
        let error = validate_tool_arguments(
            "shell_exec",
            &json!({
                "tool_title": "检查内核版本",
                "command": "uname",
                "args": ["-a"]
            }),
        );

        assert!(error.is_none());
    }

    #[test]
    fn repeated_unavailable_tool_call_stops_loop() {
        let mut counts = HashMap::new();
        let error = "tool `shell_exec` is not available to agent `reader`";
        let first = invalid_tool_call_outcome(
            "shell_exec",
            error,
            "use only these tools: pwd, read_file",
            &mut counts,
        );
        let second = invalid_tool_call_outcome(
            "shell_exec",
            error,
            "use only these tools: pwd, read_file",
            &mut counts,
        );

        assert!(matches!(first, ToolOutcome::Failed(_)));
        assert!(matches!(second, ToolOutcome::InvalidRepeated(_)));
    }

    #[test]
    fn omits_tool_choice_after_tool_result() {
        let messages = vec![
            system_text_message(resolve_agent(Some("main")).unwrap().system_prompt),
            user_message("cwd"),
            tool_message("call_1".to_string(), "pwd".to_string(), "/tmp".to_string()),
        ];

        assert_eq!(tool_choice_for_round(&messages), None);
    }
}

pub async fn run_repl(
    config: RobConfig,
    agent_override: Option<String>,
    model_override: Option<String>,
    resume_id: Option<String>,
    approval_override: Option<ApprovalPolicy>,
) -> Result<()> {
    let mut session = AgentSession::new_interactive(
        config,
        agent_override,
        model_override,
        resume_id,
        approval_override,
    )?;
    let mut rl = DefaultEditor::new()?;

    println!("ROB Linux agent CLI. Type /help for commands, /exit to quit.");
    println!("Session: {}", session.id());
    println!("Provider: {}", session.config_summary()?);

    loop {
        let line = rl.readline("rob> ");
        match line {
            Ok(input) => {
                let input = input.trim();
                if input.is_empty() {
                    continue;
                }
                let _ = rl.add_history_entry(input);
                match input {
                    "/exit" | "/quit" => break,
                    "/help" => print_help(),
                    "/tools" => print_tools(&session)?,
                    "/config" => println!("{}", session.config_summary()?),
                    "/id" => println!("{}", session.id()),
                    _ => {
                        let response = session
                            .send_user_message_events(input, |event| {
                                match event {
                                    AgentEvent::AssistantDelta(delta) => {
                                        print!("{delta}");
                                        io::stdout().flush()?;
                                    }
                                    AgentEvent::ContextWindow {
                                        estimated_tokens,
                                        threshold,
                                        compacted,
                                    } => {
                                        let state = if compacted { "compacted" } else { "active" };
                                        eprintln!(
                                            "[context {state}: ~{estimated_tokens}/{threshold} tokens]"
                                        );
                                    }
                                    AgentEvent::ToolCallStarted {
                                        id,
                                        name,
                                        arguments,
                                    } => {
                                        eprintln!("[tool calling {name} #{id} args={arguments}]");
                                    }
                                    AgentEvent::ToolCallCompleted { id, name, .. } => {
                                        eprintln!("[tool called {name} #{id}]");
                                    }
                                    AgentEvent::ToolCallFailed { id, name, error } => {
                                        eprintln!("[tool failed {name} #{id}: {error}]");
                                    }
                                    AgentEvent::ToolCallDenied { id, name } => {
                                        eprintln!("[tool denied {name} #{id}]");
                                    }
                                }
                                Ok(())
                            })
                            .await?;
                        if !response.trim().is_empty() {
                            println!();
                        }
                    }
                }
            }
            Err(ReadlineError::Interrupted) => continue,
            Err(ReadlineError::Eof) => break,
            Err(error) => return Err(error.into()),
        }
    }

    Ok(())
}

fn print_help() {
    println!("/help    Show commands");
    println!("/tools   List tools available to the agent");
    println!("/config  Show active provider");
    println!("/id      Show current session id");
    println!("/exit    Quit");
}

fn print_tools(session: &AgentSession) -> Result<()> {
    for spec in session.agent_tools()? {
        println!("{} - {}", spec.function.name, spec.function.description);
    }
    Ok(())
}
