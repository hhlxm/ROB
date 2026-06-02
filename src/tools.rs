use anyhow::{anyhow, Context, Result};
use serde::Serialize;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::process::Command;
use tokio::time::{timeout, Duration};

const MAX_OUTPUT_BYTES: usize = 16 * 1024;
const TOOL_TITLE_FIELD: &str = "tool_title";
const HOME_FLOORS: &[&str] = &[
    "一楼",
    "二楼",
    "三楼",
    "四楼",
    "五楼",
    "阁楼",
    "地下室",
    "夹层",
    "屋顶花园",
    "楼顶",
    "顶楼",
    "负一楼",
    "地下一层",
];
const HOME_ROOMS: &[&str] = &[
    "客厅",
    "餐厅",
    "主卧",
    "次卧",
    "儿童房",
    "长辈房",
    "书房",
    "厨房",
    "卫生间",
    "阳台",
    "衣帽间",
    "玄关",
    "走廊",
    "储藏室",
    "车库",
    "健身房",
    "影音室",
    "娱乐室",
    "洗衣房",
    "茶室",
    "棋牌室",
    "瑜伽室",
    "宠物房",
    "停车场",
    "电竞房",
    "全屋",
];
const LIGHT_DEVICE_NAMES: &[&str] = &[
    "灯",
    "主灯",
    "吸顶灯",
    "筒灯",
    "射灯",
    "灯带",
    "彩光灯带",
    "彩光灯",
    "客厅灯",
    "卧室灯",
    "灯泡",
    "球泡灯",
    "色温灯",
    "调光灯",
    "床头灯",
    "台灯",
    "夜灯",
];
const CURTAIN_DEVICE_NAMES: &[&str] = &[
    "窗帘",
    "窗帘电机",
    "智能窗帘",
    "智能窗帘电机",
    "开窗器",
    "卷帘",
    "卷帘电机",
    "开合帘",
    "百叶窗",
    "客厅窗帘",
    "卧室窗帘",
    "窗帘伴侣",
    "梦幻帘",
];
const POWER_DEVICE_NAMES: &[&str] = &[
    "插座",
    "智能插座",
    "插头",
    "开关插座",
    "计量插座",
    "墙壁插座",
    "开关",
    "一路开关",
    "二路开关",
    "三路开关",
    "一键开关",
    "二键开关",
    "三键开关",
    "单开",
    "双开",
    "三开",
    "墙壁开关",
    "零火开关",
    "单火开关",
    "场景开关",
    "无线开关",
    "通断器",
];
const SCENE_NAMES: &[&str] = &[
    "回家模式",
    "离家模式",
    "睡眠模式",
    "起床模式",
    "电影模式",
    "用餐模式",
    "会客模式",
    "阅读模式",
    "浪漫模式",
    "派对模式",
    "节能模式",
    "安防模式",
    "通风模式",
    "度假模式",
    "宠物模式",
];

#[derive(Debug, Clone, Serialize)]
pub struct ToolSpec {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: ToolFunctionSpec,
}

#[derive(Debug, Clone, Serialize)]
pub struct ToolFunctionSpec {
    pub name: String,
    pub description: String,
    pub parameters: Value,
}

