use serde_json::Value;
use sqlx::types::JsonValue;
use crate::storage::{StorageBackend};

pub struct S3Storage {
    // pub fn uri:
    /* uri, bucket id, auth */
}
impl S3Storage {
    fn new(settings: &JsonValue) -> Result<Self, anyhow::Error> {
        todo!()
    }
}
// impl StorageBackend for S3Storage {
//
//
//     fn write_file(&mut self, path: &str, contents: &[u8]) -> Result<(), StorageBackendError> {
//         unimplemented!()
//     }
//     fn read_file(&self, path: &str) -> Result<Vec<u8>, StorageBackendError> {
//         unimplemented!()
//     }
//     fn delete_file(&mut self, path: &str) -> Result<(), StorageBackendError> {
//         unimplemented!()
//     }
// }