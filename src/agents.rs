use crate::tools::{tool_specs_by_name, ToolSpec};
use anyhow::{anyhow, Result};

pub const MAIN_AGENT_NAME: &str = "main";

const MAIN_AGENT_PROMPT: &str =
    "You are ROB, a Linux-native CLI agent migrated from OpenOmniBot concepts. \
Use tools when they help inspect the local Linux environment. Keep answers concise. \
When using shell_exec, pass a command and argv array; never assume shell expansion.";

const READER_AGENT_PROMPT: &str = "You are ROB Reader, a read-only Linux inspection agent. \
Use read-only tools to inspect files, directories, and text. Do not attempt to execute shell commands.";

const SMART_HOME_AGENT_PROMPT: &str = "你是 ROB Smart Home，一个智能家居控制 agent。\
你的任务是把用户的自然语言指令解析成明确、可执行的智能家居工具调用。\
优先使用智能家居专用工具，不要使用 Linux shell 工具。\
当用户要控制设备时，提取楼层、房间、设备名称、动作和值；用户未说明楼层或房间时不要编造，可省略对应字段。\
当用户说“全屋”时，把 room 设置为“全屋”。\
默认调节步长：扬声器音量调大/调小为 10%，灯光亮度调大/调小为 20%，灯光色温调冷/调暖为 500K。\
灯光色调映射：暖光/暖色调为 3000K，中性光/自然光为 4000K，白光/冷光/冷色调为 6000K。\
色温字段必须填写完整 K 数值：1000K 写 1000，1200K 写 1200，3000K 写 3000，4000K 写 4000，6000K 写 6000；严禁省略末尾 0，严禁把 K 值除以 10。\
凡是用户要求调节色温、设置色温、调成暖光/中性光/自然光/白光/冷光，必须优先调用 smart_home_control_light_temperature。\
颜色映射：红色=(255,0,0)，橙色=(255,165,0)，黄色=(255,255,0)，绿色=(0,255,0)，青色=(0,255,255)，蓝色=(0,0,255)，紫色=(128,0,128)。\
如果用户同时给出多个独立控制目标，可以发起多个独立工具调用。\
如果缺少执行所必需的信息且无法合理省略，先用简短中文追问。\
工具调用后，用简洁中文确认已提交的控制意图。";

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
        AgentDefinition {
            name: "smart_home",
            description: "Smart home control agent for lights, curtains, speakers, outlets, switches, and scenes.",
            system_prompt: SMART_HOME_AGENT_PROMPT,
            tool_names: &[
                "smart_home_control_speaker",
                "smart_home_control_light_temperature",
                "smart_home_control_light",
                "smart_home_control_curtain",
                "smart_home_control_power",
                "smart_home_control_scene",
            ],
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

    #[test]
    fn smart_home_agent_has_dedicated_prompt_and_tools() {
        let agent = resolve_agent(Some("smart_home")).unwrap();

        assert!(agent.system_prompt.contains("智能家居控制 agent"));
        assert!(agent.system_prompt.contains("严禁把 K 值除以 10"));
        assert!(agent
            .system_prompt
            .contains("smart_home_control_light_temperature"));
        assert!(agent
            .tool_names()
            .contains(&"smart_home_control_light_temperature"));
        assert!(agent.tool_names().contains(&"smart_home_control_light"));
        assert!(!agent.tool_names().contains(&"shell_exec"));
    }
}
