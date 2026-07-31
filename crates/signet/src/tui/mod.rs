//! Interactive TUI hub — guided flows over the same command modules as the CLI.

mod flows;
mod prompts;
mod status;
mod theme;

use std::io::{self, IsTerminal, Write};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap};
use ratatui::Frame;
use ratatui::Terminal;

use status::ProjectStatus;
use theme::{accent, dim, title_style};

#[derive(Clone, Copy)]
struct HubItem {
    id: &'static str,
    label: &'static str,
    hint: &'static str,
}

const ITEMS: &[HubItem] = &[
    HubItem {
        id: "guided",
        label: "Guided setup",
        hint: "First-release wizard (init → identity → trust → build → release)",
    },
    HubItem {
        id: "scan",
        label: "Scan",
        hint: "Find installers + suggest signing config",
    },
    HubItem {
        id: "doctor",
        label: "Doctor",
        hint: "Check host tooling and auth",
    },
    HubItem {
        id: "init",
        label: "Init",
        hint: "Create signet.toml + .signet/",
    },
    HubItem {
        id: "identity",
        label: "Identity",
        hint: "Create or show signing identity",
    },
    HubItem {
        id: "trust",
        label: "Trust",
        hint: "Write TRUST.md from active identity",
    },
    HubItem {
        id: "build",
        label: "Build",
        hint: "Build + sign (Tauri today; more next)",
    },
    HubItem {
        id: "release",
        label: "Release",
        hint: "Checksums + GitHub Release (guided)",
    },
    HubItem {
        id: "quit",
        label: "Quit",
        hint: "Exit Signet",
    },
];

pub fn run_hub() -> anyhow::Result<()> {
    if !io::stdout().is_terminal() {
        println!("signet — interactive TUI requires a terminal");
        println!("Try: signet --help");
        println!("Or:  signet doctor");
        return Ok(());
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    stdout.execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut state = ListState::default();
    state.select(Some(recommended_index()));
    let mut flash: Option<String> = None;

    let result = loop {
        let status = ProjectStatus::probe(".");
        terminal.draw(|frame| draw_hub(frame, &mut state, &status, flash.as_deref()))?;

        let Event::Key(key) = event::read()? else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }

        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => break Ok(()),
            KeyCode::Down | KeyCode::Char('j') => {
                let i = state.selected().unwrap_or(0);
                state.select(Some((i + 1) % ITEMS.len()));
            }
            KeyCode::Up | KeyCode::Char('k') => {
                let i = state.selected().unwrap_or(0);
                state.select(Some(if i == 0 { ITEMS.len() - 1 } else { i - 1 }));
            }
            KeyCode::Char(c) if c.is_ascii_digit() => {
                let n = c.to_digit(10).unwrap_or(0) as usize;
                if (1..=ITEMS.len()).contains(&n) {
                    state.select(Some(n - 1));
                }
            }
            KeyCode::Enter => {
                let i = state.selected().unwrap_or(0);
                let id = ITEMS[i].id;
                if id == "quit" {
                    break Ok(());
                }

                restore_terminal(&mut terminal)?;
                flash = None;
                let cmd_result = dispatch(id);
                match &cmd_result {
                    Ok(()) => println!("\n✓ done"),
                    Err(err) => eprintln!("\n✗ {err}"),
                }
                pause_return();

                enable_raw_mode()?;
                io::stdout().execute(EnterAlternateScreen)?;
                terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
                state.select(Some(i));
                if let Err(err) = cmd_result {
                    flash = Some(format!("last error: {err}"));
                }
            }
            _ => {}
        }
    };

    restore_terminal(&mut terminal)?;
    result
}

fn recommended_index() -> usize {
    let status = ProjectStatus::probe(".");
    let want = status.recommended_action();
    ITEMS
        .iter()
        .position(|i| i.id == want)
        .unwrap_or(0)
}

fn draw_hub(frame: &mut Frame, state: &mut ListState, status: &ProjectStatus, flash: Option<&str>) {
    let area = frame.area();
    frame.render_widget(Clear, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(5),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(area);

    draw_header(frame, chunks[0]);
    draw_status(frame, chunks[1], status, flash);
    draw_menu(frame, chunks[2], state, status);
    draw_footer(frame, chunks[3]);
}

fn draw_header(frame: &mut Frame, area: Rect) {
    let line = Line::from(vec![
        Span::styled(" Signet ", title_style()),
        Span::styled("· self-signed desktop & mobile releases", dim()),
    ]);
    let widget = Paragraph::new(line).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(accent())
            .title(Span::styled(" hub ", accent())),
    );
    frame.render_widget(widget, area);
}

fn draw_status(frame: &mut Frame, area: Rect, status: &ProjectStatus, flash: Option<&str>) {
    let marks = |ok: bool| if ok { "✓" } else { "·" };
    let line1 = Line::from(vec![
        Span::raw(format!(
            " {} config  {} identity  {} trust  {} artifacts",
            marks(status.has_config),
            marks(status.has_identity),
            marks(status.has_trust),
            marks(status.has_artifacts),
        )),
    ]);
    let line2 = Line::from(vec![
        Span::styled(" next ", accent()),
        Span::raw(status.next_hint()),
    ]);
    let mut lines = vec![line1, line2];
    if let Some(msg) = flash {
        lines.push(Line::from(Span::styled(format!(" {msg}"), dim())));
    }
    let widget = Paragraph::new(lines)
        .wrap(Wrap { trim: true })
        .block(Block::default().borders(Borders::ALL).title(" project "));
    frame.render_widget(widget, area);
}

fn draw_menu(frame: &mut Frame, area: Rect, state: &mut ListState, status: &ProjectStatus) {
    let recommended = status.recommended_action();
    let items: Vec<ListItem> = ITEMS
        .iter()
        .enumerate()
        .map(|(idx, item)| {
            // Star sits after the number so ▸ / digits stay uncrowded: " 5. ★  …"
            let star = if item.id == recommended { "★" } else { " " };
            ListItem::new(Line::from(vec![
                Span::styled(format!(" {:>2}. {star}  ", idx + 1), dim()),
                Span::styled(format!("{:<14}", item.label), title_style()),
                Span::styled(item.hint, dim()),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" actions "))
        .highlight_style(theme::highlight())
        .highlight_symbol("▸ ");
    frame.render_stateful_widget(list, area, state);
}

fn draw_footer(frame: &mut Frame, area: Rect) {
    let widget = Paragraph::new(Line::from(vec![
        Span::styled(" ↑↓/jk ", accent()),
        Span::raw("move  "),
        Span::styled("enter ", accent()),
        Span::raw("run  "),
        Span::styled("1-9 ", accent()),
        Span::raw("jump  "),
        Span::styled("q ", accent()),
        Span::raw("quit · same engines as CLI flags"),
    ]))
    .block(Block::default().borders(Borders::ALL));
    frame.render_widget(widget, area);
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> anyhow::Result<()> {
    disable_raw_mode()?;
    terminal.backend_mut().execute(LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

fn pause_return() {
    print!("\nPress Enter to return to hub…");
    let _ = io::stdout().flush();
    let mut buf = String::new();
    let _ = io::stdin().read_line(&mut buf);
}

fn dispatch(id: &str) -> anyhow::Result<()> {
    match id {
        "guided" => flows::guided_setup(),
        "scan" => flows::guided_scan(),
        "doctor" => flows::run_doctor(),
        "init" => flows::guided_init(),
        "identity" => flows::guided_identity(),
        "trust" => flows::run_trust(),
        "build" => flows::guided_build(),
        "release" => flows::guided_release(),
        _ => Ok(()),
    }
}