pub fn tool_specs() -> Vec<ToolSpec> {
    vec![
        tool(
            "pwd",
            "Return the current working directory.",
            json!({
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }),
        ),
        tool(
            "list_dir",
            "List files and directories under a path.",
            json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Directory path. Defaults to current directory." }
                },
                "additionalProperties": false
            }),
        ),
        tool(
            "read_file",
            "Read a UTF-8 text file with an output byte limit.",
            json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" },
                    "max_bytes": { "type": "integer", "minimum": 1, "maximum": 65536 }
                },
                "required": ["path"],
                "additionalProperties": false
            }),
        ),
        tool(
            "search_text",
            "Search for text with ripgrep when available.",
            json!({
                "type": "object",
                "properties": {
                    "pattern": { "type": "string" },
                    "path": { "type": "string", "description": "File or directory path. Defaults to current directory." },
                    "max_results": { "type": "integer", "minimum": 1, "maximum": 200 }
                },
                "required": ["pattern"],
                "additionalProperties": false
            }),
        ),
        tool(
            "shell_exec",
            "Run a small allowlisted Linux command without invoking a shell. Always provide both `command` and `args`; use `args: []` when the command has no arguments. For system configuration, prefer concrete commands such as uname, whoami, env, df, ps, or ls.",
            json!({
                "type": "object",
                "properties": {
                    "command": {
                        "type": "string",
                        "enum": ["pwd", "ls", "cat", "head", "tail", "wc", "rg", "find", "date", "uname", "whoami", "df", "du", "ps", "env"]
                    },
                    "args": { "type": "array", "items": { "type": "string" }, "default": [] },
                    "timeout_ms": { "type": "integer", "minimum": 100, "maximum": 10000 }
                },
                "required": ["command", "args"],
                "additionalProperties": false
            }),
        ),
        tool(
            "smart_home_control_speaker",
            "Submit a normalized smart-home command for speaker volume control. Use this for volume up/down, mute, or setting an exact volume.",
            json!({
                "type": "object",
                "properties": {
                    "floor": { "type": "string", "enum": HOME_FLOORS, "description": "楼层。用户原话出现楼层时必须填写；未说明时省略，不要编造。" },
                    "room": { "type": "string", "enum": HOME_ROOMS, "description": "房间。用户原话出现房间时必须填写；全屋控制时使用“全屋”；未说明时省略。" },
                    "device_name": { "type": "string", "description": "设备名称或别名，例如“扬声器”“音箱”“音响”“播放器”“NAS”。用户原话出现设备名或别名时必须原样填写；未说明时可省略。" },
                    "action": {
                        "type": "string",
                        "enum": ["increase_volume", "decrease_volume", "mute", "set_volume"],
                        "description": "音量调大、调小、静音或设置指定音量。"
                    },
                    "delta_percent": { "type": "integer", "minimum": 0, "maximum": 100, "description": "相对调节百分比。increase_volume/decrease_volume 必须填写；用户未给数值时默认填 10；百分之二十填 20。" },
                    "volume_percent": { "type": "integer", "minimum": 0, "maximum": 100, "description": "目标音量百分比。mute 必须显式填 0；set_volume 必须填写；一半填 50。" }
                },
                "required": ["action"],
                "additionalProperties": false
            }),
        ),
        tool(
            "smart_home_control_light",
            "Submit a normalized smart-home command for light power, brightness, and RGB color. For color temperature or warm/neutral/white/cool tone, prefer smart_home_control_light_temperature.",
            json!({
                "type": "object",
                "properties": {
                    "floor": { "type": "string", "enum": HOME_FLOORS, "description": "楼层。用户原话出现楼层时必须填写；未说明时省略，不要编造。" },
                    "room": { "type": "string", "enum": HOME_ROOMS, "description": "房间。用户原话出现房间时必须填写；全屋控制时使用“全屋”；未说明时省略。" },
                    "device_name": { "type": "string", "enum": LIGHT_DEVICE_NAMES, "description": "灯具名称，例如灯、主灯、吸顶灯、筒灯、灯带、床头灯、台灯、夜灯。用户原话出现灯具名称时必须填写，不要省略。" },
                    "action": {
                        "type": "string",
                        "enum": [
                            "turn_on",
                            "turn_off",
                            "increase_brightness",
                            "decrease_brightness",
                            "set_brightness",
                            "increase_color_temperature",
                            "decrease_color_temperature",
                            "set_color_temperature",
                            "set_light_tone",
                            "set_color"
                        ],
                        "description": "灯光开关、亮度、色温、光色或颜色控制动作。"
                    },
                    "delta_percent": { "type": "integer", "minimum": 0, "maximum": 100, "description": "亮度相对调节百分比。increase_brightness/decrease_brightness 必须填写；用户未给数值时默认填 20；百分之三十填 30。" },
                    "brightness_percent": { "type": "integer", "minimum": 0, "maximum": 100, "description": "目标亮度百分比。set_brightness 必须填写；一半填 50。" },
                    "delta_kelvin": { "type": "integer", "minimum": 0, "maximum": 6000, "description": "色温相对调节值，必须填写完整 K 数值，不要除以 10。普通调冷/调暖默认填 500；调高 1000K 填 1000；上调 1200K 填 1200。" },
                    "color_temperature_kelvin": { "type": "integer", "minimum": 1000, "maximum": 6000, "description": "目标色温 K，必须填写完整 K 数值，不要除以 10。3000K 填 3000，4000K 填 4000，6000K 填 6000；不要写 300/400/600。" },
                    "light_tone": {
                        "type": "string",
                        "enum": ["warm", "neutral", "natural", "white", "cool"],
                        "description": "用户要求调成暖光、中性光、自然光、白光或冷光时使用，并同时填写完整 color_temperature_kelvin：warm=3000，neutral/natural=4000，white/cool=6000。"
                    },
                    "color_name": {
                        "type": "string",
                        "enum": ["红色", "橙色", "黄色", "绿色", "青色", "蓝色", "紫色"],
                        "description": "目标颜色名称。"
                    },
                    "rgb": {
                        "type": "object",
                        "properties": {
                            "r": { "type": "integer", "minimum": 0, "maximum": 255 },
                            "g": { "type": "integer", "minimum": 0, "maximum": 255 },
                            "b": { "type": "integer", "minimum": 0, "maximum": 255 }
                        },
                        "required": ["r", "g", "b"],
                        "additionalProperties": false,
                        "description": "目标 RGB 颜色值。"
                    }
                },
                "required": ["action"],
                "additionalProperties": false
            }),
        ),
        tool(
            "smart_home_control_light_temperature",
            "Submit a normalized smart-home command for light color temperature and tone. Use this for 色温, 调冷, 调暖, 暖光, 中性光, 自然光, 白光, or 冷光. Kelvin values must be full integer K values: 1000K => 1000, 4000K => 4000, never 100 or 400.",
            json!({
                "type": "object",
                "properties": {
                    "floor": { "type": "string", "enum": HOME_FLOORS, "description": "楼层。用户原话出现楼层时必须填写；未说明时省略，不要编造。" },
                    "room": { "type": "string", "enum": HOME_ROOMS, "description": "房间。用户原话出现房间时必须填写；全屋控制时使用“全屋”；未说明时省略。" },
                    "device_name": { "type": "string", "enum": LIGHT_DEVICE_NAMES, "description": "灯具名称，例如灯、主灯、吸顶灯、筒灯、灯带、床头灯、台灯、夜灯。用户原话出现灯具名称时必须填写，不要省略。" },
                    "action": {
                        "type": "string",
                        "enum": [
                            "increase_color_temperature",
                            "decrease_color_temperature",
                            "set_color_temperature",
                            "set_light_tone"
                        ],
                        "description": "色温调冷/调白/调高/增加使用 increase_color_temperature；色温调暖/调黄/调低/降低使用 decrease_color_temperature；设置指定色温使用 set_color_temperature；调成暖光/中性光/自然光/白光/冷光使用 set_light_tone。"
                    },
                    "delta_kelvin": {
                        "type": "integer",
                        "enum": [500, 600, 800, 1000, 1200],
                        "description": "色温相对调节值。increase_color_temperature/decrease_color_temperature 必须填写；用户未给数值时默认填 500；调高 1000K 填 1000；上调 1200K 填 1200。必须填完整 K 数值，不要写 100/120。"
                    },
                    "color_temperature_kelvin": {
                        "type": "integer",
                        "enum": [3000, 3500, 4000, 4500, 5000, 6000],
                        "description": "目标色温 K。3000K 填 3000，3500K 填 3500，4000K 填 4000，4500K 填 4500，5000K 填 5000，6000K 填 6000。不要写 300/350/400/450/500/600。"
                    },
                    "light_tone": {
                        "type": "string",
                        "enum": ["warm", "neutral", "natural", "white", "cool"],
                        "description": "暖光=warm 且 color_temperature_kelvin=3000；中性光=neutral 且 4000；自然光=natural 且 4000；白光=white 且 6000；冷光=cool 且 6000。"
                    }
                },
                "required": ["action"],
                "additionalProperties": false
            }),
        ),
        tool(
            "smart_home_control_curtain",
            "Submit a normalized smart-home command for curtains, blinds, rollers, and openers.",
            json!({
                "type": "object",
                "properties": {
                    "floor": { "type": "string", "enum": HOME_FLOORS, "description": "楼层。用户原话出现楼层时必须填写；未说明时省略，不要编造。" },
                    "room": { "type": "string", "enum": HOME_ROOMS, "description": "房间。用户原话出现房间时必须填写；全屋控制时使用“全屋”；未说明时省略。" },
                    "device_name": { "type": "string", "enum": CURTAIN_DEVICE_NAMES, "description": "窗帘设备名称，例如窗帘、卷帘、百叶窗、梦幻帘。用户原话出现窗帘设备名时必须填写，不要省略。" },
                    "action": {
                        "type": "string",
                        "enum": ["open", "close", "stop", "set_position", "set_angle"],
                        "description": "打开/拉开/展开使用 open；关闭/拉上/合上/收起使用 close；停止/暂停/停住使用 stop；开到百分比使用 set_position；百叶窗叶片角度使用 set_angle。"
                    },
                    "position_percent": { "type": "integer", "minimum": 0, "maximum": 100, "description": "窗帘开合度百分比。set_position 必须填写；一半填 50；百分之七十填 70。" },
                    "angle_degree": { "type": "integer", "minimum": 0, "maximum": 180, "description": "百叶窗角度。set_angle 必须填写；30度填 30，90度填 90。" }
                },
                "required": ["action"],
                "additionalProperties": false
            }),
        ),
        tool(
            "smart_home_control_power",
            "Submit a normalized smart-home command for outlets and wall switches.",
            json!({
                "type": "object",
                "properties": {
                    "floor": { "type": "string", "enum": HOME_FLOORS, "description": "楼层。用户原话出现楼层时必须填写；未说明时省略，不要编造。" },
                    "room": { "type": "string", "enum": HOME_ROOMS, "description": "房间。用户原话出现房间时必须填写；全屋控制时使用“全屋”；未说明时省略。" },
                    "device_category": {
                        "type": "string",
                        "enum": ["outlet", "wall_switch"],
                        "description": "插座、智能插座、计量插座、智能插头、插头、墙壁插座都必须使用 outlet；一路开关、二路开关、三路开关、单开、双开、三开、墙壁开关、无线开关、通断器都必须使用 wall_switch。"
                    },
                    "device_name": {
                        "type": "string",
                        "enum": POWER_DEVICE_NAMES,
                        "description": "设备名称，例如智能插座、计量插座、智能插头、墙壁插座、一路开关、双开、墙壁开关。用户原话出现电源设备名时必须填写，不要省略。"
                    },
                    "action": {
                        "type": "string",
                        "enum": ["turn_on", "turn_off"],
                        "description": "打开、开开、接通、通电、按亮、弄亮使用 turn_on；关闭、关掉、断开、断电、按灭、弄灭使用 turn_off。不要使用 stop 表示关闭。"
                    }
                },
                "required": ["device_category", "action"],
                "additionalProperties": false
            }),
        ),
        tool(
            "smart_home_control_scene",
            "Submit a normalized smart-home command to activate or deactivate a predefined home scene mode.",
            json!({
                "type": "object",
                "properties": {
                    "scene_name": {
                        "type": "string",
                        "enum": SCENE_NAMES,
                        "description": "场景模式名称。"
                    },
                    "action": {
                        "type": "string",
                        "enum": ["activate", "deactivate"],
                        "description": "打开/设置/执行/启动场景使用 activate；关闭/退出/停止场景使用 deactivate。"
                    }
                },
                "required": ["scene_name", "action"],
                "additionalProperties": false
            }),
        ),
        tool(
            "digital_file_manager",
            "Inspect a known file path or directly list/count one directory. Use for file attributes, file size, directory contents, and immediate file counts.",
            json!({
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "enum": ["get_properties", "list_directory", "count_directory"],
                        "description": "查看文件/目录属性用 get_properties；查看目录里有什么用 list_directory；统计桌面/下载目录有几个文件用 count_directory。"
                    },
                    "path": {
                        "type": "string",
                        "description": "用户给出的文件或目录路径。可以是绝对路径、相对路径或 ~/ 开头路径；必须保留用户原文目标。"
                    },
                    "include_hidden": {
                        "type": "boolean",
                        "description": "是否包含隐藏文件。用户未说明时默认 false。"
                    },
                    "max_entries": {
                        "type": "integer",
                        "minimum": 1,
                        "maximum": 200,
                        "description": "list_directory 最多返回多少项，默认 50。"
                    }
                },
                "required": ["action", "path"],
                "additionalProperties": false
            }),
        ),
        tool(
            "digital_photo_library",
            "Submit a normalized photo-library query. Use for album discovery, shared albums, album name search, finding which album contains a natural-language photo target, and single-photo capture metadata.",
            json!({
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "enum": ["list_albums", "list_shared_albums", "search_album", "find_photo_album", "get_photo_metadata"],
                        "description": "我有哪些相册=list_albums；分享出去的相册=list_shared_albums；查宝宝相册=search_album；猫的照片在哪个相册=find_photo_album；这张照片什么时候/在哪拍=get_photo_metadata。"
                    },
                    "album_query": {
                        "type": "string",
                        "description": "相册名或相册关键词，例如“宝宝相册”。用户原话出现时必须原样填写。"
                    },
                    "photo_query": {
                        "type": "string",
                        "description": "照片自然语言目标，例如“猫的照片”。"
                    },
                    "photo_id": {
                        "type": "string",
                        "description": "上下文中已有的照片 ID。"
                    },
                    "photo_path": {
                        "type": "string",
                        "description": "单张照片文件路径。"
                    },
                    "metadata_fields": {
                        "type": "array",
                        "items": {
                            "type": "string",
                            "enum": ["captured_at", "location", "camera", "exif", "album"]
                        },
                        "description": "需要查看的拍摄信息字段；未说明时可填 captured_at、location、exif。"
                    }
                },
                "required": ["action"],
                "additionalProperties": false
            }),
        ),
        tool(
            "digital_media_control",
            "Submit a normalized media playback command. Use for playing a title, pause/resume, episode navigation, switching audio/subtitle, casting, and progress lookup.",
            json!({
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "enum": ["play_title", "resume_last", "pause", "resume", "next_episode", "previous_episode", "jump_episode", "switch_audio", "switch_subtitle", "cast_to_device", "get_progress"],
                        "description": "播放片名=play_title；继续看上次那部=resume_last；暂停=pause；继续播放=resume；下一集/上一集/跳到第几集；切换音轨/字幕；投屏；看到第几集。"
                    },
                    "title": {
                        "type": "string",
                        "description": "影片、剧集、节目名称，例如“狂飙”。用户原话出现时必须原样填写。"
                    },
                    "episode_number": {
                        "type": "integer",
                        "minimum": 1,
                        "description": "目标集数。跳到第 5 集填 5。"
                    },
                    "audio_track": {
                        "type": "string",
                        "description": "目标音轨，如国语、粤语、英文；用户未指定可省略。"
                    },
                    "subtitle": {
                        "type": "string",
                        "description": "目标字幕，如中文字幕、英文字幕、关闭字幕；用户未指定可省略。"
                    },
                    "target_device": {
                        "type": "string",
                        "description": "投屏目标设备，例如“客厅电视”。"
                    }
                },
                "required": ["action"],
                "additionalProperties": false
            }),
        ),
        tool(
            "digital_security_monitor",
            "Submit a normalized camera/security query. Use for recent events, motion/person/vehicle/package checks, and identity recognition against known face labels.",
            json!({
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "enum": ["query_events", "identify_person"],
                        "description": "区域是否有人/有动静/有车/有快递用 query_events；爸爸/妈妈/张三/陌生人/熟人/门口那个是谁用 identify_person。"
                    },
                    "area": {
                        "type": "string",
                        "description": "用户提到的区域或房间，例如“门口”“院子”“客厅”“宝宝房间”“阳台”。"
                    },
                    "camera_name": {
                        "type": "string",
                        "description": "明确的摄像头名称；没有摄像头名但有区域时填 area。"
                    },
                    "time_query": {
                        "type": "string",
                        "description": "时间窗口原文，例如“刚刚”“刚才”“今天”“下午”“今晚”。必须保留原文。"
                    },
                    "event_type": {
                        "type": "string",
                        "enum": ["person", "motion", "vehicle", "package", "entry", "unknown"],
                        "description": "有人=person；动静=motion；车进出=vehicle；快递=package；进门/回来=entry；不明确填 unknown。"
                    },
                    "person_label": {
                        "type": "string",
                        "description": "已知人脸库标签或人物称谓，例如“爸爸”“妈妈”“张三”。"
                    },
                    "identity_query": {
                        "type": "string",
                        "enum": ["known_person", "stranger", "familiar", "courier", "current_subject"],
                        "description": "熟人/已知人=known_person 或 familiar；陌生人=stranger；送快递的=courier；门口那个是谁=current_subject。"
                    }
                },
                "required": ["action"],
                "additionalProperties": false
            }),
        ),
        tool(
            "digital_document_workspace",
            "Submit a normalized document operation. Use for PDF, Word, PPT, spreadsheet, OCR, structured extraction, and file-level document metadata queries. Current implementation is a mock payload for backend integration.",
            json!({
                "type": "object",
                "properties": {
                    "document_type": {
                        "type": "string",
                        "enum": ["pdf", "word", "ppt", "spreadsheet", "image", "text", "unknown"],
                        "description": "PDF=pdf；docx/Word=word；PPT=ppt；xlsx/csv/表格=spreadsheet；图片/OCR=image。"
                    },
                    "action": {
                        "type": "string",
                        "enum": [
                            "pdf_encrypt",
                            "pdf_extract_pages",
                            "pdf_merge",
                            "pdf_rotate",
                            "pdf_watermark",
                            "pdf_extract_form",
                            "word_create",
                            "word_replace_text",
                            "word_extract_comments",
                            "word_add_toc",
                            "ppt_create_from_outline",
                            "spreadsheet_add_formula_column",
                            "spreadsheet_csv_to_xlsx",
                            "spreadsheet_filter_rows",
                            "spreadsheet_create",
                            "document_get_metadata",
                            "ocr_extract_text",
                            "structured_extract_fields"
                        ],
                        "description": "按用户请求选择单步文档动作。"
                    },
                    "input_paths": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "输入文件路径列表。单文件单动作通常只填一个；PDF 合并填多个。"
                    },
                    "output_path": {
                        "type": "string",
                        "description": "输出文件路径；用户未指定可省略，由后端决定。"
                    },
                    "password": {
                        "type": "string",
                        "description": "PDF 加密密码。"
                    },
                    "page_range": {
                        "type": "string",
                        "description": "页码范围，例如“3-10”。"
                    },
                    "rotation": {
                        "type": "string",
                        "enum": ["left", "right", "180", "portrait", "landscape"],
                        "description": "PDF 旋转或横竖屏方向。"
                    },
                    "watermark_text": {
                        "type": "string",
                        "description": "水印文字。"
                    },
                    "outline": {
                        "type": "string",
                        "description": "PPT/Word 创建的大纲或要点。"
                    },
                    "slide_count": {
                        "type": "integer",
                        "minimum": 1,
                        "maximum": 50,
                        "description": "PPT 页数，例如 5 页填 5。"
                    },
                    "title": {
                        "type": "string",
                        "description": "新建文档或 PPT 标题。"
                    },
                    "find_text": {
                        "type": "string",
                        "description": "Word 替换的原文字，例如“甲方”。"
                    },
                    "replace_text": {
                        "type": "string",
                        "description": "Word 替换的新文字，例如“乙方”。"
                    },
                    "formula": {
                        "type": "string",
                        "description": "表格公式或求和公式描述。"
                    },
                    "filter_condition": {
                        "type": "string",
                        "description": "表格筛选条件，例如“金额大于 1000”。"
                    },
                    "fields": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "要提取的字段，例如金额、日期、甲方、乙方、商户。"
                    },
                    "question": {
                        "type": "string",
                        "description": "文档元信息问题，例如“多少页”“标题是什么”“作者是谁”“出版日期”。"
                    }
                },
                "required": ["document_type", "action"],
                "additionalProperties": false
            }),
        ),
        tool(
            "digital_text_assistant",
            "Submit a short-text language task. Use for summarization, translation, polishing, style rewrite, expansion, and compression of short text.",
            json!({
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "enum": ["summarize", "translate", "rewrite", "expand", "compress"],
                        "description": "总结=summarize；翻译=translate；润色/正式一点/简洁点=rewrite；扩写=expand；压缩到指定字数=compress。"
                    },
                    "text": {
                        "type": "string",
                        "description": "待处理的短文本、句子、段落或标题。"
                    },
                    "source_language": {
                        "type": "string",
                        "description": "源语言；用户未说明可省略。"
                    },
                    "target_language": {
                        "type": "string",
                        "description": "目标语言，例如“中文”“英文”。"
                    },
                    "style": {
                        "type": "string",
                        "description": "改写风格，例如“正式”“简洁”“口语化”。"
                    },
                    "max_chars": {
                        "type": "integer",
                        "minimum": 1,
                        "maximum": 2000,
                        "description": "压缩目标字数，例如 200 字填 200。"
                    }
                },
                "required": ["action", "text"],
                "additionalProperties": false
            }),
        ),
        tool(
            "digital_note_knowledge",
            "Submit a normalized note/knowledge-base operation. Use for tagging notes, linking notes, creating topics, keyword search, and simple note QA.",
            json!({
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "enum": ["tag_note", "link_notes", "create_topic", "search_notes", "question_notes"],
                        "description": "笔记标主题=tag_note；关联两条笔记=link_notes；新建主题=create_topic；找关键词=search_notes；有没有写过/问答=question_notes。"
                    },
                    "note_id": {
                        "type": "string",
                        "description": "单条笔记 ID。"
                    },
                    "note_ids": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "需要关联的笔记 ID 列表。"
                    },
                    "topic": {
                        "type": "string",
                        "description": "主题名，例如“K8s”。用户原话出现时必须原样填写。"
                    },
                    "query": {
                        "type": "string",
                        "description": "关键词或问题，例如“RAG”“OAuth”“我有没有写过关于 K8s 的笔记”。"
                    }
                },
                "required": ["action"],
                "additionalProperties": false
            }),
        ),
    ]
}

