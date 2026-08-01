//! Graduation ladder helpers — OV Authenticode, Azure Trusted Signing, Apple notarization.
//!
//! Does not use Signet self-signed identity. See `docs/graduation.md`.

mod apple;
mod notes;
mod windows;

pub use apple::{notarize, staple, NotarizeOptions};
pub use notes::honesty_notes;
pub use windows::{azure_sign_files, ov_sign_files, AzureSignOptions, OvCredential, OvSignOptions};
