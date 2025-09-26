#[cfg(target_arch = "wasm32")]
use web_sys::{Storage, window};

#[cfg(target_arch = "wasm32")]
pub fn get_local_storage() -> Option<Storage> {
    window()?.local_storage().ok()?
}
