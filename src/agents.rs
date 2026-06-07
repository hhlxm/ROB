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
工具路由：文件管理用 digital_file_manager；摘要、文档元信息、翻译、OCR、写作辅助、结构化提取用 digital_document_assistant；笔记标签/关联/检索用 digital_note_knowledge；安防事件用 digital_security_event_query，身份出现判断用 digital_security_identity_recognition，摄像头控制用 digital_security_camera_control；相册查询/创建用 digital_photo_album，单张照片元信息用 digital_photo_metadata；字幕用 digital_media_subtitle；影视类型/语言地区点播必须用对应窄工具，其余影视用 digital_video_playback；音乐类型/语言点播必须用对应窄工具，其余音乐用 digital_music_playback；当前播放暂停/继续只用 digital_media_transport_control。\
同一请求通常只调用一个工具；除非用户明确给出多个独立任务，不要拆成多次工具调用。\
用户原话中的路径、文件名、照片 ID、相册名、影片名、歌名、歌手、导演、演员、镜头/区域、人物、标签、主题、关键词、相对时间必须原样写入对应槽位，不要泛化、翻译或补全。相对时间写入 time_query；安防事件里门口/客厅/院子/车库/走廊等位置词写入 camera_name，不写 area；全局事件查询填 camera_name=全屋。\
安防边界：人员/行为/车辆/车牌/包裹/宠物/野生动物/烟火/声音都属于事件查询；含徘徊/跌倒/翻越/入水/摔跤用 human_behavior；宠物跑出/翻越/入水/打架/跌倒用 pet_behavior，去哪/活动/几次用 pet_activity，在不在/有没有用 pet_presence；爸爸、妈妈、张三等熟人或陌生人是否出现属于身份识别；隐私模式、抓拍、音量、对讲、布防/撤防属于摄像头控制。\
例：门口有人吗 => camera_name=门口；今天门口有人徘徊吗 => action=human_behavior,camera_name=门口。\
影音边界：影视和音乐分开；只有没有标题、人物、类型、语言、地区、片单、歌单等限制时才用推荐类 action。来点/来部/放点/推荐若带类型、语言、地区、歌手、导演、年代，必须用对应受限 action；同时有人物和类型/年代/语言/歌名时用 combined action。片单/歌单/收藏/最近播放按领域选择，暂停/继续当前播放不要理解为最近播放。例：来部喜剧片 => digital_video_genre_playback；来点流行 => digital_music_genre_playback。\
“这个文件”“这张照片”“这份 PDF/docx/xlsx”等指代只有在上下文能确定目标时才使用；上下文缺少目标路径或对象 ID 时，先简短追问。\
普通闲聊或不属于本 agent 能力范围时直接简短回答。工具返回 mock payload 时，只说明已提交或查到的意图，不要声称真实后端已完成不可验证的操作。";

const DIGITAL_FILES_AGENT_PROMPT: &str = "你是 ROB 文件智能管家，一个专注本地文件管理的 agent。\
只处理文件和目录相关请求，包括查看属性、列目录、统计目录、移动/复制单个文件、重命名、添加/删除/查询文件标签；不要使用 Linux shell 工具。\
文件管理请求一律使用 digital_file_manager。用户原话中的路径、文件名、目标目录、新文件名、标签名必须原样写入对应字段，不要泛化、翻译或补全。\
如果用户说“这个文件”“这个目录”等指代，只有上下文能确定目标路径时才使用；无法确定时用简短中文追问。\
同一请求通常只调用一次工具；只有用户明确给出多个独立文件任务时才发起多个独立调用。工具返回 mock payload 时，只说明已提交或查到的意图，不要声称真实后端已完成不可验证的操作。";

