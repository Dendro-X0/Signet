use std::io::{self, Write};

use crate::ui::console;

/// Prompt for a line; empty input keeps `default`.
pub fn prompt_line(label: &str, default: &str) -> io::Result<String> {
    if default.is_empty() {
        print!("  {label}: ");
    } else {
        print!("  {label} [{default}]: ");
    }
    io::stdout().flush()?;
    let mut buf = String::new();
    io::stdin().read_line(&mut buf)?;
    let trimmed = buf.trim();
    if trimmed.is_empty() {
        Ok(default.to_string())
    } else {
        Ok(trimmed.to_string())
    }
}

/// y/N or Y/n style confirm. `default_yes` controls empty answer.
pub fn confirm(label: &str, default_yes: bool) -> io::Result<bool> {
    let hint = if default_yes { "Y/n" } else { "y/N" };
    print!("  {label} [{hint}]: ");
    io::stdout().flush()?;
    let mut buf = String::new();
    io::stdin().read_line(&mut buf)?;
    let t = buf.trim().to_ascii_lowercase();
    if t.is_empty() {
        return Ok(default_yes);
    }
    Ok(matches!(t.as_str(), "y" | "yes"))
}

/// Pick 1-based index from labels; returns 0-based. Empty → default_idx.
pub fn prompt_choice(label: &str, options: &[&str], default_idx: usize) -> io::Result<usize> {
    console::section(label);
    for (i, opt) in options.iter().enumerate() {
        let mark = if i == default_idx { "▸" } else { " " };
        println!("  {mark} {}. {opt}", i + 1);
    }
    let default = (default_idx + 1).to_string();
    let answer = prompt_line("choice", &default)?;
    let n: usize = answer.parse().unwrap_or(default_idx + 1);
    if n == 0 || n > options.len() {
        Ok(default_idx)
    } else {
        Ok(n - 1)
    }
}
