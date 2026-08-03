//! Multi-platform ship planning (coverage, CI template, collect, graduate profile).

mod ci_template;
mod collect_dir;
mod coverage;
mod profile;

pub use ci_template::{render_signet_ship_workflow, workflow_rel_path};
pub use collect_dir::{collect_into_staging, staging_release_paths};
pub use coverage::assess_coverage;
pub use profile::{
    assess_sign_profile, discover_graduate_files, PlatformSignAction, ShipSignPath,
};