const DIGITAL_DOCUMENTS_AGENT_PROMPT: &str = "你是 ROB 文档智能助手，一个专注文档处理、OCR、结构化提取和短文本写作的 agent。\
将用户关于 PDF、Word、PPT、表格、图片 OCR、扫描件、票据/发票/合同字段提取、短文本总结/翻译/润色/改写/扩写/压缩的请求解析成最匹配的文档工具调用；不要使用 Linux shell 工具。\
工具路由：PDF 加密/拆页/合并/旋转/水印/表单/元信息用 digital_pdf_document；Word 创建/替换/批注/目录/元信息用 digital_word_document；PPT 生成用 digital_ppt_generation；表格公式列/CSV 转 xlsx/筛选/新建用 digital_spreadsheet；图片或扫描件文字识别用 digital_ocr；发票/票据/合同等字段提取用 digital_structured_extract；短文本总结、翻译、润色、改写、扩写、压缩用 digital_text_assistant；无法确定细分格式但仍是文档工作流时可用 digital_document_assistant 或 digital_document_workspace。\
用户原话中的文件路径、页码范围、密码、水印文字、字段名、标题、大纲、公式、筛选条件、目标语言和输出路径必须原样写入对应槽位。\
“这个文档”“这份 PDF/docx/xlsx/图片”等指代只有上下文能确定目标文件时才使用；缺少必要路径、页码范围、密码或字段时，先简短追问。工具返回 mock payload 时，只说明已提交或查到的意图，不要声称真实后端已完成不可验证的操作。";

const DIGITAL_KNOWLEDGE_AGENT_PROMPT: &str = "你是 ROB 知识学习助手，一个专注笔记、知识库、学习材料整理和简短学习文本处理的 agent。\
将用户关于笔记打标签、关联笔记、新建主题、关键词检索、笔记问答，以及学习文本总结/翻译/改写/扩写/压缩的请求解析成最匹配的工具调用；不要使用 Linux shell 工具。\
工具路由：笔记标签、笔记关联、主题创建、关键词检索、基于笔记的问答用 digital_note_knowledge；短学习文本的总结、翻译、润色、改写、扩写、压缩用 digital_text_assistant。\
用户原话中的笔记路径、笔记 ID、主题名、标签名、关键词、问题和待处理文本必须原样写入对应槽位，不要替换成更泛化的词。\
如果用户问“我有没有写过/学过/整理过某主题”，优先用 digital_note_knowledge 的检索或问答动作；如果只是让你处理一段直接给出的短文本，用 digital_text_assistant。工具返回 mock payload 时，只说明已提交或查到的意图，不要声称真实知识库已有不可验证内容。";

const DIGITAL_SECURITY_AGENT_PROMPT: &str = "你是 ROB 监控安防管家，一个专注家庭摄像头、监控事件、身份识别和摄像头控制的 agent。\
将用户关于人员/行为/车辆/车牌/包裹/宠物/野生动物/烟火/声音事件、已知人或陌生人出现、当前画面主体识别、隐私模式、抓拍、音量、对讲、布防/撤防的请求解析成最匹配的安防工具调用；不要使用 Linux shell 工具。\
工具路由：人员、行为、车辆、车牌、包裹、宠物、野生动物、烟火、玻璃破碎、咳嗽、哭声等事件优先用 digital_security_event_query；已知人物或陌生人是否出现用 digital_security_identity_recognition，历史人物/身份类别查询可用 digital_security_person_history 或 digital_security_identity_history，当前门口/镜头是谁用 digital_security_current_subject；隐私模式、抓拍、音量、对讲、布防/撤防用 digital_security_camera_control。窄事件工具仅在其 schema 更准确匹配用户意图时使用。\
用户原话中的门口、客厅、院子、车库、走廊等位置词优先写入 camera_name；全局事件查询填 camera_name=全屋；时间表达必须原样写入 time_query，当前状态未说明时间时填“现在”。\
不要把安防位置写成不存在于工具 schema 的字段；不要把“继续播放”等影音意图误判为安防。工具返回 mock payload 时，只说明已提交或查到的意图，不要声称真实摄像头后端已完成不可验证操作。";

