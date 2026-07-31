//! Local signing identity storage under `.signet/identity/`.

mod fingerprint;
pub mod store;

pub use store::{
    create_identity, import_identity, list_identities, load_active, load_named, read_active,
    set_active, CreateOptions, IdentityRecord, ImportOptions,
};
