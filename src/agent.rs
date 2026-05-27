use crate::config::{ApprovalPolicy, RobConfig};
use crate::llm::{
    assistant_message, chat_completion, content_as_text, parse_tool_arguments, system_message,
    tool_message, user_message, ChatMessage,
};
use crate::state;
use crate::tools::{run_tool, tool_specs};
use anyhow::Result;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use serde_json::Value;
use std::io::{self, Write};
use uuid::Uuid;

const MAX_TOOL_ROUNDS: usize = 6;

pub struct AgentSession {
    id: String,
    config: RobConfig,
    messages: Vec<ChatMessage>,
    approval_policy: ApprovalPolicy,
    interactive_approval: bool,
}

impl AgentSession {
    pub fn new(
        config: RobConfig,
        model_override: Option<String>,
        resume_id: Option<String>,
        approval_override: Option<ApprovalPolicy>,
    ) -> Result<Self> {
        Self::new_inner(config, model_override, resume_id, approval_override, false)
    }

    fn new_interactive(
        config: RobConfig,
        model_override: Option<String>,
        resume_id: Option<String>,
        approval_override: Option<ApprovalPolicy>,
    ) -> Result<Self> {
        Self::new_inner(config, model_override, resume_id, approval_override, true)
    }

    fn new_inner(
        mut config: RobConfig,
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
        let (id, messages) = if let Some(id) = resume_id {
            let record = state::load_session(&id)?;
            (record.id, record.messages)
        } else {
            (Uuid::new_v4().to_string(), vec![system_message()])
        };

        Ok(Self {
            id,
            config,
            messages,
            approval_policy,
            interactive_approval,
        })
    }

    pub async fn send_user_message(&mut self, input: &str) -> Result<String> {
        self.messages.push(user_message(input));
        self.persist()?;

        for _ in 0..MAX_TOOL_ROUNDS {
            let profile = self.config.active_profile()?;
            let tools = tool_specs();
            let model_message = chat_completion(profile, &self.messages, &tools).await?;
            let tool_calls = model_message.tool_calls.clone().unwrap_or_default();
            let assistant = assistant_message(model_message);
            let answer = content_as_text(assistant.content.clone());
            self.messages.push(assistant);

            if tool_calls.is_empty() {
                self.persist()?;
                return Ok(answer);
            }

            for call in tool_calls {
                let args = parse_tool_arguments(&call.function.arguments);
                let output = if self.approve_tool(&call.function.name, &args)? {
                    match run_tool(&call.function.name, args).await {
                        Ok(output) => output,
                        Err(error) => format!("tool error: {error:#}"),
                    }
                } else {
                    "tool denied by approval policy".to_string()
                };
                self.messages
                    .push(tool_message(call.id, call.function.name, output));
                self.persist()?;
            }
        }

        self.persist()?;
        Ok("Stopped after reaching the tool-call round limit.".to_string())
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn transcript(&self) -> &[ChatMessage] {
        &self.messages
    }

    pub fn config_summary(&self) -> Result<String> {
        let profile = self.config.active_profile()?;
        Ok(format!(
            "{} | {} | {} | {} | approval={}",
            profile.name, profile.base_url, profile.model, profile.protocol, self.approval_policy
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
}

pub async fn run_repl(
    config: RobConfig,
    model_override: Option<String>,
    resume_id: Option<String>,
    approval_override: Option<ApprovalPolicy>,
) -> Result<()> {
    let mut session =
        AgentSession::new_interactive(config, model_override, resume_id, approval_override)?;
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
                    "/tools" => print_tools(),
                    "/config" => println!("{}", session.config_summary()?),
                    "/id" => println!("{}", session.id()),
                    _ => {
                        let response = session.send_user_message(input).await?;
                        if !response.trim().is_empty() {
                            println!("{response}");
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

fn print_tools() {
    for spec in tool_specs() {
        println!("{} - {}", spec.function.name, spec.function.description);
    }
}
