//! Multi-platform ship planning (coverage, CI template, collect, graduate profile, secrets).

mod ci_readiness;
mod ci_template;
mod collect_dir;
mod coverage;
mod profile;
mod secrets;

pub use ci_readiness::assess_ci_readiness;
pub use ci_template::{render_signet_ship_workflow, workflow_rel_path};
pub use collect_dir::{collect_into_staging, staging_release_paths};
pub use coverage::assess_coverage;
pub use profile::{
    assess_sign_profile, discover_graduate_files, PlatformSignAction, ShipSignPath,
};
pub use secrets::{run_secrets, SecretsArgs};
