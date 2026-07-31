use std::path::Path;

const RULE: &str = "────────────────────────────────────────";

/// Flow / command banner.
pub fn banner(title: &str) {
    println!("┌─ selfsign · {title}");
    println!("└{RULE}");
}

/// Section header with a light rule.
pub fn section(title: &str) {
    println!();
    println!("{title}");
    println!("{RULE}");
}

pub fn blank() {
    println!();
}

/// Aligned key = value row (keys padded to `width`).
pub fn kv(width: usize, key: &str, value: &str) {
    println!("  {key:<width$} = {value}");
}

/// Two-column status: key + ok/missing.
pub fn status(width: usize, key: &str, ok: bool, detail: &str) {
    let mark = if ok { "ok     " } else { "missing" };
    if detail.is_empty() {
        println!("  {key:<width$}  {mark}");
    } else {
        println!("  {key:<width$}  {mark}  {detail}");
    }
}

pub fn bullet(text: &str) {
    println!("  • {text}");
}

pub fn muted(text: &str) {
    println!("  ({text})");
}

pub fn note(text: &str) {
    println!("  note  {text}");
}

pub fn step(current: usize, total: usize, label: &str, done: bool) {
    let state = if done { "done" } else { "…" };
    println!("— step {current}/{total}: {label} ({state})");
}

pub fn step_active(current: usize, total: usize, label: &str) {
    println!();
    println!("— step {current}/{total}: {label}");
}

pub fn numbered(n: usize, command: &str, why: &str) {
    println!("  {n}. {command}");
    println!("     → {why}");
}

pub fn ok_line(text: &str) {
    println!("  ✓ {text}");
}

pub fn skip_line(text: &str) {
    println!("  · {text}");
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
