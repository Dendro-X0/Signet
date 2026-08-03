//! Repository self-check: find installable apps and suggest signing config.

mod report;
mod walk;

pub use report::{
    draft_targets, framework_id_for_kind, merge_platforms, preferred_project, print_human,
    DetectedProject, Platform, ProjectKind, ScanReport,
};
pub use walk::scan_repository;

// Keep `ProjectKind` in the scan surface for TUI/tests (`crate::scan::ProjectKind`).
const _: ProjectKind = ProjectKind::Tauri;
