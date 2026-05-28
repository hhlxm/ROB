mod agent;
mod agents;
mod config;
mod context;
mod llm;
mod state;
mod tools;
mod tui;

use anyhow::Result;
use clap::{Parser, Subcommand};
use config::{ApprovalPolicy, ProviderProfile, ReasoningEffort, RobConfig};
use std::io::{self, Write};

#[derive(Parser)]
#[command(name = "rob")]
#[command(about = "OpenOmniBot-inspired Linux agent CLI")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Configure model providers.
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
    /// Start an interactive agent chat.
    Chat {
        /// Select the agent definition for this session.
        #[arg(long)]
        agent: Option<String>,
        /// Override the configured model for this session.
        #[arg(long)]
        model: Option<String>,
        /// Resume a saved session id.
        #[arg(long)]
        resume: Option<String>,
        /// Override the tool approval policy for this session.
        #[arg(long)]
        approval: Option<ApprovalPolicy>,
    },
    /// Start the terminal UI agent chat.
    Tui {
        /// Select the agent definition for this session.
        #[arg(long)]
        agent: Option<String>,
        /// Override the configured model for this session.
        #[arg(long)]
        model: Option<String>,
        /// Resume a saved session id.
        #[arg(long)]
        resume: Option<String>,
        /// Override the tool approval policy for this session.
        #[arg(long)]
        approval: Option<ApprovalPolicy>,
    },
    /// Send one message and print the assistant response.
    Ask {
        /// Message to send to the agent.
        message: String,
        /// Select the agent definition for this turn.
        #[arg(long)]
        agent: Option<String>,
        /// Override the configured model for this turn.
        #[arg(long)]
        model: Option<String>,
        /// Resume a saved session id.
        #[arg(long)]
        resume: Option<String>,
        /// Override the tool approval policy for this turn.
        #[arg(long)]
        approval: Option<ApprovalPolicy>,
    },
    /// Inspect or run local Linux tools.
    Tools {
        #[command(subcommand)]
        command: ToolsCommand,
    },
    /// Inspect available agent definitions.
    Agents {
        #[command(subcommand)]
        command: AgentCommand,
    },
    /// Inspect saved agent sessions.
    Sessions {
        #[command(subcommand)]
        command: SessionCommand,
    },
}

#[derive(Subcommand)]
enum ConfigCommand {
    /// Create or replace the active provider profile.
    Set {
        #[arg(long, default_value = "default")]
        name: String,
        #[arg(long)]
        base_url: String,
        #[arg(long)]
        model: String,
        #[arg(long)]
        api_key_env: Option<String>,
        #[arg(long)]
        api_key: Option<String>,
        #[arg(long, default_value = "openai-compatible")]
        protocol: String,
    },
    /// Show the active provider profile.
    Show,
    /// List configured provider profiles.
    List,
    /// Switch the active provider by name or id.
    Use {
        /// Provider profile name or id.
        profile: String,
    },
    /// Print the config file path.
    Path,
    /// Set the default tool approval policy.
    SetApproval {
        #[arg(value_enum)]
        policy: ApprovalPolicy,
    },
    /// Set context window management defaults.
    SetContext {
        /// Estimated prompt-token threshold before runtime compaction.
        #[arg(long)]
        token_threshold: usize,
        /// Number of recent messages to keep verbatim after compaction.
        #[arg(long)]
        recent_messages: usize,
    },
    /// Set reasoning/thinking effort for compatible providers.
    SetReasoning {
        #[arg(value_enum)]
        effort: ReasoningEffort,
    },
}

#[derive(Subcommand)]
enum ToolsCommand {
    /// List local tools available to the agent.
    List {
        /// Show tools for a specific agent instead of all built-in tools.
        #[arg(long)]
        agent: Option<String>,
    },
    /// Run a local tool manually with JSON arguments.
    Run {
        name: String,
        #[arg(default_value = "{}")]
        args: String,
    },
}

#[derive(Subcommand)]
enum AgentCommand {
    /// List built-in agents.
    List,
    /// Show one agent's prompt and tools.
    Show { name: String },
}

#[derive(Subcommand)]
enum SessionCommand {
    /// List saved sessions.
    List,
    /// Print a saved session as JSON.
    Show { id: String },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Config { command } => handle_config(command).await,
        Command::Chat {
            agent: agent_name,
            model,
            resume,
            approval,
        } => {
            let config = RobConfig::load_or_default()?;
            agent::run_repl(config, agent_name, model, resume, approval).await
        }
        Command::Tui {
            agent: agent_name,
            model,
            resume,
            approval,
        } => {
            let config = RobConfig::load_or_default()?;
            tui::run_tui(config, agent_name, model, resume, approval).await
        }
        Command::Ask {
            message,
            agent: agent_name,
            model,
            resume,
            approval,
        } => {
            let config = RobConfig::load_or_default()?;
            let mut session =
                agent::AgentSession::new(config, agent_name, model, resume, approval)?;
            let response = session
                .send_user_message_streaming(&message, |delta| {
                    print!("{delta}");
                    io::stdout().flush()?;
                    Ok(())
                })
                .await?;
            if !response.trim().is_empty() {
                println!();
            }
            eprintln!("session: {}", session.id());
            Ok(())
        }
        Command::Tools { command } => handle_tools(command).await,
        Command::Agents { command } => handle_agents(command).await,
        Command::Sessions { command } => handle_sessions(command).await,
    }
}

