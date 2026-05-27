# ROB

ROB 是 OpenOmniBot agent 框架的 Rust / Linux 原生 CLI 迁移版本。它把原 Android 项目中的「模型服务商配置、用户消息输入、agent 循环、工具调用、会话管理」这些核心概念迁移到终端环境，目标是让开发者可以在 Linux 命令行中快速配置模型并与 agent 对话。

当前版本是一个可运行的框架骨架，适合继续扩展成更完整的 Linux agent。

## 目前能做什么

- 在 CLI 中配置多个模型服务商 profile。
- 使用 OpenAI-compatible `/chat/completions` API 与模型对话。
- 支持一次性提问和交互式聊天。
- 支持全屏 TUI 聊天界面。
- 支持 OpenAI-compatible SSE 流式输出。
- 支持模型发起 tool call，并由本地 Linux 工具执行。
- 支持工具执行审批策略：自动执行或交互式确认。
- 自动保存 agent 会话，并支持恢复历史会话。
- 提供基础单元测试和 CLI smoke check 入口。

## 快速开始

进入项目目录：

```bash
cd /mnt/emmc/lxm/ROB
```

构建：

```bash
cargo build
```

运行测试：

```bash
cargo test
```

查看 CLI：

```bash
cargo run -- --help
```

## 配置模型服务商

推荐把 API key 放在环境变量里，不直接写入配置文件。

OpenAI-compatible 示例：

```bash
export OPENAI_API_KEY=your_api_key

cargo run -- config set \
  --name default \
  --base-url https://api.openai.com/v1 \
  --api-key-env OPENAI_API_KEY \
  --model your-model
```

DeepSeek 示例：

```bash
export DEEPSEEK_API_KEY=your_api_key

cargo run -- config set \
  --name deepseek \
  --base-url https://api.deepseek.com/v1 \
  --api-key-env DEEPSEEK_API_KEY \
  --model deepseek-chat
```

常用配置命令：

```bash
cargo run -- config show
cargo run -- config list
cargo run -- config use deepseek
cargo run -- config path
```

上下文窗口管理和思考强度：

```bash
cargo run -- config set-context --token-threshold 32000 --recent-messages 12
cargo run -- config set-reasoning medium
```

`set-context` 会控制发给模型的运行上下文：完整 session 仍然落盘，但超过估算 token 阈值时，ROB 会把较早消息折叠为上下文摘要，并保留最近消息原文。

`set-reasoning` 支持 `auto`、`no`、`low`、`medium`、`high`。其中 `no` 会在兼容服务商上发送 `enable_thinking=false`，其他强度会发送 OpenAI-compatible 的 `reasoning_effort`。

配置文件默认保存到用户配置目录：

```text
~/.config/rob/config.toml
```

也可以通过 `ROB_CONFIG` 指定配置文件路径：

```bash
ROB_CONFIG=/tmp/rob-config.toml cargo run -- config show
```

## 与 Agent 对话

一次性提问：

```bash
cargo run -- ask "List files in the current directory"
```

`ask` 会边接收模型 delta 边打印到 stdout。

交互式聊天：

```bash
cargo run -- chat
```

`chat` 会在每轮回答中流式打印内容。

全屏 TUI 聊天：

```bash
cargo run -- tui
```

`tui` 会在消息区实时追加模型输出，并显示工具调用的开始、完成、失败或拒绝状态。

交互式命令：

- `/help`：显示交互命令。
- `/tools`：列出当前可用工具。
- `/config`：显示当前模型服务商配置摘要。
- `/id`：显示当前会话 id。
- `/exit`：退出聊天。

临时覆盖模型：

```bash
cargo run -- chat --model another-model
cargo run -- tui --model another-model
cargo run -- ask "hello" --model another-model
```

TUI 快捷键：

- `Enter`：发送消息。
- `Esc` 或 `Ctrl-C`：退出 TUI。
- `Backspace`：删除输入字符。
- `Ctrl-U`：清空输入框。
- `Up` / `Down`：滚动消息区。

TUI 内支持 `/help`、`/id`、`/config`、`/exit`。

## 工具调用

当前支持的 Linux 工具：

- `pwd`：返回当前工作目录。
- `list_dir`：列出目录内容。
- `read_file`：读取 UTF-8 文本文件。
- `search_text`：使用 `rg` 搜索文本；没有 `rg` 时使用内置 fallback。
- `shell_exec`：执行小范围 allowlist 内的 Linux 命令。

查看工具：

```bash
cargo run -- tools list
```

手动运行工具：

```bash
cargo run -- tools run pwd
cargo run -- tools run list_dir '{"path":"src"}'
cargo run -- tools run shell_exec '{"command":"uname","args":["-s"]}'
```

`shell_exec` 不会调用 shell，只会以 `command + args` 形式执行，并限制在 allowlist 内。即使没有参数也必须传 `args: []`。如果模型连续返回缺少必填参数的工具调用，ROB 会先把修正提示写回模型；再次重复同类无效调用时会提前停止，避免在 agent loop 中空转到轮次上限。当前 allowlist 包含：