pub fn tool_specs_by_name(names: &[&str]) -> Result<Vec<ToolSpec>> {
    let all = tool_specs();
    names
        .iter()
        .map(|name| {
            all.iter()
                .find(|spec| spec.function.name == *name)
                .cloned()
                .ok_or_else(|| anyhow!("unknown tool `{name}` in agent definition"))
        })
        .collect()
}

pub async fn run_tool(name: &str, args: Value) -> Result<String> {
    match name {
        "pwd" => Ok(std::env::current_dir()?.display().to_string()),
        "list_dir" => list_dir(args),
        "read_file" => read_file(args),
        "search_text" => search_text(args).await,
        "shell_exec" => shell_exec(args).await,
        "smart_home_control_speaker"
        | "smart_home_control_light_temperature"
        | "smart_home_control_light"
        | "smart_home_control_curtain"
        | "smart_home_control_power"
        | "smart_home_control_scene" => smart_home_command(name, args),
        "digital_file_manager" => digital_file_manager(args),
        "digital_photo_library"
        | "digital_media_control"
        | "digital_security_monitor"
        | "digital_document_workspace"
        | "digital_text_assistant"
        | "digital_note_knowledge" => digital_life_mock_command(name, args),
        _ => Err(anyhow!("unknown tool `{name}`")),
    }
}

