use std::fs;
use std::path::{Path, PathBuf};

pub mod bin;
pub mod json;

#[derive(Debug, Clone, Copy)]
pub enum DataType {
    Ensemble,
    Simulation,
    Analysis,
}

impl DataType {
    pub fn folder(&self) -> &'static str {
        match self {
            DataType::Ensemble => "ensemble",
            DataType::Simulation => "simulation",
            DataType::Analysis => "analysis",
        }
    }
}

pub trait DataPersistence: serde::Serialize + for<'de> serde::Deserialize<'de> {
    fn data_type() -> DataType;
    fn id(&self) -> usize;
    fn tag(&self) -> usize;

    fn binary_path(&self) -> PathBuf {
        Path::new("./data")
            .join(Self::data_type().folder())
            .join(format!("t{}-i{}.bin", self.tag(), self.id()))
    }

    fn json_path(&self) -> PathBuf {
        Path::new("./plots/data")
            .join(Self::data_type().folder())
            .join(format!("t{}-i{}.json", self.tag(), self.id()))
    }
}

pub fn list_binary_files<T: DataPersistence>() -> Result<Vec<PathBuf>, std::io::Error> {
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

pub fn load_binary_file<T: DataPersistence>(
    file_path: &Path,
) -> Result<T, Box<dyn std::error::Error>> {
    bin::load_binary(file_path)
}
