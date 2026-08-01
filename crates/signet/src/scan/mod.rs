//! Repository self-check: find installable apps and suggest signing config.

mod report;
mod walk;

pub use report::{
    framework_id_for_kind, preferred_project, print_human, DetectedProject, ProjectKind, ScanReport,
};
pub use walk::scan_repository;

// Keep `ProjectKind` in the scan surface for TUI/tests (`crate::scan::ProjectKind`).
const _: ProjectKind = ProjectKind::Tauri;