pub fn tool_context_prompt(specs: &[ToolSpec]) -> String {
    let mut prompt = String::from(
        "Tool usage context for this turn:\n\
- Use only tools provided in the current `tools` request field.\n\
- Tool arguments must strictly match each tool JSON schema; include every required field.\n\
- Tool arguments must be one complete valid JSON object: close every `{` and `[`, quote every key and string, and do not output partial JSON.\n\
- Every tool call must include `tool_title`, a short 4-12 word title in the same language as the user.\n\
- If the user mentions a floor, room, or device name/alias, copy it into the matching tool argument; do not omit or generalize it.\n\
- Explicitly fill default numeric control values: speaker volume delta_percent=10, light brightness delta_percent=20, light color-temperature delta_kelvin=500, mute volume_percent=0.\n\
- For power tools, outlets/plugs/wall outlets use device_category=outlet; wall switches and single/double/triple switches use device_category=wall_switch.\n\
- Prefer specialized tools before `shell_exec`: use `pwd`, `list_dir`, `read_file`, or `search_text` when they fit.\n\
- Use at most one tool call per model round unless calls are independent.\n\
- After a tool result, inspect it before deciding whether another tool is needed.\n\
- For `shell_exec`, always provide both `command` and `args`; use `args: []` when there are no arguments. Example: {\"command\":\"uname\",\"args\":[\"-a\"],\"timeout_ms\":3000}.\n\n\
Available tools:\n",
    );

    for spec in specs {
        prompt.push_str("- `");
        prompt.push_str(&spec.function.name);
        prompt.push_str("`: ");
        prompt.push_str(&spec.function.description);
        let required = required_fields(&spec.function.parameters);
        if !required.is_empty() {
            prompt.push_str(" Required: ");
            prompt.push_str(&required.join(", "));
            prompt.push('.');
        }
        prompt.push('\n');
    }

    prompt
}

