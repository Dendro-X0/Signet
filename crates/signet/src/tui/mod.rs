//! Interactive TUI hub — guided flows over the same command modules as the CLI.

mod flows;
mod framework_pick;
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
use ratatui::widgets::{Clear, List, ListItem, ListState, Paragraph, Wrap};
use ratatui::Frame;
use ratatui::Terminal;

use status::ProjectStatus;
use theme::{accent, dim, ok, panel, title_style, warn};

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
        hint: "Sign → Prove → Check wizard (framework pick + verify/inspect)",
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
        hint: "Sign · create or show signing identity",
    },
    HubItem {
        id: "trust",
        label: "Trust",
        hint: "Prove · write TRUST.md from active identity",
    },
    HubItem {
        id: "build",
        label: "Build",
        hint: "Sign · build + sign (any configured framework)",
    },
    HubItem {
        id: "verify",
        label: "Verify",
        hint: "Check · fingerprints + SHA256SUMS",
    },
    HubItem {
        id: "inspect",
        label: "Inspect",
        hint: "Check · signed/unsigned per platform",
    },
    HubItem {
        id: "graduate",
        label: "Graduate notes",
        hint: "Official Sign · OV / Azure / notarize honesty",
    },
    HubItem {
        id: "release",
        label: "Release",
        hint: "Prove · checksums + GitHub Release",
    },
    HubItem {
        id: "self-status",
        label: "CLI status",
        hint: "How this Signet binary was installed",
    },
    HubItem {
        id: "self-update",
        label: "Update Signet",
        hint: "Download latest CLI from GitHub Releases",
    },
    HubItem {
        id: "self-uninstall",
        label: "Uninstall Signet",
        hint: "Remove installer-managed CLI (not project .signet/)",
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
                    Ok(()) => crate::ui::console::ok_line("done"),
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
        Span::styled("· ", dim()),
        Span::styled("Sign", accent()),
        Span::styled(" → ", dim()),
        Span::styled("Prove", accent()),
        Span::styled(" → ", dim()),
        Span::styled("Check", accent()),
    ]);
    let widget = Paragraph::new(line).block(panel("hub"));
    frame.render_widget(widget, area);
}

fn draw_status(frame: &mut Frame, area: Rect, status: &ProjectStatus, flash: Option<&str>) {
    let mark = |ready: bool| -> Span<'static> {
        if ready {
            Span::styled("✓", ok())
        } else {
            Span::styled("·", dim())
        }
    };
    let label = |text: &'static str| Span::styled(text, dim());
    let line1 = Line::from(vec![
        Span::raw(" "),
        mark(status.has_config),
        label(" config  "),
        mark(status.has_identity),
        label(" identity  "),
        mark(status.has_trust),
        label(" trust  "),
        mark(status.has_artifacts),
        label(" artifacts"),
    ]);
    let mut next_spans = vec![Span::styled(" next ", accent())];
    next_spans.extend(phase_hint_spans(&status.next_hint()));
    let line2 = Line::from(next_spans);
    let mut lines = vec![line1, line2];
    if let Some(msg) = flash {
        lines.push(Line::from(vec![
            Span::raw(" "),
            Span::styled(msg.to_string(), warn()),
        ]));
    }
    let widget = Paragraph::new(lines)
        .wrap(Wrap { trim: true })
        .block(panel("project"));
    frame.render_widget(widget, area);
}

