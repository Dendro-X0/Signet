//! Multi-platform ship planning (coverage, CI template, collect).

mod ci_template;
mod collect_dir;
mod coverage;

pub use ci_template::{render_signet_ship_workflow, workflow_rel_path};
pub use collect_dir::{collect_into_staging, staging_release_paths};
pub use coverage::assess_coverage;
