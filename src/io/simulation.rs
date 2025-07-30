//! # Simulation IO Module - Simulation Data Persistence
//!
//! This module handles saving and loading of simulation frame data and run metadata.
//! Simulation data is saved as binary files using serde serialization in the `./data/simulation/` directory.
//!
//! ## File Format
//!
//! - **Location**: `./data/simulation/[tag]/`
//! - **Frames**: `frames.bin` - Binary serialized frame data
//! - **Metadata**: `metadata.bin` - Binary serialized run metadata
//!
//! ## Usage
//!
//! ```rust
//! use std::sync::mpsc;
//! use flocking_lib::io::simulation;
//!
//! // Setup channel to receive frame data from simulation
//! let (tx, rx) = mpsc::channel();
//!
//! // Start simulation data receiver
//! simulation::start_receiver(rx, "my_simulation".to_string(), run_metadata);
//! ```

use crate::simulation::{FrameData, SimulationParams};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::{SystemTime, UNIX_EPOCH};

/// Complete simulation run data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationRun {
    /// Metadata about the simulation run
    pub metadata: SimulationMetadata,
    /// All frame data from the simulation
    pub frames: Vec<FrameData>,
}

/// Metadata associated with each simulation run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationMetadata {
    /// User-defined tag for the simulation run
    pub tag: String,
    /// Tag of the ensemble used as initial conditions
    pub ensemble_tag: String,
    /// Simulation parameters used
    pub params: SimulationParams,
    /// Total number of frames saved
    pub total_frames: usize,
    /// Frame saving interval (every N steps)
    pub frame_interval: u64,
    /// Timestamp when simulation was started
    pub started_at: u64,
    /// Timestamp when simulation was completed
    pub completed_at: Option<u64>,
    /// File format version for compatibility
    pub version: String,
    /// Whether the simulation completed successfully
    pub completed_successfully: bool,
}

/// Status of a simulation run
#[derive(Debug, Clone)]
pub struct SimulationStatus {
    /// Directory path
    pub path: PathBuf,
    /// Whether the run is valid and loadable
    pub is_valid: bool,
    /// Error message if invalid
    pub error: Option<String>,
    /// Metadata if valid
    pub metadata: Option<SimulationMetadata>,
    /// Number of frames available
    pub frame_count: usize,
}

/// Starts a receiver thread that listens for frame data from MPSC channel
/// and saves it to disk with the specified tag
///
/// # Arguments
///
/// * `rx` - MPSC receiver channel for frame data
/// * `tag` - Tag name for the simulation run
/// * `ensemble_tag` - Tag of the ensemble used as initial conditions
/// * `params` - Simulation parameters
/// * `frame_interval` - Frame saving interval
pub fn start_receiver(
    rx: mpsc::Receiver<FrameData>,
    tag: String,
    ensemble_tag: String,
    params: SimulationParams,
    frame_interval: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    // Ensure simulation directory exists
    crate::io::ensure_data_directories()?;

    let run_dir = get_simulation_run_path(&tag);
    fs::create_dir_all(&run_dir)?;

    // Create initial metadata
    let mut metadata = SimulationMetadata {
        tag: tag.clone(),
        ensemble_tag,
        params,
        total_frames: 0,
        frame_interval,
        started_at: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
        completed_at: None,
        version: env!("CARGO_PKG_VERSION").to_string(),
        completed_successfully: false,
    };

    // Save initial metadata
    save_metadata(&metadata, &tag)?;

    let mut frames = Vec::new();

    // Receive and collect frame data
    while let Ok(frame) = rx.recv() {
        frames.push(frame);
        metadata.total_frames = frames.len();

        // Periodically save progress
        if frames.len() % 100 == 0 {
            save_frames(&frames, &tag)?;
            save_metadata(&metadata, &tag)?;
            println!("Saved {} frames for simulation '{}'", frames.len(), tag);
        }
    }

    // Mark as completed and save final data
    metadata.completed_at = Some(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs());
    metadata.completed_successfully = true;

    save_frames(&frames, &tag)?;
    save_metadata(&metadata, &tag)?;

    println!(
        "Simulation '{}' completed successfully with {} frames",
        tag,
        frames.len()
    );

    Ok(())
}

/// Saves simulation metadata to disk
///
/// # Arguments
///
/// * `metadata` - The metadata to save
/// * `tag` - Tag name for the simulation run
pub fn save_metadata(
    metadata: &SimulationMetadata,
    tag: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let run_dir = get_simulation_run_path(tag);
    fs::create_dir_all(&run_dir)?;

    let metadata_path = run_dir.join("metadata.bin");
    let file = File::create(&metadata_path)?;
    let writer = BufWriter::new(file);

    bincode::serialize_into(writer, metadata)?;

    Ok(())
}

