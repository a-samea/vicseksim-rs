use crate::io::DataPersistence;
use std::fs;
use std::path::{Path, PathBuf};

pub fn save_file<T: DataPersistence>(data: &T) -> Result<(), Box<dyn std::error::Error>> {
    let file_path = data.binary_path();

    if let Some(parent) = file_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let binary_data = bincode::serialize(data)?;
    std::fs::write(&file_path, binary_data)?;

    Ok(())
}

pub fn load_file<T: DataPersistence>(file_path: &Path) -> Result<T, Box<dyn std::error::Error>> {
    let binary_data = std::fs::read(file_path)?;
    let data: T = bincode::deserialize(&binary_data)?;
    Ok(data)
}

pub fn list_files<T: DataPersistence>() -> Result<Vec<PathBuf>, std::io::Error> {
    let dir_path = Path::new("./data").join(T::data_type().folder());

    if !dir_path.exists() {
        return Ok(Vec::new());
    }

    let mut files = Vec::new();
    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("bin") {
            files.push(path);
        }
    }
    files.sort();
    Ok(files)
}
