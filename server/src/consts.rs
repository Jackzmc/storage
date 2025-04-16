use std::time::Duration;
use rocket::data::ByteUnit;

/// The maximum amount of bytes that can be uploaded at once
pub const MAX_UPLOAD_SIZE: ByteUnit = ByteUnit::Mebibyte(100_000);

/// The number of encryption rounds
pub const ENCRYPTION_ROUNDS: u32 = 20;

pub const SESSION_LIFETIME_SECONDS: u64 = 3600 * 24 * 14; // 14 days

pub const SESSION_COOKIE_NAME: &'static str = "storage-session";