async fn handle_config(command: ConfigCommand) -> Result<()> {
    match command {
        ConfigCommand::Set {
            name,
            base_url,
            model,
            api_key_env,
            api_key,
            protocol,
        } => {
            let profile =
                ProviderProfile::new(name, base_url, model, api_key_env, api_key, protocol);
            let mut config = RobConfig::load_or_default()?;
            config.set_active_profile(profile);
            config.save()?;
            println!(
                "Saved active provider to {}",
                config::config_path()?.display()
            );
            Ok(())
        }
        ConfigCommand::Show => {
            let config = RobConfig::load_or_default()?;
            let profile = config.active_profile()?;
            println!("name: {}", profile.name);
            println!("base_url: {}", profile.base_url);
            println!("model: {}", profile.model);
            println!("protocol: {}", profile.protocol);
            println!("tool_approval: {}", config.tool_approval);
            println!("reasoning_effort: {}", config.reasoning.effort);
            println!(
                "context: token_threshold={} recent_messages={}",
                config.context.token_threshold, config.context.recent_messages
            );
            if let Some(env) = &profile.api_key_env {
                println!("api_key_env: {env}");
            }
            println!("configured: {}", profile.resolve_api_key().is_ok());
            Ok(())
        }
        ConfigCommand::List => {
            let config = RobConfig::load_or_default()?;
            if config.profiles.is_empty() {
                println!("No provider profiles configured.");
                return Ok(());
            }
            for profile in &config.profiles {
                let active = if profile.id == config.active_profile_id {
                    "*"
                } else {
                    " "
                };
                println!(
                    "{active} {} ({}) model={} base_url={}",
                    profile.name, profile.id, profile.model, profile.base_url
                );
            }
            Ok(())
        }
        ConfigCommand::Use { profile } => {
            let mut config = RobConfig::load_or_default()?;
            config.set_active_profile_by_ref(&profile)?;
            config.save()?;
            println!("Active provider set to {profile}");
            Ok(())
        }
        ConfigCommand::SetApproval { policy } => {
            let mut config = RobConfig::load_or_default()?;
            config.tool_approval = policy;
            config.save()?;
            println!("Default tool approval set to {}", config.tool_approval);
            Ok(())
        }
        ConfigCommand::SetContext {
            token_threshold,
            recent_messages,
        } => {
            let mut config = RobConfig::load_or_default()?;
            config.context.token_threshold = token_threshold.max(1);
            config.context.recent_messages = recent_messages.max(1);
            config.save()?;
            println!(
                "Context window set to threshold={} recent_messages={}",
                config.context.token_threshold, config.context.recent_messages
            );
            Ok(())
        }
        ConfigCommand::SetReasoning { effort } => {
            let mut config = RobConfig::load_or_default()?;
            config.reasoning.effort = effort;
            config.save()?;
            println!("Reasoning effort set to {}", config.reasoning.effort);
            Ok(())
        }
        ConfigCommand::Path => {
            println!("{}", config::config_path()?.display());
            Ok(())
        }
    }
}

async fn handle_tools(command: ToolsCommand) -> Result<()> {
    match command {
        ToolsCommand::List { agent } => {
            let tools = if let Some(agent) = agent {
                agents::resolve_agent(Some(&agent))?.tools()?
            } else {
                tools::tool_specs()
            };
            for tool in tools {
                println!("{} - {}", tool.function.name, tool.function.description);
            }
            Ok(())
        }
        ToolsCommand::Run { name, args } => {
            let args: serde_json::Value = serde_json::from_str(&args)?;
            let result = tools::run_tool(&name, args).await?;
            println!("{result}");
            Ok(())
        }
    }
}

async fn handle_agents(command: AgentCommand) -> Result<()> {
    match command {
        AgentCommand::List => {
            for agent in agents::builtin_agents() {
                println!(
                    "{} - {} tools={}",
                    agent.name,
                    agent.description,
                    agent.tool_names().join(",")
                );
            }
            Ok(())
        }
        AgentCommand::Show { name } => {
            let agent = agents::resolve_agent(Some(&name))?;
            println!("name: {}", agent.name);
            println!("description: {}", agent.description);
            println!("tools: {}", agent.tool_names().join(", "));
            println!("prompt:\n{}", agent.system_prompt);
            Ok(())
        }
    }
}

async fn handle_sessions(command: SessionCommand) -> Result<()> {
    match command {
        SessionCommand::List => {
            let sessions = state::list_sessions()?;
            if sessions.is_empty() {
                println!("No saved sessions.");
                return Ok(());
            }
            for item in sessions {
                println!(
                    "{} messages={} updated_at={}",
                    item.id, item.message_count, item.updated_at
                );
            }
            Ok(())
        }
        SessionCommand::Show { id } => {
            let session = state::load_session(&id)?;
            println!("{}", serde_json::to_string_pretty(&session)?);
            Ok(())
        }
    }
}