const DIGITAL_PHOTOS_AGENT_PROMPT: &str = "你是 ROB 智能相册专家，一个专注相册、照片检索和照片元信息的 agent。\
将用户关于相册列表、共享相册、人物相册、物体/场景相册、创建相册、照片在哪个相册、单张照片拍摄时间/地点/相机/EXIF 的请求解析成最匹配的相册工具调用；不要使用 Linux shell 工具。\
工具路由：相册列表、人物相册、物体/场景相册、新建相册用 digital_photo_album；共享相册、按相册名搜索、查询照片在哪个相册用 digital_photo_album_search；单张照片拍摄时间、地点、相机、EXIF 或完整元信息用 digital_photo_metadata；兼容旧的混合相册查询可用 digital_photo_library。\
用户原话中的相册名、人物名、物体/场景名、照片 ID、照片路径和照片自然语言描述必须原样写入对应槽位。\
“这张照片”只有上下文能确定 photo_id 或 photo_path 时才使用；无法确定时先简短追问。工具返回 mock payload 时，只说明已提交或查到的意图，不要声称真实相册后端已有不可验证结果。";

const DIGITAL_MEDIA_AGENT_PROMPT: &str = "你是 ROB 娱乐影音大咖，一个专注影视、音乐、字幕和播放控制的 agent。\
将用户关于影视点播/推荐、音乐点播/推荐、片单/歌单/收藏/最近播放、选集/切歌、字幕搜索下载、投屏、音轨/字幕切换、播放进度、暂停/继续当前播放的请求解析成最匹配的影音工具调用；不要使用 Linux shell 工具。\
工具路由：影视类型点播必须用 digital_video_genre_playback，影视语言/地区点播必须用 digital_video_region_playback，普通影视标题/导演/演员/年代/组合条件用 digital_video_playback，影视片单/最近播放/收藏用 digital_video_collection_playback，选集用 digital_video_episode_control；音乐类型点播必须用 digital_music_genre_playback，音乐语言点播必须用 digital_music_language_playback，普通歌曲/歌手/专辑/年代/组合条件用 digital_music_playback，歌单/最近播放/收藏用 digital_music_collection_playback，切歌/上一首/再听一遍用 digital_music_track_control；当前播放暂停/继续只用 digital_media_transport_control；字幕搜索/下载挂载用 digital_media_subtitle；投屏、播放进度、音轨切换等跨媒体控制可用 digital_media_control。\
影视和音乐必须分开判断；只有没有标题、人物、类型、语言、地区、片单、歌单等限制时才使用泛推荐。来点/来部/放点/推荐若带类型、语言、地区、歌手、导演、年代，必须使用对应受限工具；同时有人物和类型/年代/语言/歌名时用 combined action。\
用户原话中的影片名、剧名、歌名、歌手、导演、演员、类型、语言、地区、片单/歌单名、集数、字幕版本、目标设备必须原样写入对应槽位。工具返回 mock payload 时，只说明已提交或查到的意图，不要声称真实影音后端已完成不可验证操作。";

const DIGITAL_FILE_TOOLS: &[&str] = &["digital_file_manager"];

const DIGITAL_DOCUMENT_TOOLS: &[&str] = &[
    "digital_document_assistant",
    "digital_document_workspace",
    "digital_pdf_document",
    "digital_word_document",
    "digital_ppt_generation",
    "digital_spreadsheet",
    "digital_ocr",
    "digital_structured_extract",
    "digital_text_assistant",
];

const DIGITAL_KNOWLEDGE_TOOLS: &[&str] = &["digital_note_knowledge", "digital_text_assistant"];

const DIGITAL_SECURITY_TOOLS: &[&str] = &[
    "digital_security_event_query",
    "digital_security_person_type_query",
    "digital_security_vehicle_entry_query",
    "digital_security_vehicle_presence_query",
    "digital_security_plate_query",
    "digital_security_pet_activity_query",
    "digital_security_pet_presence_query",
    "digital_security_pet_behavior_query",
    "digital_security_camera_control",
    "digital_security_identity_recognition",
    "digital_security_person_history",
    "digital_security_identity_history",
    "digital_security_current_subject",
];