fn tool(name: &str, description: &str, parameters: Value) -> ToolSpec {
    ToolSpec {
        tool_type: "function".to_string(),
        function: ToolFunctionSpec {
            name: name.to_string(),
            description: description.to_string(),
            parameters: decorate_parameter_schema(parameters),
        },
    }
}

fn required_fields(parameters: &Value) -> Vec<String> {
    parameters
        .get("required")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(ToString::to_string)
        .collect()
}

fn decorate_parameter_schema(mut parameters: Value) -> Value {
    let Some(object) = parameters.as_object_mut() else {
        return parameters;
    };

    let properties = object.entry("properties").or_insert_with(|| json!({}));
    if let Some(properties) = properties.as_object_mut() {
        properties.insert(
            TOOL_TITLE_FIELD.to_string(),
            json!({
                "type": "string",
                "description": "A short 4-12 word title for this tool call, in the same language as the user."
            }),
        );
    }

    let required = object
        .entry("required")
        .or_insert_with(|| Value::Array(Vec::new()));
    if let Some(required) = required.as_array_mut() {
        let has_tool_title = required.iter().any(|value| value == TOOL_TITLE_FIELD);
        if !has_tool_title {
            required.insert(0, Value::String(TOOL_TITLE_FIELD.to_string()));
        }
    }

    parameters
}

fn list_dir(args: Value) -> Result<String> {
    let path = string_arg(&args, "path").unwrap_or_else(|| ".".to_string());
    let mut entries = Vec::new();
    for entry in std::fs::read_dir(&path).with_context(|| format!("failed to read dir {path}"))? {
        let entry = entry?;
        let metadata = entry.metadata()?;
        let kind = if metadata.is_dir() { "dir" } else { "file" };
        entries.push(format!("{kind}\t{}", entry.file_name().to_string_lossy()));
    }
    entries.sort();
    Ok(truncate(entries.join("\n"), MAX_OUTPUT_BYTES))
}

fn read_file(args: Value) -> Result<String> {
    let path = required_string_arg(&args, "path")?;
    let max_bytes = usize_arg(&args, "max_bytes").unwrap_or(MAX_OUTPUT_BYTES);
    let raw =
        std::fs::read_to_string(&path).with_context(|| format!("failed to read file {path}"))?;
    Ok(truncate(raw, max_bytes))
}

async fn search_text(args: Value) -> Result<String> {
    let pattern = required_string_arg(&args, "pattern")?;
    let path = string_arg(&args, "path").unwrap_or_else(|| ".".to_string());
    let max_results = usize_arg(&args, "max_results").unwrap_or(50).min(200);

    if command_exists("rg").await {
        let output = Command::new("rg")
            .arg("--line-number")
            .arg("--color=never")
            .arg("--max-count")
            .arg(max_results.to_string())
            .arg(&pattern)
            .arg(&path)
            .output()
            .await
            .context("failed to run rg")?;
        return command_output(output);
    }

    let mut matches = Vec::new();
    search_text_fallback(Path::new(&path), &pattern, max_results, &mut matches)?;
    Ok(truncate(matches.join("\n"), MAX_OUTPUT_BYTES))
}

