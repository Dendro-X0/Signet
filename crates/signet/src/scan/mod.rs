//! Repository self-check: find installable apps and suggest signing config.

mod report;
mod walk;

pub use report::{print_human, ProjectKind, ScanReport};
pub use walk::scan_repository;
