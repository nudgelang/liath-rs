use std::path::{Path, PathBuf};
use std::fs;
use anyhow::Result;
use uuid::Uuid;

pub struct FileStorage {
    base_path: PathBuf,
}

impl FileStorage {
    pub fn new<P: AsRef<Path>>(base_path: P) -> Result<Self> {
        let base_path = base_path.as_ref().to_path_buf();
        fs::create_dir_all(&base_path)?;
        Ok(Self { base_path })
    }

    pub fn store(&self, content: &[u8]) -> Result<String> {
        let file_id = Uuid::new_v4().to_string();
        let file_path = self.base_path.join(&file_id);
        fs::write(file_path, content)?;
        Ok(file_id)
    }

    pub fn retrieve(&self, file_id: &str) -> Result<Vec<u8>> {
        let file_path = self.base_path.join(file_id);
        Ok(fs::read(file_path)?)
    }

    pub fn delete(&self, file_id: &str) -> Result<()> {
        let file_path = self.base_path.join(file_id);
        fs::remove_file(file_path)?;
        Ok(())
    }
}