async fn shell_exec(args: Value) -> Result<String> {
    let command = required_string_arg(&args, "command")?;
    let argv = string_array_arg(&args, "args")?;
    let timeout_ms = usize_arg(&args, "timeout_ms")
        .unwrap_or(3000)
        .clamp(100, 10000);

    let allowed = [
        "pwd", "ls", "cat", "head", "tail", "wc", "rg", "find", "date", "uname", "whoami", "df",
        "du", "ps", "env",
    ];
    if !allowed.contains(&command.as_str()) {
        return Err(anyhow!("command `{command}` is not in the allowlist"));
    }

    let child = Command::new(&command).args(argv).output();
    let output = timeout(Duration::from_millis(timeout_ms as u64), child)
        .await
        .context("command timed out")?
        .with_context(|| format!("failed to run {command}"))?;
    command_output(output)
}

fn smart_home_command(name: &str, mut args: Value) -> Result<String> {
    normalize_smart_home_args(name, &mut args);
    Ok(serde_json::to_string_pretty(&json!({
        "status": "accepted",
        "tool": name,
        "command": args,
        "note": "normalized smart-home command payload; map this to the real home gateway integration"
    }))?)
}

fn digital_file_manager(args: Value) -> Result<String> {
    let action = required_string_arg(&args, "action")?;
    let raw_path = required_string_arg(&args, "path")?;
    let path = expand_user_path(&raw_path);

    match action.as_str() {
        "get_properties" => file_properties(&raw_path, &path),
        "list_directory" => list_directory_details(&args, &raw_path, &path),
        "count_directory" => count_directory_entries(&args, &raw_path, &path),
        _ => Err(anyhow!(
            "unsupported digital_file_manager action `{action}`"
        )),
    }
}

fn file_properties(raw_path: &str, path: &Path) -> Result<String> {
    let metadata = std::fs::metadata(path)
        .with_context(|| format!("failed to read metadata for {}", path.display()))?;
    let kind = file_kind(&metadata);

    Ok(serde_json::to_string_pretty(&json!({
        "status": "ok",
        "tool": "digital_file_manager",
        "action": "get_properties",
        "path": raw_path,
        "resolved_path": path.display().to_string(),
        "properties": {
            "kind": kind,
            "size_bytes": metadata.len(),
            "readonly": metadata.permissions().readonly(),
            "modified_epoch_seconds": epoch_seconds(metadata.modified().ok()),
            "accessed_epoch_seconds": epoch_seconds(metadata.accessed().ok()),
            "created_epoch_seconds": epoch_seconds(metadata.created().ok())
        }
    }))?)
}

fn list_directory_details(args: &Value, raw_path: &str, path: &Path) -> Result<String> {
    let include_hidden = bool_arg(args, "include_hidden").unwrap_or(false);
    let max_entries = usize_arg(args, "max_entries").unwrap_or(50).clamp(1, 200);
    let mut entries = Vec::new();

    for entry in std::fs::read_dir(path)
        .with_context(|| format!("failed to read directory {}", path.display()))?
    {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        if !include_hidden && name.starts_with('.') {
            continue;
        }
        let metadata = entry.metadata()?;
        entries.push(json!({
            "name": name,
            "kind": file_kind(&metadata),
            "size_bytes": metadata.len(),
            "readonly": metadata.permissions().readonly()
        }));
        if entries.len() >= max_entries {
            break;
        }
    }

    entries.sort_by(|left, right| {
        left["name"]
            .as_str()
            .unwrap_or_default()
            .cmp(right["name"].as_str().unwrap_or_default())
    });

    Ok(serde_json::to_string_pretty(&json!({
        "status": "ok",
        "tool": "digital_file_manager",
        "action": "list_directory",
        "path": raw_path,
        "resolved_path": path.display().to_string(),
        "include_hidden": include_hidden,
        "entries": entries,
        "returned_count": entries.len()
    }))?)
}

fn count_directory_entries(args: &Value, raw_path: &str, path: &Path) -> Result<String> {
    let include_hidden = bool_arg(args, "include_hidden").unwrap_or(false);
    let mut files = 0usize;
    let mut dirs = 0usize;
    let mut others = 0usize;

    for entry in std::fs::read_dir(path)
        .with_context(|| format!("failed to read directory {}", path.display()))?
    {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        if !include_hidden && name.starts_with('.') {
            continue;
        }
        let metadata = entry.metadata()?;
        if metadata.is_file() {
            files += 1;
        } else if metadata.is_dir() {
            dirs += 1;
        } else {
            others += 1;
        }
    }

    Ok(serde_json::to_string_pretty(&json!({
        "status": "ok",
        "tool": "digital_file_manager",
        "action": "count_directory",
        "path": raw_path,
        "resolved_path": path.display().to_string(),
        "include_hidden": include_hidden,
        "counts": {
            "files": files,
            "directories": dirs,
            "others": others,
            "total": files + dirs + others
        }
    }))?)
}

fn digital_life_mock_command(name: &str, args: Value) -> Result<String> {
    Ok(serde_json::to_string_pretty(&json!({
        "status": "accepted",
        "tool": name,
        "command": args,
        "note": "normalized digital-life payload; map this to the real album, media, security, document, OCR, note, or text backend integration"
    }))?)
}

fn expand_user_path(path: &str) -> PathBuf {
    if path == "~" {
        if let Some(home) = dirs::home_dir() {
            return home;
        }
    }

    if let Some(rest) = path.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(rest);
        }
    }

    PathBuf::from(path)
}

fn file_kind(metadata: &std::fs::Metadata) -> &'static str {
    if metadata.is_dir() {
        "directory"
    } else if metadata.is_file() {
        "file"
    } else {
        "other"
    }
}

fn epoch_seconds(time: Option<SystemTime>) -> Option<u64> {
    time.and_then(|value| value.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_secs())
}

