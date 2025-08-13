use crate::io::DataPersistence;
use std::path::Path;

pub fn save_binary<T: DataPersistence>(data: &T) -> Result<(), Box<dyn std::error::Error>> {
    let file_path = data.binary_path();
    
    if let Some(parent) = file_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    
    let binary_data = bincode::serialize(data)?;
    std::fs::write(&file_path, binary_data)?;
    
    Ok(())
}

pub fn load_binary<T: DataPersistence>(file_path: &Path) -> Result<T, Box<dyn std::error::Error>> {
    let binary_data = std::fs::read(file_path)?;
    let data: T = bincode::deserialize(&binary_data)?;
    Ok(data)
}
