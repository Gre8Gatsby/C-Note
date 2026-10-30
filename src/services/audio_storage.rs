use crate::error::Result;
use std::path::{Path, PathBuf};
use tokio::fs;
use uuid::Uuid;

#[derive(Clone)]
pub struct AudioStorage {
    upload_dir: PathBuf,
}

impl AudioStorage {
    pub async fn new(upload_dir_str: &str) -> Result<Self> {
        let upload_dir = PathBuf::from(upload_dir_str);
        if !upload_dir.exists() {
            fs::create_dir_all(&upload_dir).await?;
        }
        Ok(Self { upload_dir })
    }

    pub async fn save_audio(&self, original_filename: Option<&str>, data: &[u8]) -> Result<(String, PathBuf)> {
        let extension = original_filename
            .and_then(|f| Path::new(f).extension())
            .and_then(|ext| ext.to_str())
            .unwrap_or("aac");

        let unique_name = format!("{}.{}", Uuid::new_v4(), extension);
        let target_path = self.upload_dir.join(&unique_name);

        fs::write(&target_path, data).await?;

        Ok((unique_name, target_path))
    }

    pub fn get_file_path(&self, filename: &str) -> PathBuf {
        self.upload_dir.join(filename)
    }

    pub async fn delete_audio(&self, filename: &str) -> Result<()> {
        let path = self.get_file_path(filename);
        if path.exists() {
            fs::remove_file(path).await?;
        }
        Ok(())
    }
}