fn normalize_smart_home_args(name: &str, args: &mut Value) {
    let Some(object) = args.as_object_mut() else {
        return;
    };

    if name == "smart_home_control_speaker" {
        normalize_speaker_defaults(object);
        return;
    }

    if name == "smart_home_control_light" {
        normalize_light_defaults(object);
    }

    if !matches!(
        name,
        "smart_home_control_light" | "smart_home_control_light_temperature"
    ) {
        return;
    }

    match object.get("action").and_then(Value::as_str) {
        Some("set_light_tone") => normalize_light_tone_kelvin(object),
        Some("set_color_temperature") => scale_kelvin_field(object, "color_temperature_kelvin"),
        Some("increase_color_temperature") | Some("decrease_color_temperature") => {
            scale_common_delta_kelvin(object)
        }
        _ => {}
    }
}

fn normalize_speaker_defaults(object: &mut serde_json::Map<String, Value>) {
    match object.get("action").and_then(Value::as_str) {
        Some("increase_volume") | Some("decrease_volume") => {
            insert_default_number(object, "delta_percent", 10);
        }
        Some("mute") => {
            insert_default_number(object, "volume_percent", 0);
        }
        _ => {}
    }
}

fn normalize_light_defaults(object: &mut serde_json::Map<String, Value>) {
    match object.get("action").and_then(Value::as_str) {
        Some("increase_brightness") | Some("decrease_brightness") => {
            insert_default_number(object, "delta_percent", 20);
        }
        Some("increase_color_temperature") | Some("decrease_color_temperature") => {
            insert_default_number(object, "delta_kelvin", 500);
        }
        _ => {}
    }
}

fn normalize_light_tone_kelvin(object: &mut serde_json::Map<String, Value>) {
    let kelvin = match object.get("light_tone").and_then(Value::as_str) {
        Some("warm") => Some(3000),
        Some("neutral") | Some("natural") => Some(4000),
        Some("white") | Some("cool") => Some(6000),
        _ => None,
    };

    if let Some(kelvin) = kelvin {
        object.insert(
            "color_temperature_kelvin".to_string(),
            Value::Number(kelvin.into()),
        );
    } else {
        scale_kelvin_field(object, "color_temperature_kelvin");
    }
}

fn scale_kelvin_field(object: &mut serde_json::Map<String, Value>, field: &str) {
    let Some(value) = object.get(field).and_then(Value::as_i64) else {
        return;
    };
    if (100..=600).contains(&value) {
        object.insert(field.to_string(), Value::Number((value * 10).into()));
    }
}

fn scale_common_delta_kelvin(object: &mut serde_json::Map<String, Value>) {
    insert_default_number(object, "delta_kelvin", 500);
    let Some(value) = object.get("delta_kelvin").and_then(Value::as_i64) else {
        return;
    };
    if matches!(value, 100 | 120) {
        object.insert(
            "delta_kelvin".to_string(),
            Value::Number((value * 10).into()),
        );
    }
}

fn insert_default_number(
    object: &mut serde_json::Map<String, Value>,
    field: &str,
    default_value: i64,
) {
    if object.get(field).map_or(true, Value::is_null) {
        object.insert(field.to_string(), Value::Number(default_value.into()));
    }
}

fn command_output(output: std::process::Output) -> Result<String> {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let status = output.status.code().unwrap_or(-1);
    Ok(truncate(
        format!("exit_code: {status}\nstdout:\n{stdout}\nstderr:\n{stderr}"),
        MAX_OUTPUT_BYTES,
    ))
}

async fn command_exists(command: &str) -> bool {
    Command::new(command)
        .arg("--version")
        .output()
        .await
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn search_text_fallback(
    path: &Path,
    pattern: &str,
    max_results: usize,
    matches: &mut Vec<String>,
) -> Result<()> {
    if matches.len() >= max_results {
        return Ok(());
    }
    if path.is_file() {
        search_file(path, pattern, max_results, matches)?;
        return Ok(());
    }
    if !path.is_dir() {
        return Ok(());
    }

    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let child = entry.path();
        if should_skip(&child) {
            continue;
        }
        if child.is_dir() {
            search_text_fallback(&child, pattern, max_results, matches)?;
        } else {
            search_file(&child, pattern, max_results, matches)?;
        }
        if matches.len() >= max_results {
            break;
        }
    }
    Ok(())
}

fn search_file(
    path: &Path,
    pattern: &str,
    max_results: usize,
    matches: &mut Vec<String>,
) -> Result<()> {
    let Ok(content) = std::fs::read_to_string(path) else {
        return Ok(());
    };
    for (index, line) in content.lines().enumerate() {
        if line.contains(pattern) {
            matches.push(format!("{}:{}:{}", path.display(), index + 1, line));
            if matches.len() >= max_results {
                break;
            }
        }
    }
    Ok(())
}

fn should_skip(path: &PathBuf) -> bool {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    matches!(name, ".git" | "target" | "node_modules" | ".gradle")
}

fn string_arg(args: &Value, key: &str) -> Option<String> {
    args.get(key)?.as_str().map(ToString::to_string)
}

fn required_string_arg(args: &Value, key: &str) -> Result<String> {
    string_arg(args, key).ok_or_else(|| anyhow!("missing string argument `{key}`"))
}

fn usize_arg(args: &Value, key: &str) -> Option<usize> {
    args.get(key)?.as_u64().map(|value| value as usize)
}

fn bool_arg(args: &Value, key: &str) -> Option<bool> {
    args.get(key)?.as_bool()
}

fn string_array_arg(args: &Value, key: &str) -> Result<Vec<String>> {
    let Some(value) = args.get(key) else {
        return Ok(Vec::new());
    };
    let array = value
        .as_array()
        .ok_or_else(|| anyhow!("argument `{key}` must be an array"))?;
    array
        .iter()
        .map(|item| {
            item.as_str()
                .map(ToString::to_string)
                .ok_or_else(|| anyhow!("argument `{key}` items must be strings"))
        })
        .collect()
}

