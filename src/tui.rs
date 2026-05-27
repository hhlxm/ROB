use crate::agent::{AgentEvent, AgentSession};
use crate::config::{ApprovalPolicy, RobConfig};
use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Terminal,
};
use serde_json::Value;
use std::{io, time::Duration};

pub async fn run_tui(
    config: RobConfig,
    model_override: Option<String>,
    resume_id: Option<String>,
    approval_override: Option<ApprovalPolicy>,
) -> Result<()> {
    let mut session = AgentSession::new(config, model_override, resume_id, approval_override)?;
    let mut app = TuiApp::from_session(&session)?;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_loop(&mut terminal, &mut session, &mut app).await;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

struct TuiApp {
    input: String,
    lines: Vec<Line<'static>>,
    streaming_assistant: Option<String>,
    status: String,
    scroll: u16,
}

impl TuiApp {
    fn from_session(session: &AgentSession) -> Result<Self> {
        let mut app = Self {
            input: String::new(),
            lines: Vec::new(),
            streaming_assistant: None,
            status: format!("Session {} | {}", session.id(), session.config_summary()?),
            scroll: 0,
        };

        for message in session.transcript() {
            if message.role == "system" {
                continue;
            }

            match message.role.as_str() {
                "assistant" => {
                    if let Some(content) = &message.content {
                        if !content.trim().is_empty() {
                            app.push_message(&message.role, content);
                        }
                    }
                    if let Some(tool_calls) = &message.tool_calls {
                        for call in tool_calls {
                            let arguments = serde_json::from_str(&call.function.arguments)
                                .unwrap_or_else(|_| Value::Object(Default::default()));
                            app.push_tool_started(&call.id, &call.function.name, &arguments);
                        }
                    }
                }
                "tool" => {
                    let name = message.name.as_deref().unwrap_or("tool");
                    let id = message.tool_call_id.as_deref().unwrap_or("tool_call");
                    app.push_tool_result(id, name, message.content.as_deref().unwrap_or_default());
                }
                _ => {
                    if let Some(content) = &message.content {
                        if !content.trim().is_empty() {
                            app.push_message(&message.role, content);
                        }
                    }
                }
            }
        }

        if app.lines.is_empty() {
            app.push_system("Ready. Type a message and press Enter. Esc or Ctrl-C exits.");
        }

        Ok(app)
    }

    fn push_system(&mut self, content: &str) {
        self.lines.push(Line::from(vec![
            Span::styled("system", Style::default().fg(Color::Yellow)),
            Span::raw("  "),
            Span::raw(content.to_string()),
        ]));
        self.lines.push(Line::raw(""));
    }

    fn push_message(&mut self, role: &str, content: &str) {
        self.lines.push(message_line(role, content));
        self.lines.push(Line::raw(""));
    }

    fn push_assistant_delta(&mut self, delta: &str) {
        let content = self.streaming_assistant.get_or_insert_with(String::new);
        content.push_str(delta);
    }

    fn commit_assistant_stream(&mut self) {
        if let Some(content) = self.streaming_assistant.take() {
            if !content.trim().is_empty() {
                self.push_message("assistant", &content);
            }
        }
    }

    fn finish_assistant_stream(&mut self, fallback: &str) {
        let content = self
            .streaming_assistant
            .take()
            .filter(|content| !content.trim().is_empty())
            .unwrap_or_else(|| fallback.to_string());
        if content.trim().is_empty() {
            self.push_system("Model returned an empty response.");
        } else {
            self.push_message("assistant", &content);
        }
    }

    fn push_tool_event(
        &mut self,
        status: ToolStatus,
        id: &str,
        name: &str,
        detail: Option<String>,
    ) {
        self.commit_assistant_stream();

        let (status_text, color) = status.display();
        self.lines.push(Line::from(vec![
            Span::styled(
                "tool",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled(
                status_text,
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::styled(name.to_string(), Style::default().fg(Color::White)),
            Span::raw(format!("  #{id}")),
        ]));

        if let Some(detail) = detail.filter(|detail| !detail.trim().is_empty()) {
            self.lines.push(Line::from(vec![
                Span::raw("      "),
                Span::styled(detail, Style::default().fg(Color::DarkGray)),
            ]));
        }

        self.lines.push(Line::raw(""));
    }

    fn push_tool_started(&mut self, id: &str, name: &str, arguments: &Value) {
        self.push_tool_event(
            ToolStatus::Calling,
            id,
            name,
            Some(format!("args {}", format_json(arguments))),
        );
    }

    fn push_tool_completed(&mut self, id: &str, name: &str, output: &str) {
        self.push_tool_event(
            ToolStatus::Called,
            id,
            name,
            Some(format!("output {}", summarize_tool_output(output))),
        );
    }

    fn push_tool_result(&mut self, id: &str, name: &str, output: &str) {
        match classify_tool_output(output) {
            ToolStatus::Called => self.push_tool_completed(id, name, output),
            ToolStatus::Failed => self.push_tool_failed(id, name, output),
            ToolStatus::Denied => self.push_tool_denied(id, name),
            ToolStatus::Calling => {
                self.push_tool_started(id, name, &Value::Object(Default::default()))
            }
        }
    }

    fn push_tool_failed(&mut self, id: &str, name: &str, error: &str) {
        self.push_tool_event(
            ToolStatus::Failed,
            id,
            name,
            Some(format!("error {}", summarize_tool_output(error))),
        );
    }

    fn push_tool_denied(&mut self, id: &str, name: &str) {
        self.push_tool_event(ToolStatus::Denied, id, name, None);
    }

    fn push_agent_event(&mut self, event: AgentEvent) {
        match event {
            AgentEvent::AssistantDelta(delta) => {
                self.status = "Streaming model response...".to_string();
                self.push_assistant_delta(&delta);
            }
            AgentEvent::ToolCallStarted {
                id,
                name,
                arguments,
            } => {
                self.status = format!("Calling tool {name}...");
                self.push_tool_started(&id, &name, &arguments);
            }
            AgentEvent::ToolCallCompleted { id, name, output } => {
                self.status = format!("Tool {name} completed");
                self.push_tool_completed(&id, &name, &output);
            }
            AgentEvent::ToolCallFailed { id, name, error } => {
                self.status = format!("Tool {name} failed");
                self.push_tool_failed(&id, &name, &error);
            }
            AgentEvent::ToolCallDenied { id, name } => {
                self.status = format!("Tool {name} denied");
                self.push_tool_denied(&id, &name);
            }
        }
    }

    fn display_lines(&self) -> Vec<Line<'static>> {
        let mut lines = self.lines.clone();
        if let Some(content) = &self.streaming_assistant {
            lines.push(message_line("assistant", content));
        }
        lines
    }

    fn scroll_down(&mut self) {
        self.scroll = self.scroll.saturating_add(1);
    }

    fn scroll_up(&mut self) {
        self.scroll = self.scroll.saturating_sub(1);
    }
}

enum ToolStatus {
    Calling,
    Called,
    Failed,
    Denied,
}

fn classify_tool_output(output: &str) -> ToolStatus {
    let normalized = output.trim();
    if normalized == "tool denied by approval policy" {
        ToolStatus::Denied
    } else if normalized.starts_with("tool error:") {
        ToolStatus::Failed
    } else {
        ToolStatus::Called
    }
}

impl ToolStatus {
    fn display(&self) -> (&'static str, Color) {
        match self {
            Self::Calling => ("Calling", Color::Blue),
            Self::Called => ("Called", Color::Green),
            Self::Failed => ("Failed", Color::Red),
            Self::Denied => ("Denied", Color::Yellow),
        }
    }
}

fn message_line(role: &str, content: &str) -> Line<'static> {
    let color = match role {
        "user" => Color::Cyan,
        "assistant" => Color::Green,
        _ => Color::Gray,
    };
    Line::from(vec![
        Span::styled(
            role.to_string(),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::raw(content.to_string()),
    ])
}

fn format_json(value: &Value) -> String {
    truncate_chars(
        &serde_json::to_string(value).unwrap_or_else(|_| value.to_string()),
        240,
    )
}

fn summarize_tool_output(output: &str) -> String {
    let summary = output
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .take(3)
        .collect::<Vec<_>>()
        .join(" | ");

    if summary.is_empty() {
        "(empty)".to_string()
    } else {
        truncate_chars(&summary, 240)
    }
}

fn truncate_chars(value: &str, max_chars: usize) -> String {
    let mut truncated = value.chars().take(max_chars).collect::<String>();
    if value.chars().count() > max_chars {
        truncated.push_str("...");
    }
    truncated
}

async fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    session: &mut AgentSession,
    app: &mut TuiApp,
) -> Result<()> {
    loop {
        draw_ui(terminal, app)?;

        if !event::poll(Duration::from_millis(100))? {
            continue;
        }

        let Event::Key(key) = event::read()? else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }

        match key.code {
            KeyCode::Esc => break,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break,
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                app.input.clear();
            }
            KeyCode::Char(ch) => app.input.push(ch),
            KeyCode::Backspace => {
                app.input.pop();
            }
            KeyCode::Enter => {
                let input = app.input.trim().to_string();
                app.input.clear();
                if input.is_empty() {
                    continue;
                }
                if input == "/exit" || input == "/quit" {
                    break;
                }
                if input == "/help" {
                    app.push_system("/help, /id, /config, /exit are available. Enter sends.");
                    continue;
                }
                if input == "/id" {
                    app.push_system(session.id());
                    continue;
                }
                if input == "/config" {
                    app.push_system(&session.config_summary()?);
                    continue;
                }

                app.push_message("user", &input);
                app.status = "Waiting for model response...".to_string();
                draw_ui(terminal, app)?;

                match session
                    .send_user_message_events(&input, |event| {
                        app.push_agent_event(event);
                        draw_ui(terminal, app)?;
                        Ok(())
                    })
                    .await
                {
                    Ok(response) => {
                        app.finish_assistant_stream(&response);
                        app.status = format!("Session {} saved", session.id());
                    }
                    Err(error) => {
                        app.streaming_assistant = None;
                        app.push_system(&format!("Error: {error:#}"));
                        app.status = "Request failed".to_string();
                    }
                }
            }
            KeyCode::Up | KeyCode::PageUp => app.scroll_up(),
            KeyCode::Down | KeyCode::PageDown => app.scroll_down(),
            _ => {}
        }
    }

    Ok(())
}

fn draw_ui(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &TuiApp) -> Result<()> {
    terminal.draw(|frame| {
        let area = frame.size();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(5),
                Constraint::Length(3),
                Constraint::Length(1),
            ])
            .split(area);

        let transcript = Paragraph::new(app.display_lines())
            .block(Block::default().title("ROB Agent").borders(Borders::ALL))
            .wrap(Wrap { trim: false })
            .scroll((app.scroll, 0));
        frame.render_widget(transcript, chunks[0]);

        let input = Paragraph::new(app.input.as_str())
            .block(Block::default().title("Message").borders(Borders::ALL));
        frame.render_widget(input, chunks[1]);

        let status = Paragraph::new(app.status.as_str());
        frame.render_widget(status, chunks[2]);
    })?;
    Ok(())
}
