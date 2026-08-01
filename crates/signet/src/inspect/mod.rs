//! Best-effort host signature inspection (signed / unsigned / unknown).

mod probe;
mod report;

pub use probe::inspect_path;
pub use report::{InspectReport, SignatureStatus};