```text
pwd, ls, cat, head, tail, wc, rg, find, date, uname, whoami, df, du, ps, env
```

每次请求模型前，ROB 会像 OpenOmniBot 一样把工具使用规则和当前工具清单动态注入到运行上下文中，同时仍通过 OpenAI-compatible `tools` 字段传递完整 schema。这个运行上下文不会写入 session 历史。

## 工具审批策略

默认策略是 `auto`，表示模型请求的 allowlisted 工具会自动执行。

切换为交互确认：

```bash
cargo run -- config set-approval on-request
```

切回自动执行：

```bash
cargo run -- config set-approval auto
```

也可以只对当前运行覆盖：

```bash
cargo run -- chat --approval on-request
cargo run -- tui --approval auto
cargo run -- ask "show current directory" --approval auto
```

注意：`on-request` 只有在 `chat` 交互模式下会询问用户；`tui` 和非交互式 `ask` 下会拒绝工具执行，避免后台命令无提示运行。

## 会话管理

ROB 会把对话消息保存到用户 state 目录，便于继续上下文。

列出会话：

```bash
cargo run -- sessions list
```

恢复会话：

```bash
cargo run -- chat --resume <session-id>
cargo run -- tui --resume <session-id>
```

查看会话 JSON：

```bash
cargo run -- sessions show <session-id>
```

默认会话目录位于用户 state 目录下：

```text
~/.local/state/rob/sessions
```

也可以通过 `ROB_STATE` 指定：

```bash
ROB_STATE=/tmp/rob-state cargo run -- sessions list
```

## 代码结构

```text
src/main.rs    CLI 入口，定义 config/chat/tui/ask/tools/sessions 子命令。
src/config.rs  模型服务商 profile、配置文件读写、审批策略。
src/llm.rs     OpenAI-compatible chat completions 请求和消息结构。
src/agent.rs   AgentSession、消息循环、tool-call 执行和会话持久化。
src/tui.rs     ratatui/crossterm 终端 UI。
src/tools.rs   Linux 工具 schema、参数解析、执行和安全限制。
src/state.rs   会话保存、读取、列表和 state 目录管理。
```

核心流程：

1. `main.rs` 解析 CLI 命令。
2. `config.rs` 读取 active provider。
3. `agent.rs` 创建或恢复 `AgentSession`。
4. `context.rs` 根据估算 token 阈值构造运行上下文，必要时把较早消息压缩成摘要。
5. `tools.rs` 生成工具使用规则和工具清单，并作为运行时 system context 注入本轮请求。
6. 用户消息进入 `llm.rs` 的 chat-completions 请求，附带 reasoning/thinking 控制参数和完整 tool schema。
7. `llm.rs` 按 SSE `data:` chunk 解析流式输出、reasoning 内容、usage 和 tool calls。
8. 如果模型返回 tool calls，`agent.rs` 先追加 assistant tool_call 消息，再根据审批策略调用 `tools.rs`。
9. 每个 tool 结果都会追加为 tool 消息并持久化，然后继续下一轮模型请求，直到模型返回最终回答。
10. 每段 user / assistant / tool 消息都保存到 `state.rs` 管理的完整 session 文件。

## 开发指南

格式检查：

```bash
cargo fmt --check
```

运行测试：

```bash
cargo test
```

构建：

```bash
cargo build
```

推荐的本地 smoke check：

```bash
cargo run -- --help
cargo run -- config --help
cargo run -- chat --help
cargo run -- tui --help
cargo run -- tools list
cargo run -- tools run pwd
cargo run -- tools run list_dir '{"path":"src"}'
cargo run -- sessions list
```

使用临时配置文件测试 provider 配置：

```bash
ROB_CONFIG=/tmp/rob-config.toml cargo run -- config set \
  --name smoke \
  --base-url https://example.test/v1 \
  --api-key-env ROB_TEST_KEY \
  --model smoke-model

ROB_CONFIG=/tmp/rob-config.toml ROB_TEST_KEY=secret cargo run -- config show
```

## 扩展方向

适合继续开发的方向：

- 增强 streaming 输出，例如 token 统计、reasoning delta 和取消控制。
- 增加 Anthropic / DeepSeek 专用 adapter。
- 增加更丰富的 Linux 工具，例如文件写入、补丁应用、进程检查。
- 增加类似 Codex 的 sandbox / approval 分层策略。
- 增强 TUI 的 streaming、工具审批弹窗和历史搜索。
- 将 OpenOmniBot 中更多 scene / task / host request 概念迁移到 Rust 类型系统。
- 增加集成测试，使用 mock HTTP server 验证真实 tool-call loop。

## 当前定位

ROB 不是 Android UI 自动化迁移的完整替代品，也不是完整的 Codex 复刻。它当前的作用是：

- 作为 OpenOmniBot agent 框架迁移到 Linux CLI 的起点。
- 提供可运行的 Rust agent loop。
- 提供模型服务商配置、消息输入、工具调用、会话管理这些基础能力。
- 为后续扩展 Linux 原生 agent 能力提供清晰的模块边界。