fn draw_menu(frame: &mut Frame, area: Rect, state: &mut ListState, status: &ProjectStatus) {
    let recommended = status.recommended_action();
    // Pad labels to the longest name so hints never collide ("Graduate notes" / "Uninstall Signet").
    let label_width = ITEMS
        .iter()
        .map(|i| i.label.chars().count())
        .max()
        .unwrap_or(14)
        .max(14);
    // Prefix: "▸ " (highlight) + " 12. ★  " ≈ leave room; clip hint to remaining columns.
    let prefix_cols = 2usize.saturating_add(8); // highlight symbol + " NN. ★  "
    let inner_width = area.width.saturating_sub(2) as usize; // borders
    let hint_budget = inner_width
        .saturating_sub(prefix_cols)
        .saturating_sub(label_width)
        .saturating_sub(2); // gap before hint

    let items: Vec<ListItem> = ITEMS
        .iter()
        .enumerate()
        .map(|(idx, item)| {
            // Star sits after the number so ▸ / digits stay uncrowded: " 5. ★  …"
            let star = if item.id == recommended { "★" } else { " " };
            let star_style = if item.id == recommended {
                accent()
            } else {
                dim()
            };
            let label = format!("{:<width$}", item.label, width = label_width);
            let hint = truncate_hint(item.hint, hint_budget);
            let mut spans = vec![
                Span::styled(format!(" {:>2}. ", idx + 1), dim()),
                Span::styled(format!("{star}  "), star_style),
                Span::styled(label, title_style()),
                Span::raw("  "),
            ];
            spans.extend(phase_hint_spans(&hint));
            ListItem::new(Line::from(spans))
        })
        .collect();

    let list = List::new(items)
        .block(panel("actions"))
        .highlight_style(theme::highlight())
        .highlight_symbol("▸ ");
    frame.render_stateful_widget(list, area, state);
}

/// Highlight Sign / Prove / Check phase words; leave the rest dim.
fn phase_hint_spans(hint: &str) -> Vec<Span<'static>> {
    const PHASES: &[&str] = &["Official Sign", "Sign", "Prove", "Check"];
    let mut out = Vec::new();
    let mut rest = hint.to_string();
    // Emit owned spans so ListItem lifetimes stay simple.
    while !rest.is_empty() {
        let mut hit: Option<(usize, &str)> = None;
        for phase in PHASES {
            if let Some(i) = rest.find(phase) {
                match hit {
                    Some((best, _)) if i >= best => {}
                    _ => hit = Some((i, *phase)),
                }
            }
        }
        let Some((i, phase)) = hit else {
            out.push(Span::styled(rest, dim()));
            break;
        };
        if i > 0 {
            out.push(Span::styled(rest[..i].to_string(), dim()));
        }
        out.push(Span::styled(phase.to_string(), accent()));
        rest = rest[i + phase.len()..].to_string();
    }
    out
}

fn truncate_hint(hint: &str, max_chars: usize) -> String {
    if max_chars == 0 {
        return String::new();
    }
    let count = hint.chars().count();
    if count <= max_chars {
        return hint.to_string();
    }
    if max_chars <= 1 {
        return "…".to_string();
    }
    let keep = max_chars - 1;
    let mut s: String = hint.chars().take(keep).collect();
    s.push('…');
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn label_column_fits_longest_hub_item() {
        let w = ITEMS
            .iter()
            .map(|i| i.label.chars().count())
            .max()
            .unwrap();
        assert!(w >= "Uninstall Signet".chars().count());
        assert!(w >= "Graduate notes".chars().count());
    }

    #[test]
    fn truncate_hint_adds_ellipsis() {
        assert_eq!(truncate_hint("abcdef", 4), "abc…");
        assert_eq!(truncate_hint("hi", 10), "hi");
        assert_eq!(truncate_hint("x", 0), "");
    }

    #[test]
    fn phase_hint_highlights_sign_prove_check() {
        let spans = phase_hint_spans("Sign · build + sign");
        assert!(spans.iter().any(|s| s.content == "Sign"));
        let spans = phase_hint_spans("Official Sign · OV");
        assert_eq!(spans[0].content, "Official Sign");
    }
}

fn draw_footer(frame: &mut Frame, area: Rect) {
    let widget = Paragraph::new(Line::from(vec![
        Span::styled(" ↑↓/jk ", accent()),
        Span::styled("move  ", dim()),
        Span::styled("enter ", accent()),
        Span::styled("run  ", dim()),
        Span::styled("1-9 ", accent()),
        Span::styled("jump  ", dim()),
        Span::styled("q ", accent()),
        Span::styled("quit · digits wrap past 9 via arrows", dim()),
    ]))
    .block(panel("keys"));
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
        "verify" => flows::guided_verify(),
        "inspect" => flows::guided_inspect(),
        "graduate" => flows::run_graduate_notes(),
        "release" => flows::guided_release(),
        "self-status" => flows::run_self_status(),
        "self-update" => flows::run_self_update(),
        "self-uninstall" => flows::run_self_uninstall(),
        _ => Ok(()),
    }
}