/// Saves frame data to disk
///
/// # Arguments
///
/// * `frames` - The frame data to save
/// * `tag` - Tag name for the simulation run
pub fn save_frames(frames: &[FrameData], tag: &str) -> Result<(), Box<dyn std::error::Error>> {
    let run_dir = get_simulation_run_path(tag);
    fs::create_dir_all(&run_dir)?;

    let frames_path = run_dir.join("frames.bin");
    let file = File::create(&frames_path)?;
    let writer = BufWriter::new(file);

    bincode::serialize_into(writer, frames)?;

    Ok(())
}

/// Loads complete simulation run data
///
/// # Arguments
///
/// * `tag` - Tag name of the simulation run to load
pub fn load_simulation_run(tag: &str) -> Result<SimulationRun, Box<dyn std::error::Error>> {
    let metadata = load_metadata(tag)?;
    let frames = load_frames(tag)?;

    Ok(SimulationRun { metadata, frames })
}

/// Loads only simulation metadata (without frame data)
///
/// # Arguments
///
/// * `tag` - Tag name of the simulation run
pub fn load_metadata(tag: &str) -> Result<SimulationMetadata, Box<dyn std::error::Error>> {
    let run_dir = get_simulation_run_path(tag);
    let metadata_path = run_dir.join("metadata.bin");

    if !metadata_path.exists() {
        return Err(format!("Simulation metadata not found: {}", metadata_path.display()).into());
    }

    let file = File::open(&metadata_path)?;
    let reader = BufReader::new(file);

    let metadata: SimulationMetadata = bincode::deserialize_from(reader)?;

    Ok(metadata)
}

/// Loads frame data from a simulation run
///
/// # Arguments
///
/// * `tag` - Tag name of the simulation run
pub fn load_frames(tag: &str) -> Result<Vec<FrameData>, Box<dyn std::error::Error>> {
    let run_dir = get_simulation_run_path(tag);
    let frames_path = run_dir.join("frames.bin");

    if !frames_path.exists() {
        return Err(format!("Simulation frames not found: {}", frames_path.display()).into());
    }

    let file = File::open(&frames_path)?;
    let reader = BufReader::new(file);

    let frames: Vec<FrameData> = bincode::deserialize_from(reader)?;

    Ok(frames)
}

/// Loads a specific range of frames from a simulation run
///
/// # Arguments
///
/// * `tag` - Tag name of the simulation run
/// * `start_frame` - Starting frame index (inclusive)
/// * `end_frame` - Ending frame index (exclusive)
pub fn load_frame_range(
    tag: &str,
    start_frame: usize,
    end_frame: usize,
) -> Result<Vec<FrameData>, Box<dyn std::error::Error>> {
    let frames = load_frames(tag)?;

    if start_frame >= frames.len() {
        return Err("Start frame index out of bounds".into());
    }

    let end_idx = std::cmp::min(end_frame, frames.len());
    Ok(frames[start_frame..end_idx].to_vec())
}

/// Enumerates all simulation runs and verifies their status
pub fn enumerate_simulation_runs() -> Result<Vec<SimulationStatus>, Box<dyn std::error::Error>> {
    let simulation_dir = Path::new("./data/simulation");

    if !simulation_dir.exists() {
        return Ok(Vec::new());
    }

    let mut statuses = Vec::new();

    for entry in fs::read_dir(simulation_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            let status = verify_simulation_run(&path);
            statuses.push(status);
        }
    }

    // Sort by start time (newest first)
    statuses.sort_by(|a, b| match (&a.metadata, &b.metadata) {
        (Some(meta_a), Some(meta_b)) => meta_b.started_at.cmp(&meta_a.started_at),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => a.path.cmp(&b.path),
    });

    Ok(statuses)
}

/// Verifies the status of a simulation run directory
fn verify_simulation_run(path: &Path) -> SimulationStatus {
    let tag = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("unknown")
        .to_string();

    match load_metadata(&tag) {
        Ok(metadata) => {
            // Try to count frames
            let frame_count = match load_frames(&tag) {
                Ok(frames) => frames.len(),
                Err(_) => 0,
            };

            SimulationStatus {
                path: path.to_path_buf(),
                is_valid: true,
                error: None,
                metadata: Some(metadata),
                frame_count,
            }
        }
        Err(e) => SimulationStatus {
            path: path.to_path_buf(),
            is_valid: false,
            error: Some(e.to_string()),
            metadata: None,
            frame_count: 0,
        },
    }
}

/// Gets the directory path for a simulation run with the given tag
fn get_simulation_run_path(tag: &str) -> PathBuf {
    Path::new("./data/simulation").join(tag)
}

/// Lists all available simulation run tags
pub fn list_simulation_tags() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let statuses = enumerate_simulation_runs()?;
    let tags: Vec<String> = statuses
        .into_iter()
        .filter_map(|status| {
            if status.is_valid {
                status.metadata.map(|meta| meta.tag)
            } else {
                None
            }
        })
        .collect();

    Ok(tags)
}
