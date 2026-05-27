use crate::agent::AgentSession;
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
            if message.role == "system" || message.role == "tool" {
                continue;
            }
            if let Some(content) = &message.content {
                if !content.trim().is_empty() {
                    app.push_message(&message.role, content);
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
                    .send_user_message_streaming(&input, |delta| {
                        app.push_assistant_delta(delta);
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
