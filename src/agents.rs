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
将用户关于文件、文档、笔记、安防、相册照片、影音的请求解析成一个最匹配的专用工具调用；不要使用 Linux shell 工具。\
工具路由：文件管理用 digital_file_manager；摘要、文档元信息、翻译、OCR、写作辅助、结构化提取用 digital_document_assistant；笔记标签/关联/检索用 digital_note_knowledge；安防事件用 digital_security_event_query，身份出现判断用 digital_security_identity_recognition，摄像头控制用 digital_security_camera_control；相册查询/创建用 digital_photo_album，单张照片元信息用 digital_photo_metadata；字幕用 digital_media_subtitle；影视播放意图用 digital_video_playback；音乐播放意图用 digital_music_playback；当前播放暂停/继续只用 digital_media_transport_control。\
同一请求通常只调用一个工具；除非用户明确给出多个独立任务，不要拆成多次工具调用。\
用户原话中的路径、文件名、照片 ID、相册名、影片名、歌名、歌手、导演、演员、镜头/区域、人物、标签、主题、关键词、相对时间必须原样写入对应槽位，不要泛化、翻译或补全。相对时间写入 time_query；明确镜头/区域写入 camera_name 或 area；全局安防查询可填 area=全屋。\
安防边界：人员/行为/车辆/车牌/包裹/宠物/野生动物/烟火/声音都属于事件查询；爸爸、妈妈、张三等熟人或陌生人是否出现属于身份识别；隐私模式、抓拍、音量、对讲、布防/撤防属于摄像头控制。\
影音边界：影视和音乐分开；只有没有标题、人物、类型、语言、地区、片单、歌单等限制时才用推荐类 action。出现类型、语言、地区、片单/歌单、收藏、最近播放、集数/曲目切换时，按工具 schema 选择相应 action。暂停/继续当前播放不要理解为最近播放。\
“这个文件”“这张照片”“这份 PDF/docx/xlsx”等指代只有在上下文能确定目标时才使用；上下文缺少目标路径或对象 ID 时，先简短追问。\
普通闲聊或不属于本 agent 能力范围时直接简短回答。工具返回 mock payload 时，只说明已提交或查到的意图，不要声称真实后端已完成不可验证的操作。";

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
                "digital_document_assistant",
                "digital_note_knowledge",
                "digital_security_event_query",
                "digital_security_identity_recognition",
                "digital_security_camera_control",
                "digital_photo_album",
                "digital_photo_metadata",
                "digital_media_subtitle",
                "digital_video_playback",
                "digital_media_transport_control",
                "digital_music_playback",
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
        assert!(agent.system_prompt.contains("digital_document_assistant"));
        assert!(agent
            .system_prompt
            .contains("digital_security_camera_control"));
        assert!(agent.tool_names().contains(&"digital_file_manager"));
        assert!(agent.tool_names().contains(&"digital_document_assistant"));
        assert!(agent.tool_names().contains(&"digital_photo_album"));
        assert!(agent.tool_names().contains(&"digital_security_event_query"));
        assert!(agent
            .tool_names()
            .contains(&"digital_security_identity_recognition"));
        assert!(agent
            .tool_names()
            .contains(&"digital_security_camera_control"));
        assert!(agent.tool_names().contains(&"digital_media_subtitle"));
        assert!(agent.tool_names().contains(&"digital_video_playback"));
        assert!(agent.tool_names().contains(&"digital_music_playback"));
        assert!(agent
            .tool_names()
            .contains(&"digital_media_transport_control"));
        assert_eq!(agent.tool_names().len(), 12);
        assert!(!agent.tool_names().contains(&"digital_media_control"));
        assert!(!agent.tool_names().contains(&"digital_invoice_extract"));
        assert!(!agent.tool_names().contains(&"digital_contract_extract"));
        assert!(!agent.tool_names().contains(&"digital_receipt_extract"));
        assert!(agent.tool_names().contains(&"digital_note_knowledge"));
        assert!(!agent.tool_names().contains(&"shell_exec"));
    }
}
