//! # Ensemble IO Module
//!
//! This module provides input/output functionality for ensemble data persistence and retrieval.
//! It handles the serialization, deserialization, and file management of ensemble collections
//! for the flocking simulation system.
//!
//! ## Overview
//!
//! The ensemble IO module serves as the bridge between in-memory ensemble data structures
//! and persistent storage. It supports:
//! - Binary serialization/deserialization of ensemble data using bincode
//! - Concurrent ensemble saving through receiver threads
//! - Directory discovery and ensemble enumeration
//! - Structured file naming and organization
//!
//! ## File Organization
//!
//! Ensemble files are stored in the `./data/ensemble/` directory with the naming convention:
//! ```text
//! {tag}-{id}.bin
//! ```
//! Where:
//! - `tag`: A string identifier for the ensemble type or experiment
//! - `id`: A unique numeric identifier for the specific ensemble instance
//!
//! ## Integration Points
//!
//! - **Ensemble Module**: Provides `EnsembleResult` data structures for persistence
//! - **IO Module**: Integrates with directory management through `ensure_data_directories()`
//! - **CLI Interface**: Used for batch ensemble generation and analysis workflows
//! - **Simulation Module**: Loads ensembles as initial conditions for simulations
//!
//! ## Error Handling
//!
//! The module uses `Box<dyn std::error::Error>` for error propagation, allowing
//! flexible error handling across different failure modes:
//! - File system errors (directory creation, file access)
//! - Serialization/deserialization errors
//! - Data validation errors
//!
//! Functions may panic on corrupted data during deserialization, which is the
//! expected behavior for data integrity validation.

use std::fs;
use std::path::Path;
use std::sync::mpsc;
use std::thread;
use crate::ensemble::{EnsembleEntryResult};
use crate::io::{get_data_path, save_data, load_data, get_current_timestamp, DataType};



/// Starts a background receiver thread for concurrent ensemble saving
/// 
/// This function spawns a dedicated thread that listens on an MPSC channel for
/// EnsembleResult data and automatically saves each ensemble to disk. It adds
/// timestamp metadata and provides progress feedback through console output.
/// 
/// The receiver thread will run until the channel is closed (all senders dropped).
/// This enables concurrent ensemble generation where multiple worker threads can
/// send completed ensembles for saving without blocking.
///
/// # Arguments
///
/// * `rx` - MPSC receiver channel for EnsembleResult data
///
/// # Returns
///
/// * A join handle for the spawned receiver thread that returns `Result<(), String>`
pub fn start_receiver_thread(
    rx: mpsc::Receiver<EnsembleEntryResult>,
) -> thread::JoinHandle<Result<(), String>> {
    thread::spawn(move || {
        // Ensure ensemble directory exists
        crate::io::ensure_data_directories().map_err(|e| e.to_string())?;

        // Process each ensemble result as it arrives
        while let Ok(ensemble_result) = rx.recv() {
            // Add timestamp info
            let ensemble_with_metadata = EnsembleEntryResult {
                created_at: get_current_timestamp(),
                ..ensemble_result
            };

            // Save to file using the tag
            save_data(
                &ensemble_with_metadata, 
                &get_data_path(DataType::Ensemble, &ensemble_with_metadata.tag, &ensemble_with_metadata.id)
            ).map_err(|e| e.to_string())?;

            println!(
                "Ensemble '{}' (ID: {}) saved successfully with {} birds",
                ensemble_with_metadata.tag,
                ensemble_with_metadata.id,
                ensemble_with_metadata.birds.len()
            );
        }

        Ok(())
    })
}

/// Lists all ensemble files and extracts their tags and IDs
/// 
/// This function scans the `./data/ensemble/` directory for all `.bin` files,
/// parses their filenames to extract tag and ID information, and validates
/// each file by loading it. Only successfully loadable ensembles are included
/// in the results.
/// 
/// The function expects filenames in the format `{tag}-{id}.bin` and will skip
/// any files that don't match this pattern. Files that cannot be deserialized
/// will cause the function to panic (expected behavior for data validation).
///
/// # Returns
/// 
/// * `Ok(Vec<(String, usize)>)` - A vector of tuples containing (tag, id) for each valid ensemble
/// * `Err(Box<dyn std::error::Error>)` - Error if directory cannot be read
/// 
/// # Panics
/// 
/// This function will panic if it encounters corrupted ensemble files that cannot
/// be deserialized. This is the expected behavior for data integrity validation.
pub fn list_ensemble_tags_and_ids() -> Result<Vec<(String, usize)>, Box<dyn std::error::Error>> {
    let ensemble_dir = Path::new("./data/ensemble");
    
    if !ensemble_dir.exists() {
        return Ok(Vec::new());
    }

    let mut results = Vec::new();
    
    for entry in fs::read_dir(ensemble_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        // Skip if not a .bin file
        if !path.extension().map_or(false, |ext| ext == "bin") {
            continue;
        }

        // Extract filename without extension
        let file_name = match path.file_stem().and_then(|name| name.to_str()) {
            Some(name) => name,
            None => continue,
        };

        // Parse filename format: {tag}-{id}
        let dash_pos = match file_name.rfind('-') {
            Some(pos) => pos,
            None => continue,
        };

        let tag = file_name[..dash_pos].to_string();
        let id_str = &file_name[dash_pos + 1..];
        
        let id = match id_str.parse::<usize>() {
            Ok(id) => id,
            Err(_) => continue,
        };

        // Load the ensemble to verify it's valid and get the actual tag and id
        match load_ensemble(&tag, &id) {
            Ok(ensemble) => {
                results.push((ensemble.tag, ensemble.id));
            }
            Err(_) => {
                unreachable!("Failed to load ensemble")
            }
        }
    }
    
    Ok(results)
}

