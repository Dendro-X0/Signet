//! Framework-agnostic artifact contract (Phase 9).

mod adapter;
mod android;
mod capacitor;
mod electron;
mod expo;
mod flutter;
mod ios;
mod kind;
mod react_native;
mod tauri;
mod types;
mod walk_outputs;

pub use adapter::{select_adapter, BuildOpts, FrameworkAdapter};
pub use android::AndroidAdapter;
pub use capacitor::CapacitorAdapter;
pub use electron::ElectronAdapter;
pub use expo::ExpoAdapter;
pub use flutter::FlutterAdapter;
pub use ios::IosAdapter;
pub use kind::ArtifactKind;
pub use react_native::ReactNativeAdapter;
pub use tauri::TauriAdapter;
pub use types::{artifacts_json, host_signable, Artifact};

/// Compatibility alias — same as [`Artifact`].
pub type DiscoveredArtifact = Artifact;

#[allow(dead_code)]
fn _contract_surface() {
    let _ = std::any::type_name::<TauriAdapter>();
    let _ = std::any::type_name::<ElectronAdapter>();
    let _ = std::any::type_name::<AndroidAdapter>();
    let _ = std::any::type_name::<IosAdapter>();
    let _ = std::any::type_name::<FlutterAdapter>();
    let _ = std::any::type_name::<ReactNativeAdapter>();
    let _ = std::any::type_name::<ExpoAdapter>();
    let _ = std::any::type_name::<CapacitorAdapter>();
    let _ = std::any::type_name::<dyn FrameworkAdapter>();
    let _ = artifacts_json(&[]);
}
