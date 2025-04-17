use std::env::join_paths;
use std::fs::File;
use std::io::BufReader;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use anyhow::{anyhow, Error};
use log::debug;
use sqlx::types::JsonValue;
use crate::storage::{FileEntry, FileType, StorageBackend};

pub struct LocalStorage {
    folder_root: PathBuf
}

impl LocalStorage {
    pub(crate) fn new(settings: &JsonValue) -> Result<Self, anyhow::Error> {
        let folder_root = settings["path"].as_str().ok_or_else(|| anyhow::anyhow!("No 'path' configured"))?;
        Ok(LocalStorage {
            folder_root: PathBuf::from(folder_root)
        })
    }
}

fn get_path(folder_root: &PathBuf, library_id: &str, mut path: &Path) -> Result<PathBuf, anyhow::Error> {
    if path.starts_with("/") {
        path = path.strip_prefix("/")?
    }
    let path = folder_root.join(library_id).join(path);
    // Prevent path traversal
    debug!("root={:?}", folder_root);
    debug!("path={:?}", path);
    if !path.starts_with(&folder_root) {
        return Err(anyhow!("Invalid path provided"))
    }
    debug!("{:?}", path);
    Ok(path)
}
impl StorageBackend for LocalStorage {
    fn touch_file(&self, library_id: &str, rel_path: &PathBuf, file_type: FileType) -> Result<(), anyhow::Error> {
        let path = get_path(&self.folder_root, library_id, rel_path)?;
        match file_type {
            FileType::File => {
                // open and close file
                File::open(path).map_err(|e| anyhow!(e))?;
            }
            FileType::Folder => {
                std::fs::create_dir_all(path).map_err(|e| anyhow!(e))?;
            }
            _ => return Err(anyhow!("Unsupported"))
        }
        Ok(())
    }
    fn write_file(&self, library_id: &str, rel_path: &PathBuf, contents: &[u8]) -> Result<(), Error> {
        let path = get_path(&self.folder_root, library_id, rel_path)?;
        std::fs::write(path, contents).map_err(|e| anyhow!(e))
    }

    fn read_file(&self, library_id: &str, rel_path: &PathBuf) -> Result<Option<Vec<u8>>, Error> {
        let path = get_path(&self.folder_root, library_id, rel_path)?;
        match std::fs::read(path) {
            Ok(contents) => Ok(Some(contents)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(anyhow!(e)),
        }
    }
    fn list_files(&self, library_id: &str, rel_path: &PathBuf) -> Result<Vec<FileEntry>, Error> {
        let path = get_path(&self.folder_root, library_id, rel_path)?;
        Ok(std::fs::read_dir(path)?
            .map(|entry| entry.unwrap())
            .map(|entry| {
                let meta = entry.metadata().unwrap();
                let file_type = meta.file_type().into();
                // TODO: filter out 'other'
                FileEntry {
                    _type: file_type,
                    path: entry.file_name().into_string().unwrap(),
                    size: meta.size()
                }
            })
            .collect())
    }

    fn delete_file(&self, library_id: &str, rel_path: &PathBuf) -> Result<(), Error> {
         let path = get_path(&self.folder_root, library_id, rel_path)?;
         // TODO: check if folder?
         std::fs::remove_file(path).map_err(|e| anyhow!(e))
     }

    fn move_file(&self, library_id: &str, rel_path: &PathBuf, new_rel_path: &PathBuf) -> Result<(), Error> {
        let path = get_path(&self.folder_root, library_id, rel_path)?;
        std::fs::rename(path, new_rel_path).map_err(|e| anyhow!(e))
    }

    fn get_read_stream(&self, library_id: &str, rel_path: &PathBuf,) -> Result<BufReader<File>, Error> {
        let path = get_path(&self.folder_root, library_id, rel_path)?;
        let file = File::open(path)?;
        Ok(BufReader::new(file))
    }
}