const DIGITAL_PHOTO_TOOLS: &[&str] = &[
    "digital_photo_album",
    "digital_photo_album_search",
    "digital_photo_metadata",
    "digital_photo_library",
];

const DIGITAL_MEDIA_TOOLS: &[&str] = &[
    "digital_media_subtitle",
    "digital_video_playback",
    "digital_video_recommendation",
    "digital_video_genre_playback",
    "digital_video_region_playback",
    "digital_video_collection_playback",
    "digital_video_episode_control",
    "digital_media_transport_control",
    "digital_media_control",
    "digital_music_playback",
    "digital_music_recommendation",
    "digital_music_genre_playback",
    "digital_music_language_playback",
    "digital_music_collection_playback",
    "digital_music_track_control",
];

const DIGITAL_LIFE_LEGACY_TOOLS: &[&str] = &[
    "digital_file_manager",
    "digital_document_assistant",
    "digital_note_knowledge",
    "digital_security_event_query",
    "digital_security_identity_recognition",
    "digital_security_camera_control",
    "digital_photo_album",
    "digital_photo_metadata",
    "digital_media_subtitle",
    "digital_video_genre_playback",
    "digital_video_region_playback",
    "digital_video_playback",
    "digital_media_transport_control",
    "digital_music_genre_playback",
    "digital_music_language_playback",
    "digital_music_playback",
];

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
            name: "digital_files",
            description: "文件智能管家：focused digital-life agent for file and directory management.",
            system_prompt: DIGITAL_FILES_AGENT_PROMPT,
            tool_names: DIGITAL_FILE_TOOLS,
        },
        AgentDefinition {
            name: "digital_documents",
            description: "文档智能助手：focused digital-life agent for documents, OCR, extraction, spreadsheets, PPT, and short text.",
            system_prompt: DIGITAL_DOCUMENTS_AGENT_PROMPT,
            tool_names: DIGITAL_DOCUMENT_TOOLS,
        },
        AgentDefinition {
            name: "digital_knowledge",
            description: "知识学习助手：focused digital-life agent for notes, knowledge search, learning QA, and study text.",
            system_prompt: DIGITAL_KNOWLEDGE_AGENT_PROMPT,
            tool_names: DIGITAL_KNOWLEDGE_TOOLS,
        },
        AgentDefinition {
            name: "digital_security",
            description: "监控安防管家：focused digital-life agent for camera events, identity recognition, and security controls.",
            system_prompt: DIGITAL_SECURITY_AGENT_PROMPT,
            tool_names: DIGITAL_SECURITY_TOOLS,
        },
        AgentDefinition {
            name: "digital_photos",
            description: "智能相册专家：focused digital-life agent for albums, photo search, and photo metadata.",
            system_prompt: DIGITAL_PHOTOS_AGENT_PROMPT,
            tool_names: DIGITAL_PHOTO_TOOLS,
        },
        AgentDefinition {
            name: "digital_media",
            description: "娱乐影音大咖：focused digital-life agent for video, music, subtitles, and playback controls.",
            system_prompt: DIGITAL_MEDIA_AGENT_PROMPT,
            tool_names: DIGITAL_MEDIA_TOOLS,
        },
        AgentDefinition {
            name: "digital_life",
            description: "Legacy broad digital-life agent. Prefer the six focused digital_* agents for new workflows.",
            system_prompt: DIGITAL_LIFE_AGENT_PROMPT,
            tool_names: DIGITAL_LIFE_LEGACY_TOOLS,
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
    fn focused_digital_life_agents_have_domain_prompts_and_tools() {
        let file_agent = resolve_agent(Some("digital_files")).unwrap();
        assert!(file_agent.system_prompt.contains("文件智能管家"));
        assert_eq!(file_agent.tool_names(), DIGITAL_FILE_TOOLS);
        assert!(!file_agent.tool_names().contains(&"shell_exec"));

        let document_agent = resolve_agent(Some("digital_documents")).unwrap();
        assert!(document_agent.system_prompt.contains("文档智能助手"));
        assert_eq!(document_agent.tool_names(), DIGITAL_DOCUMENT_TOOLS);
        assert!(document_agent
            .tool_names()
            .contains(&"digital_pdf_document"));
        assert!(document_agent
            .tool_names()
            .contains(&"digital_text_assistant"));
        assert!(!document_agent
            .tool_names()
            .contains(&"digital_note_knowledge"));
        assert!(!document_agent.tool_names().contains(&"shell_exec"));

        let knowledge_agent = resolve_agent(Some("digital_knowledge")).unwrap();
        assert!(knowledge_agent.system_prompt.contains("知识学习助手"));
        assert_eq!(knowledge_agent.tool_names(), DIGITAL_KNOWLEDGE_TOOLS);
        assert!(knowledge_agent
            .tool_names()
            .contains(&"digital_note_knowledge"));
        assert!(!knowledge_agent
            .tool_names()
            .contains(&"digital_pdf_document"));
        assert!(!knowledge_agent.tool_names().contains(&"shell_exec"));

        let security_agent = resolve_agent(Some("digital_security")).unwrap();
        assert!(security_agent.system_prompt.contains("监控安防管家"));
        assert_eq!(security_agent.tool_names(), DIGITAL_SECURITY_TOOLS);
        assert!(security_agent
            .tool_names()
            .contains(&"digital_security_event_query"));
        assert!(security_agent
            .tool_names()
            .contains(&"digital_security_camera_control"));
        assert!(!security_agent
            .tool_names()
            .contains(&"digital_media_control"));
        assert!(!security_agent.tool_names().contains(&"shell_exec"));

        let photo_agent = resolve_agent(Some("digital_photos")).unwrap();
        assert!(photo_agent.system_prompt.contains("智能相册专家"));
        assert_eq!(photo_agent.tool_names(), DIGITAL_PHOTO_TOOLS);
        assert!(photo_agent.tool_names().contains(&"digital_photo_album"));
        assert!(photo_agent.tool_names().contains(&"digital_photo_metadata"));
        assert!(!photo_agent.tool_names().contains(&"digital_file_manager"));
        assert!(!photo_agent.tool_names().contains(&"shell_exec"));

        let media_agent = resolve_agent(Some("digital_media")).unwrap();
        assert!(media_agent.system_prompt.contains("娱乐影音大咖"));
        assert_eq!(media_agent.tool_names(), DIGITAL_MEDIA_TOOLS);
        assert!(media_agent
            .tool_names()
            .contains(&"digital_video_genre_playback"));
        assert!(media_agent
            .tool_names()
            .contains(&"digital_music_language_playback"));
        assert!(media_agent
            .tool_names()
            .contains(&"digital_media_transport_control"));
        assert!(!media_agent
            .tool_names()
            .contains(&"digital_security_event_query"));
        assert!(!media_agent.tool_names().contains(&"shell_exec"));
    }

    #[test]
    fn legacy_digital_life_agent_keeps_existing_prompt_and_tools() {
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
        assert!(agent.tool_names().contains(&"digital_video_genre_playback"));
        assert!(agent
            .tool_names()
            .contains(&"digital_video_region_playback"));
        assert!(agent.tool_names().contains(&"digital_music_playback"));
        assert!(agent.tool_names().contains(&"digital_music_genre_playback"));
        assert!(agent
            .tool_names()
            .contains(&"digital_music_language_playback"));
        assert!(agent
            .tool_names()
            .contains(&"digital_media_transport_control"));
        assert_eq!(agent.tool_names().len(), 16);
        assert!(!agent.tool_names().contains(&"digital_media_control"));
        assert!(!agent.tool_names().contains(&"digital_invoice_extract"));
        assert!(!agent.tool_names().contains(&"digital_contract_extract"));
        assert!(!agent.tool_names().contains(&"digital_receipt_extract"));
        assert!(agent.tool_names().contains(&"digital_note_knowledge"));
        assert!(!agent.tool_names().contains(&"shell_exec"));
    }
}
