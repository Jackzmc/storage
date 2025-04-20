use std::cell::OnceCell;
use std::env;
use std::sync::{LazyLock, OnceLock};
use std::time::Duration;
use rocket::data::ByteUnit;
use rocket::serde::Serialize;

/// The maximum amount of bytes that can be uploaded at once
pub const MAX_UPLOAD_SIZE: ByteUnit = ByteUnit::Mebibyte(100_000);

/// The number of encryption rounds
pub const ENCRYPTION_ROUNDS: u32 = 12;

pub const SESSION_LIFETIME_SECONDS: u64 = 3600 * 24 * 14; // 14 days

pub const SESSION_COOKIE_NAME: &'static str = "storage-session";


#[derive(Serialize)]
pub struct FileConstants<'a> {
    pub display_options: &'a[&'a str],
    pub sort_keys: &'a[&'a str],
}
pub const FILE_CONSTANTS: FileConstants = FileConstants {
    display_options: &["list", "grid"],
    sort_keys: &["name", "last_modified", "size"],
};


/// Disables CSRF & password verification for login
/// Used for development due to no session persistence
pub static DISABLE_LOGIN_CHECK: LazyLock<bool> = LazyLock::new(|| {
    env::var("DANGER_DISABLE_LOGIN_CHECKS").is_ok()
});
pub fn init_statics() {
}