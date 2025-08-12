use crate::io::Persistable;
use std::path::Path;

pub struct BinIO;

impl BinIO {
    /// Save any persistable data to disk
    pub fn save<T: Persistable>(data: &T) -> Result<(), Box<dyn std::error::Error>> {
        let file_path = data.file_path();

        // Create parent directory if needed
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let json_data = serde_json::to_string_pretty(data)?;
        std::fs::write(&file_path, json_data)?;

        // Log successful save
        if let Ok(metadata) = std::fs::metadata(&file_path) {
            println!(
                "Saved {}: {} (size: {} bytes)",
                data.data_type().as_str(),
                file_path.display(),
                metadata.len()
            );
        }

        Ok(())
    }

    /// Load any persistable data from disk
    pub fn load<T: Persistable>(file_path: &Path) -> Result<T, Box<dyn std::error::Error>> {
        if !file_path.exists() {
            return Err(format!("File not found: {}", file_path.display()).into());
        }

        let json_data = std::fs::read_to_string(file_path)?;
        let data: T = serde_json::from_str(&json_data)?;

        Ok(data)
    }
}
