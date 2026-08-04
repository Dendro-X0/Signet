//! Collect release assets and publish to GitHub Releases.

mod auth;
mod collect;
mod github;
mod notes;

pub use auth::{assess_github_auth, offer_open_auth_setup};
pub use collect::{
    collect_release_files_with_opts, verify_checksums_cover_opts, CollectOpts,
};
pub use github::{detect_github_repo, publish_github_release, GitHubPublishOpts};
pub use notes::build_release_notes;
