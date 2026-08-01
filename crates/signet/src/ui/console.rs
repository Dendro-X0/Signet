use std::io::{self, IsTerminal, Write};
use std::path::Path;
use std::sync::OnceLock;

use crossterm::style::{Color, Print, ResetColor, SetForegroundColor, Stylize};
use crossterm::QueueableCommand;

const RULE: &str = "────────────────────────────────────────";

fn color_enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| {
        if std::env::var_os("NO_COLOR").is_some() {
            return false;
        }
        if std::env::var_os("SIGNET_FORCE_COLOR").is_some() {
            return true;
        }
        io::stdout().is_terminal()
    })
}

fn paint(text: &str, color: Color) -> String {
    if color_enabled() {
        text.with(color).to_string()
    } else {
        text.to_string()
    }
}

fn paint_bold(text: &str, color: Color) -> String {
    if color_enabled() {
        text.with(color).bold().to_string()
    } else {
        text.to_string()
    }
}

fn accent(text: &str) -> String {
    paint(text, Color::Cyan)
}

fn accent_bold(text: &str) -> String {
    paint_bold(text, Color::Cyan)
}

fn title(text: &str) -> String {
    paint_bold(text, Color::White)
}

fn dim(text: &str) -> String {
    paint(text, Color::DarkGrey)
}

fn ok_color(text: &str) -> String {
    paint(text, Color::Green)
}

fn warn_color(text: &str) -> String {
    paint(text, Color::Yellow)
}

/// Flow / command banner — framed like the hub panels.
pub fn banner(title_text: &str) {
    let top = format!("┌─ Signet · {title_text}");
    let bottom = format!("└{RULE}");
    println!("{}", accent(&top));
    println!("{}", accent(&bottom));
}

/// Section header with a light rule.
pub fn section(title_text: &str) {
    println!();
    println!("{}", accent_bold(title_text));
    println!("{}", dim(RULE));
}

pub fn blank() {
    println!();
}

/// Aligned key = value row (keys padded to `width`).
pub fn kv(width: usize, key: &str, value: &str) {
    let k = dim(&format!("{key:<width$}"));
    let eq = dim(" = ");
    println!("  {k}{eq}{}", title(value));
}

/// Two-column status: key + ok/missing.
pub fn status(width: usize, key: &str, ok: bool, detail: &str) {
    let mark = if ok {
        ok_color("ok     ")
    } else {
        warn_color("missing")
    };
    let k = dim(&format!("{key:<width$}"));
    if detail.is_empty() {
        println!("  {k}  {mark}");
    } else {
        println!("  {k}  {mark}  {}", dim(detail));
    }
}

pub fn bullet(text: &str) {
    println!("  {} {text}", accent("•"));
}

pub fn muted(text: &str) {
    println!("{}", dim(&format!("  ({text})")));
}

pub fn note(text: &str) {
    println!("  {}  {text}", accent("note"));
}

pub fn step(current: usize, total: usize, label: &str, done: bool) {
    let state = if done {
        ok_color("done")
    } else {
        dim("…")
    };
    println!(
        "{} step {current}/{total}: {} ({state})",
        dim("—"),
        title(label)
    );
}

pub fn step_active(current: usize, total: usize, label: &str) {
    println!();
    println!(
        "{} step {current}/{total}: {}",
        accent("—"),
        title(label)
    );
}

pub fn numbered(n: usize, command: &str, why: &str) {
    println!("  {}. {}", dim(&n.to_string()), accent_bold(command));
    println!("     {} {}", dim("→"), dim(why));
}

pub fn ok_line(text: &str) {
    println!("  {} {text}", ok_color("✓"));
}

pub fn skip_line(text: &str) {
    println!("  {} {text}", dim("·"));
}

/// Platform group header inside installers section.
pub fn platform_header(platform: &str, count: usize) {
    println!(
        "  {}  {}",
        accent_bold(platform),
        dim(&format!("({count} found)"))
    );
}

/// Choice marker for numbered lists (▸ selected).
pub fn choice_row(selected: bool, index: usize, label: &str) {
    let mark = if selected {
        accent("▸")
    } else {
        dim(" ")
    };
    let num = dim(&format!("{}.", index));
    let name = if selected {
        title(label)
    } else {
        label.to_string()
    };
    println!("  {mark} {num} {name}");
}

/// Prompt chrome — accent label, dim default hint. Does not read input.
pub fn write_prompt(label: &str, default_hint: Option<&str>) -> io::Result<()> {
    let mut stdout = io::stdout();
    if color_enabled() {
        stdout.queue(SetForegroundColor(Color::Cyan))?;
        stdout.queue(Print(format!("  {label}")))?;
        stdout.queue(ResetColor)?;
        if let Some(hint) = default_hint {
            stdout.queue(SetForegroundColor(Color::DarkGrey))?;
            stdout.queue(Print(format!(" [{hint}]")))?;
            stdout.queue(ResetColor)?;
        }
        stdout.queue(Print(": "))?;
    } else if let Some(hint) = default_hint {
        write!(stdout, "  {label} [{hint}]: ")?;
    } else {
        write!(stdout, "  {label}: ")?;
    }
    stdout.flush()
}

/// Path relative to root when possible; forward slashes.
pub fn display_path(root: &Path, path: &Path) -> String {
    let rel = path
        .strip_prefix(root)
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|_| path.to_path_buf());
    let s = rel.to_string_lossy().replace('\\', "/");
    if s.is_empty() {
        ".".into()
    } else {
        s
    }
}
