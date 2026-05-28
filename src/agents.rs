use crate::tools::{tool_specs_by_name, ToolSpec};
use anyhow::{anyhow, Result};

pub const MAIN_AGENT_NAME: &str = "main";

const MAIN_AGENT_PROMPT: &str =
    "You are ROB, a Linux-native CLI agent migrated from OpenOmniBot concepts. \
Use tools when they help inspect the local Linux environment. Keep answers concise. \
When using shell_exec, pass a command and argv array; never assume shell expansion.";

const READER_AGENT_PROMPT: &str = "You are ROB Reader, a read-only Linux inspection agent. \
Use read-only tools to inspect files, directories, and text. Do not attempt to execute shell commands.";

#[derive(Debug, Clone)]
pub struct AgentDefinition {
    pub name: &'static str,
    pub description: &'static str,
    pub system_prompt: &'static str,
    tool_names: &'static [&'static str],
}

impl AgentDefinition {
    pub fn tools(&self) -> Result<Vec<ToolSpec>> {
        tool_specs_by_name(self.tool_names)
    }

    pub fn tool_names(&self) -> &'static [&'static str] {
        self.tool_names
    }
}

pub fn main_agent() -> AgentDefinition {
    AgentDefinition {
        name: MAIN_AGENT_NAME,
        description: "Default Linux agent with the full built-in tool set.",
        system_prompt: MAIN_AGENT_PROMPT,
        tool_names: &["pwd", "list_dir", "read_file", "search_text", "shell_exec"],
    }
}

pub fn builtin_agents() -> Vec<AgentDefinition> {
    vec![
        main_agent(),
        AgentDefinition {
            name: "reader",
            description: "Read-only inspection agent without shell execution.",
            system_prompt: READER_AGENT_PROMPT,
            tool_names: &["pwd", "list_dir", "read_file", "search_text"],
        },
    ]
}

pub fn resolve_agent(name: Option<&str>) -> Result<AgentDefinition> {
    let name = name.unwrap_or(MAIN_AGENT_NAME);
    builtin_agents()
        .into_iter()
        .find(|agent| agent.name == name)
        .ok_or_else(|| {
            let available = builtin_agents()
                .into_iter()
                .map(|agent| agent.name)
                .collect::<Vec<_>>()
                .join(", ");
            anyhow!("agent `{name}` was not found; available agents: {available}")
        })
}

pub fn agent_for_system_prompt(prompt: Option<&str>) -> AgentDefinition {
    let Some(prompt) = prompt else {
        return main_agent();
    };
    builtin_agents()
        .into_iter()
        .find(|agent| agent.system_prompt == prompt)
        .unwrap_or_else(main_agent)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn main_agent_has_existing_prompt_and_tools() {
        let agent = resolve_agent(Some("main")).unwrap();
        let tools = agent.tools().unwrap();
        let tool_names = tools
            .iter()
            .map(|tool| tool.function.name.as_str())
            .collect::<Vec<_>>();

        assert!(agent.system_prompt.contains("You are ROB"));
        assert_eq!(
            tool_names,
            vec!["pwd", "list_dir", "read_file", "search_text", "shell_exec"]
        );
    }

    #[test]
    fn reader_agent_has_own_prompt_and_tool_subset() {
        let agent = resolve_agent(Some("reader")).unwrap();
        let tool_names = agent.tool_names();

        assert!(agent.system_prompt.contains("ROB Reader"));
        assert!(tool_names.contains(&"read_file"));
        assert!(!tool_names.contains(&"shell_exec"));
    }
}
