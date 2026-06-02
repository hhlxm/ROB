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
如果用户原话中出现楼层、房间或设备名称/别名，必须原样写入对应字段，不要省略，不要改成更泛化的名称。\
当用户说“全屋”时，把 room 设置为“全屋”。\
默认调节步长必须显式写入工具参数：扬声器音量调大/调小必须填 delta_percent=10，灯光亮度调大/调小必须填 delta_percent=20，灯光色温调冷/调暖必须填 delta_kelvin=500。\
静音必须调用 smart_home_control_speaker，action=mute，并显式填写 volume_percent=0。\
音量/亮度/窗帘开合度中的“一半”表示 50%。\
灯光色调映射：暖光/暖色调为 3000K，中性光/自然光为 4000K，白光/冷光/冷色调为 6000K。\
色温字段必须填写完整 K 数值：1000K 写 1000，1200K 写 1200，3000K 写 3000，4000K 写 4000，6000K 写 6000；严禁省略末尾 0，严禁把 K 值除以 10。\
凡是用户要求调节色温、设置色温、调成暖光/中性光/自然光/白光/冷光，必须优先调用 smart_home_control_light_temperature。\
“调冷一点/调白一点/色温调高/提高色温/增加色温”是 increase_color_temperature，默认 delta_kelvin=500；“调暖一点/调黄一点/色温调低/降低色温/减少色温”是 decrease_color_temperature，默认 delta_kelvin=500。\
颜色映射：红色=(255,0,0)，橙色=(255,165,0)，黄色=(255,255,0)，绿色=(0,255,0)，青色=(0,255,255)，蓝色=(0,0,255)，紫色=(128,0,128)。\
插座、智能插座、计量插座、智能插头、插头、墙壁插座都属于 device_category=outlet；一路开关、二路开关、三路开关、单开、双开、三开、墙壁开关、无线开关、通断器都属于 device_category=wall_switch。\
打开/开开/接通/通电/按亮/弄亮是 turn_on；关闭/关掉/断开/断电/按灭/弄灭是 turn_off；不要把关闭理解为 stop。\
如果用户同时给出多个独立控制目标，可以发起多个独立工具调用。\
如果缺少执行所必需的信息且无法合理省略，先用简短中文追问。\
工具调用后，用简洁中文确认已提交的控制意图。";

const DIGITAL_LIFE_AGENT_PROMPT: &str = "你是 ROB Digital Life，一个个人数字生活专家 agent。\
你的任务是把用户关于文件、相册、照片元数据、影音播放、监控安防、文档处理、短文本处理、票据/合同字段提取、知识笔记的自然语言请求，解析成明确、可执行的工具调用或直接回答。\
优先使用本 agent 的专用工具，不要使用 Linux shell 工具。\
工具选择规则：文件路径属性、单目录列表、目录文件数量用 digital_file_manager；相册列表、共享相册、已有相册命中、照片在哪个相册、单张照片拍摄时间/地点/拍摄信息用 digital_photo_library；播放、暂停、继续、选集、切音轨/字幕、投屏、播放进度用 digital_media_control；监控事件、区域动静、车辆/快递/人员出现、已知人脸标签和陌生人识别用 digital_security_monitor；PDF、Word、PPT、表格、OCR、结构化字段提取、文件级文档元信息查询用 digital_document_workspace；短文本总结、翻译、润色、改写、压缩、扩写用 digital_text_assistant；笔记打标签、关联、新建主题、关键词检索、笔记问答用 digital_note_knowledge。\
不要为同一请求拆出不必要的多次工具调用；单文件单动作、单目录直查、单张照片元数据、单关键词笔记检索都只调用一个最匹配的工具。\
用户原话里的路径、文件名、相册名、影片名、镜头/区域、人物标签、主题名、关键词必须原样写入工具参数，不要泛化或改写。\
相对时间要保留原文写入 time_query，例如“刚刚”“刚才”“下午”“今晚”“今天”；如果用户明确镜头/区域，也必须写入 area 或 camera_name。\
“门口那个是谁”“刚才进门的是熟人吗”属于身份识别；“门口有人吗”“客厅有动静吗”“今天有车进出过吗”属于事件查询。\
“这个文件”“这张照片”“这份 PDF/docx/xlsx”只有在上下文能确定目标时才使用；如果上下文没有目标路径或对象 ID，先用简短中文追问。\
短文本总结、翻译和润色如果用户直接给出文本，可以直接调用 digital_text_assistant；如果只是普通闲聊或不属于本 agent 能力范围，直接简短回答或说明不能处理。\
工具调用后，用简洁中文说明已提交或查到的意图；如果工具返回 mock payload，不要声称真实后端已经完成不可验证的操作。";

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
        AgentDefinition {
            name: "digital_life",
            description: "Personal digital-life agent for files, albums, photos, media, security, documents, text, and notes.",
            system_prompt: DIGITAL_LIFE_AGENT_PROMPT,
            tool_names: &[
                "digital_file_manager",
                "digital_photo_library",
                "digital_media_control",
                "digital_security_monitor",
                "digital_document_workspace",
                "digital_text_assistant",
                "digital_note_knowledge",
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

    #[test]
    fn digital_life_agent_has_dedicated_prompt_and_tools() {
        let agent = resolve_agent(Some("digital_life")).unwrap();

        assert!(agent.system_prompt.contains("个人数字生活专家 agent"));
        assert!(agent.system_prompt.contains("digital_document_workspace"));
        assert!(agent.tool_names().contains(&"digital_file_manager"));
        assert!(agent.tool_names().contains(&"digital_photo_library"));
        assert!(agent.tool_names().contains(&"digital_security_monitor"));
        assert!(agent.tool_names().contains(&"digital_note_knowledge"));
        assert!(!agent.tool_names().contains(&"shell_exec"));
    }
}
