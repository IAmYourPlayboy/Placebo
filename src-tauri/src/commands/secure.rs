//! Tauri commands for storing small secrets (currently the auth token) in the OS keychain.
//!
//! Uses the platform-native backend via the `keyring` crate:
//!   * Windows  – Credential Manager
//!   * macOS    – Keychain
//!   * Linux    – Secret Service (gnome-keyring / KWallet)
//!
//! Falling back to a file-based store on systems without a keyring is the frontend's job
//! (see `src/auth/tokenStorage.ts`). Here we surface a clean (`Ok(None)` / `Ok(())`) API and
//! convert any `keyring::Error` into a string so the JS side gets a structured-enough payload.

use keyring::Entry;

const SERVICE: &str = "placebo";

fn entry(key: &str) -> Result<Entry, String> {
    Entry::new(SERVICE, key).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn secure_set(key: String, value: String) -> Result<(), String> {
    entry(&key)?
        .set_password(&value)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn secure_get(key: String) -> Result<Option<String>, String> {
    match entry(&key)?.get_password() {
        Ok(v) => Ok(Some(v)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub fn secure_delete(key: String) -> Result<(), String> {
    match entry(&key)?.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}