/// Loads ensemble data from a binary file
/// 
/// This function deserializes an EnsembleResult from disk using the standardized
/// file path format. It performs existence checks and handles file IO errors
/// gracefully while allowing deserialization errors to panic (expected behavior
/// for data integrity validation).
///
/// # Arguments
/// 
/// * `tag` - Tag name of the ensemble to load
/// * `id` - ID of the ensemble to load
/// 
/// # Returns
/// 
/// * `Ok(EnsembleResult)` - Successfully loaded and deserialized ensemble data
/// * `Err(Box<dyn std::error::Error>)` - File not found or IO error
/// 
/// # Panics
/// 
/// This function will panic if the file exists but contains corrupted data that
/// cannot be deserialized. This is the expected behavior for data integrity validation.
pub fn load_ensemble(tag: &str, id: &usize) -> Result<EnsembleEntryResult, Box<dyn std::error::Error>> {
    let file_path = get_data_path(DataType::Ensemble, tag, id);
    load_data(&file_path)
}


/// Exports ensemble data to JSON format for Python analysis and visualization
/// 
/// This function converts ensemble data into a JSON format that can be easily
/// imported and processed by Python scripts for analysis and visualization.
/// The exported JSON contains bird initial conditions, metadata, and generation parameters.
/// 
/// # Arguments
/// 
/// * `tag` - The ensemble tag identifier
/// * `id` - The ensemble ID
/// * `output_path` - Path where to save the JSON file
/// 
/// # Returns
/// 
/// * `Ok(())` - Successfully exported the data
/// * `Err(Box<dyn std::error::Error>)` - Error loading ensemble or writing JSON
/// 
/// # JSON Structure
/// 
/// The exported JSON has the following structure:
/// ```json
/// {
///   "metadata": {
///     "ensemble_id": 1,
///     "tag": "experiment_1",
///     "created_at": 1628765432
///   },
///   "parameters": {
///     "num_birds": 100,
///     "radius": 1.0,
///     "speed": 1.0,
///     "min_distance": 0.1
///   },
///   "birds": [
///     {
///       "position": {"x": 1.0, "y": 0.0, "z": 0.0},
///       "velocity": {"x": 0.0, "y": 1.0, "z": 0.0}
///     }
///   ]
/// }
/// ```
/// 
/// # Example Usage
/// 
/// ```rust
/// use flocking_lib::io::ensemble::export_to_json;
/// use std::path::Path;
/// 
/// // Export ensemble data for Python visualization
/// let output_path = Path::new("./data/ensemble/my_ensemble.json");
/// export_to_json("experiment", &1, &output_path);
/// ```
/// 
/// The exported JSON can then be loaded in Python for:
/// - Initial condition visualization
/// - Distribution analysis of bird positions and velocities
/// - Parameter validation and comparison
/// - Setup for simulation batch processing
pub fn export_to_json(tag: &str, id: &usize, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    use serde_json::json;
    
    // Load ensemble data
    let ensemble_result = load_ensemble(tag, id)?;

    // Convert to JSON-friendly format
    let json_data = json!({
        "metadata": {
            "ensemble_id": ensemble_result.id,
            "tag": ensemble_result.tag,
            "created_at": ensemble_result.created_at
        },
        "parameters": {
            "num_birds": ensemble_result.params.n_particles,
            "radius": ensemble_result.params.radius,
            "speed": ensemble_result.params.speed,
            "min_distance": ensemble_result.params.min_distance
        },
        "birds": ensemble_result.birds.iter()
                .map(|bird| {
                    json!({
                        "position": {
                            "x": bird.position.x,
                            "y": bird.position.y,
                            "z": bird.position.z
                        },
                        "velocity": {
                            "x": bird.velocity.x,
                            "y": bird.velocity.y,
                            "z": bird.velocity.z
                        }
                    })
                }).collect::<Vec<_>>()
    });

    // Write JSON to file
    let json_string = serde_json::to_string_pretty(&json_data)?;
    std::fs::write(output_path, json_string)?;

    // Log the file size
    if let Ok(metadata) = std::fs::metadata(output_path) {
        println!("Exported ensemble data to JSON: {} (size: {} bytes)", output_path.display(), metadata.len());
    }

    Ok(())
}

