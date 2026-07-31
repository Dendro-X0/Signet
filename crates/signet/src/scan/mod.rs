//! Repository self-check: find installable apps and suggest signing config.

mod report;
mod walk;

pub use report::{print_human, ScanReport};
pub use walk::scan_repository;
