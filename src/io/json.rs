use crate::io::DataPersistence;

pub fn export_json<T: DataPersistence>(data: &T) -> Result<(), Box<dyn std::error::Error>> {
    let file_path = data.json_path();
    
    if let Some(parent) = file_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    
    let json_data = serde_json::to_string_pretty(data)?;
    std::fs::write(&file_path, json_data)?;
    
    Ok(())
}