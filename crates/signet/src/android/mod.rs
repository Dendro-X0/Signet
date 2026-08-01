//! Android keystore + APK signing helpers (Phase 11).

mod keystore;
mod sign;
mod tools;

pub use keystore::{
    create_keystore, import_keystore, keystore_paths, load_meta, read_cert_sha256, store_pass,
};
pub use sign::sign_apks;
pub use tools::{find_apksigner, find_keytool};

#[allow(dead_code)]
fn _android_surface() {
    let _ = std::any::type_name::<keystore::AndroidKeyMeta>();
    let _ = std::any::type_name::<keystore::AndroidKeyPaths>();
    let _ = std::any::type_name::<sign::AndroidSignReport>();
    let _ = keystore::key_pass;
    let _ = sign::sign_apk;
    let _ = tools::find_jarsigner;
}
