//! Framework-agnostic artifact contract (Phase 9).

mod adapter;
mod kind;
mod tauri;
mod types;

pub use adapter::{select_adapter, BuildOpts, FrameworkAdapter};
pub use kind::ArtifactKind;
pub use tauri::TauriAdapter;
pub use types::{artifacts_json, host_signable, Artifact};

/// Compatibility alias — same as [`Artifact`].
pub type DiscoveredArtifact = Artifact;

// Ensure contract exports stay linked in the binary (Phase 9 surface).
#[allow(dead_code)]
fn _contract_surface() {
    let _ = std::any::type_name::<TauriAdapter>();
    let _ = std::any::type_name::<dyn FrameworkAdapter>();
    let _ = artifacts_json(&[]);
}