fn truncate(mut value: String, max_bytes: usize) -> String {
    if value.len() <= max_bytes {
        return value;
    }
    value.truncate(max_bytes);
    value.push_str("\n[truncated]");
    value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn shell_exec_rejects_non_allowlisted_command() {
        let result = run_tool(
            "shell_exec",
            json!({
                "command": "sh",
                "args": ["-c", "echo no"]
            }),
        )
        .await;

        assert!(result.is_err());
    }

    #[test]
    fn shell_exec_schema_requires_command_and_args() {
        let specs = tool_specs();
        let shell = specs
            .iter()
            .find(|spec| spec.function.name == "shell_exec")
            .unwrap();
        let required = shell.function.parameters["required"].as_array().unwrap();

        assert!(required.iter().any(|value| value == "tool_title"));
        assert!(required.iter().any(|value| value == "command"));
        assert!(required.iter().any(|value| value == "args"));
    }

    #[test]
    fn smart_home_light_schema_exposes_locations_and_actions() {
        let specs = tool_specs();
        let light = specs
            .iter()
            .find(|spec| spec.function.name == "smart_home_control_light")
            .unwrap();

        assert!(light.function.parameters["properties"]["floor"]["enum"]
            .as_array()
            .unwrap()
            .iter()
            .any(|value| value == "一楼"));
        assert!(light.function.parameters["properties"]["room"]["enum"]
            .as_array()
            .unwrap()
            .iter()
            .any(|value| value == "全屋"));
        assert!(light.function.parameters["properties"]["action"]["enum"]
            .as_array()
            .unwrap()
            .iter()
            .any(|value| value == "set_color"));
        assert!(
            light.function.parameters["properties"]["color_temperature_kelvin"]["description"]
                .as_str()
                .unwrap()
                .contains("不要写 300/400/600")
        );
    }

    #[test]
    fn smart_home_temperature_schema_uses_full_kelvin_enums() {
        let specs = tool_specs();
        let temperature = specs
            .iter()
            .find(|spec| spec.function.name == "smart_home_control_light_temperature")
            .unwrap();

        assert!(temperature
            .function
            .description
            .contains("never 100 or 400"));
        assert!(
            temperature.function.parameters["properties"]["delta_kelvin"]["enum"]
                .as_array()
                .unwrap()
                .iter()
                .any(|value| value == 1200)
        );
        assert!(
            temperature.function.parameters["properties"]["color_temperature_kelvin"]["enum"]
                .as_array()
                .unwrap()
                .iter()
                .any(|value| value == 4000)
        );
    }

    #[test]
    fn tool_context_prompt_includes_schema_guidance() {
        let prompt = tool_context_prompt(&tool_specs());

        assert!(prompt.contains("Tool arguments must strictly match"));
        assert!(prompt.contains("tool_title"));
        assert!(prompt.contains("`shell_exec`"));
        assert!(prompt.contains("Required: tool_title, command, args"));
        assert!(prompt.contains(r#""command":"uname""#));
    }

    #[tokio::test]
    async fn pwd_tool_returns_current_directory() {
        let result = run_tool("pwd", json!({})).await.unwrap();

        assert!(result.contains("ROB"));
    }

    #[tokio::test]
    async fn smart_home_tool_returns_normalized_payload() {
        let result = run_tool(
            "smart_home_control_scene",
            json!({
                "tool_title": "打开回家模式",
                "scene_name": "回家模式",
                "action": "activate"
            }),
        )
        .await
        .unwrap();

        assert!(result.contains("smart_home_control_scene"));
        assert!(result.contains("回家模式"));
    }

    #[tokio::test]
    async fn smart_home_light_normalizes_common_kelvin_short_forms() {
        let absolute = run_tool(
            "smart_home_control_light_temperature",
            json!({
                "tool_title": "设置色温",
                "action": "set_color_temperature",
                "color_temperature_kelvin": 400
            }),
        )
        .await
        .unwrap();
        let tone = run_tool(
            "smart_home_control_light_temperature",
            json!({
                "tool_title": "设置冷光",
                "action": "set_light_tone",
                "light_tone": "cool",
                "color_temperature_kelvin": 600
            }),
        )
        .await
        .unwrap();
        let delta = run_tool(
            "smart_home_control_light_temperature",
            json!({
                "tool_title": "调大色温",
                "action": "increase_color_temperature",
                "delta_kelvin": 120
            }),
        )
        .await
        .unwrap();

        assert!(absolute.contains(r#""color_temperature_kelvin": 4000"#));
        assert!(tone.contains(r#""color_temperature_kelvin": 6000"#));
        assert!(delta.contains(r#""delta_kelvin": 1200"#));
    }

    #[tokio::test]
    async fn smart_home_normalizes_default_control_values() {
        let volume = run_tool(
            "smart_home_control_speaker",
            json!({
                "tool_title": "调大音量",
                "action": "increase_volume"
            }),
        )
        .await
        .unwrap();
        let mute = run_tool(
            "smart_home_control_speaker",
            json!({
                "tool_title": "扬声器静音",
                "action": "mute"
            }),
        )
        .await
        .unwrap();
        let brightness = run_tool(
            "smart_home_control_light",
            json!({
                "tool_title": "调亮灯光",
                "action": "increase_brightness"
            }),
        )
        .await
        .unwrap();
        let temperature = run_tool(
            "smart_home_control_light_temperature",
            json!({
                "tool_title": "调冷灯光",
                "action": "increase_color_temperature"
            }),
        )
        .await
        .unwrap();

        assert!(volume.contains(r#""delta_percent": 10"#));
        assert!(mute.contains(r#""volume_percent": 0"#));
        assert!(brightness.contains(r#""delta_percent": 20"#));
        assert!(temperature.contains(r#""delta_kelvin": 500"#));
    }

    #[test]
    fn digital_life_schemas_expose_grouped_actions() {
        let specs = tool_specs();
        let document = specs
            .iter()
            .find(|spec| spec.function.name == "digital_document_workspace")
            .unwrap();
        let media = specs
            .iter()
            .find(|spec| spec.function.name == "digital_media_control")
            .unwrap();

        assert!(document.function.parameters["properties"]["action"]["enum"]
            .as_array()
            .unwrap()
            .iter()
            .any(|value| value == "pdf_merge"));
        assert!(document.function.parameters["properties"]["action"]["enum"]
            .as_array()
            .unwrap()
            .iter()
            .any(|value| value == "ocr_extract_text"));
        assert!(media.function.parameters["properties"]["action"]["enum"]
            .as_array()
            .unwrap()
            .iter()
            .any(|value| value == "cast_to_device"));
    }

    #[tokio::test]
    async fn digital_file_manager_reads_current_directory_properties() {
        let result = run_tool(
            "digital_file_manager",
            json!({
                "tool_title": "查看目录属性",
                "action": "get_properties",
                "path": "."
            }),
        )
        .await
        .unwrap();

        assert!(result.contains(r#""tool": "digital_file_manager""#));
        assert!(result.contains(r#""kind": "directory""#));
    }

    #[tokio::test]
    async fn digital_life_mock_tools_return_normalized_payload() {
        let result = run_tool(
            "digital_media_control",
            json!({
                "tool_title": "投屏到客厅电视",
                "action": "cast_to_device",
                "target_device": "客厅电视"
            }),
        )
        .await
        .unwrap();

        assert!(result.contains(r#""status": "accepted""#));
        assert!(result.contains(r#""tool": "digital_media_control""#));
        assert!(result.contains("客厅电视"));
    }
}
