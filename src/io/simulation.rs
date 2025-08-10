//! # Simulation IO Module
//!
//! This module provides input/output functionality for simulation data persistence and retrieval.
//! It handles the serialization, deserialization, and file management of simulation results
//! for the flocking simulation system.
//!
//! ## Overview
//!
//! The simulation IO module serves as the bridge between in-memory simulation data structures
//! and persistent storage. It supports:
//! - Binary serialization/deserialization of simulation data using bincode
//! - Concurrent simulation saving through receiver threads using MPSC channels
//! - Directory discovery and simulation enumeration
//! - Structured file naming and organization
//! - Frame collection from simulation snapshots
//!
//! ## File Organization
//!
//! Simulation files are stored in the `./data/simulation/` directory with the naming convention:
//! ```text
//! {tag}-{id}.bin
//! ```
//! Where:
//! - `tag`: A string identifier for the simulation type or experiment
//! - `id`: A unique numeric identifier for the specific simulation instance
//!
//! ## Integration Points
//!
//! - **Simulation Module**: Provides `SimulationSnapshot` and `SimulationResult` data structures
//! - **IO Module**: Integrates with directory management through `ensure_data_directories()`
//! - **CLI Interface**: Used for batch simulation execution and analysis workflows
//! - **Analysis Module**: Loads simulation results for post-processing analysis
//!
//! ## MPSC Integration
//!
//! The module receives `SimulationSnapshot` data through MPSC channels from simulation threads.
//! Each simulation run spawns a receiver thread that collects snapshots and builds a complete
//! `SimulationResult` for persistence.
//!
//! ### Example Usage
//!
//! ```rust,no_run
//! use std::sync::mpsc;
//! use std::collections::HashMap;
//! use flocking_lib::simulation::{SimulationSnapshot, SimulationParams};
//! use flocking_lib::io::simulation::{FrameCollector, start_receiver_thread};
//!
//! // Set up MPSC channel for frame collection
//! let (tx, rx) = mpsc::channel();
//!
//! // Prepare frame collectors for multiple simulations
//! let mut collectors = HashMap::new();
//! let params = SimulationParams {
//!     num_birds: 100,
//!     radius: 1.0,
//!     speed: 1.0,
//!     dt: 0.01,
//!     interaction_radius: 0.5,
//!     eta: 0.1,
//!     iterations: 1000,
//! };
//!
//! collectors.insert(1, FrameCollector::new(
//!     1,                    // simulation id
//!     "test".to_string(),   // tag
//!     42,                   // ensemble id
//!     params,
//! ));
//!
//! // Start the IO receiver thread
//! let io_handle = start_receiver_thread(rx, collectors);
//!
//! // Send simulation snapshots (this would be done by the simulation engine)
//! // tx.send((1, snapshot)).unwrap();
//!
//! // When all snapshots are sent, close the channel and wait for completion
//! drop(tx);
//! io_handle.join().unwrap().unwrap();
//! ```
//!
//! ### Alternative Dynamic Usage
//!
//! For more flexibility, use the dynamic receiver that handles simulation initialization:
//!
//! ```rust,no_run
//! use std::sync::mpsc;
//! use flocking_lib::simulation::{SimulationParams};
//! use flocking_lib::io::simulation::{SimulationMessage, start_dynamic_receiver_thread};
//!
//! let (tx, rx) = mpsc::channel();
//! let io_handle = start_dynamic_receiver_thread(rx);
//!
//! // Initialize a new simulation
//! tx.send(SimulationMessage::Init {
//!     id: 1,
//!     tag: "experiment_1".to_string(),
//!     ensemble_id: 42,
//!     params: SimulationParams {
//!         num_birds: 100,
//!         radius: 1.0,
//!         speed: 1.0,
//!         dt: 0.01,
//!         interaction_radius: 0.5,
//!         eta: 0.1,
//!         iterations: 1000,
//!     },
//! }).unwrap();
//!
//! // Send snapshots as they are generated
//! // tx.send(SimulationMessage::Snapshot { simulation_id: 1, snapshot }).unwrap();
//!
//! // Finalize the simulation when complete
//! tx.send(SimulationMessage::Finalize { simulation_id: 1 }).unwrap();
//!
//! drop(tx);
//! io_handle.join().unwrap().unwrap();
//! ```

use std::fs;
use std::path::{Path};
use std::sync::mpsc;
use std::thread;
use std::collections::HashMap;
use crate::simulation::{SimulationSnapshot, SimulationResult, SimulationParams};
use crate::io::{get_data_path, save_data, load_data, get_current_timestamp, DataType};


/// Frame collector for building simulation results from individual snapshots
/// 
/// This structure accumulates `SimulationSnapshot` frames received through MPSC
/// channels and builds a complete `SimulationResult` when the simulation finishes.
#[derive(Debug)]
pub struct FrameCollector {
    /// Simulation metadata and parameters
    pub id: usize,
    pub tag: String,
    pub ensemble_id: usize,
    pub params: SimulationParams,
    
    /// Collected simulation frames
    pub snapshots: Vec<SimulationSnapshot>,
    
    /// Metadata
    pub total_steps: u64,
}

impl FrameCollector {
    /// Creates a new frame collector for a simulation
    /// 
    /// # Arguments
    /// 
    /// * `id` - Unique simulation identifier
    /// * `tag` - Simulation tag for grouping
    /// * `ensemble_id` - Associated ensemble identifier
    /// * `params` - Simulation parameters
    /// 
    /// # Returns
    /// 
    /// A new `FrameCollector` instance ready to receive snapshots
    pub fn new(id: usize, tag: String, ensemble_id: usize, params: SimulationParams) -> Self {
        Self {
            id,
            tag,
            ensemble_id,
            params,
            snapshots: Vec::new(),
            total_steps: 0,
        }
    }
    
    /// Adds a simulation snapshot to the collection
    /// 
    /// # Arguments
    /// 
    /// * `snapshot` - The simulation snapshot to add
    pub fn add_snapshot(&mut self, snapshot: SimulationSnapshot) {
        self.total_steps = self.total_steps.max(snapshot.step);
        self.snapshots.push(snapshot);
    }
    
    /// Finalizes the collection and creates a complete simulation result
    /// 
    /// # Returns
    /// 
    /// A complete `SimulationResult` ready for persistence
    pub fn finalize(self) -> SimulationResult {
        let duration_seconds = self.total_steps as f64 * self.params.dt;
        let final_state = self.snapshots
            .last()
            .map(|snapshot| snapshot.birds.clone())
            .unwrap_or_default();
            
        SimulationResult {
            id: self.id,
            tag: self.tag,
            ensemble_id: self.ensemble_id,
            params: self.params,
            snapshots: self.snapshots,
            final_state,
            created_at: get_current_timestamp(),
            total_steps: self.total_steps,
            duration_seconds,
        }
    }
}

/// Starts a background receiver thread for concurrent simulation frame collection and saving
/// 
/// This function spawns a dedicated thread that listens on an MPSC channel for
/// SimulationSnapshot data, collects them into complete simulation results, and 
/// automatically saves each completed simulation to disk. It provides progress 
/// feedback through console output.
/// 
/// The receiver thread will run until the channel is closed (all senders dropped).
/// This enables concurrent simulation execution where multiple worker threads can
/// send snapshots for collection without blocking.
/// 
/// # Arguments
///
/// * `rx` - MPSC receiver channel for SimulationSnapshot data
/// * `collectors` - HashMap mapping simulation IDs to their frame collectors
///
/// # Returns
///
/// * A join handle for the spawned receiver thread that returns `Result<(), String>`
/// 
/// # Note
/// 
/// This function expects that simulation metadata (id, tag, ensemble_id, params) 
/// is somehow communicated alongside the snapshots. For simplicity, this implementation
/// assumes the collectors are pre-configured. In practice, you might want to send
/// a setup message first or include metadata in each snapshot.
pub fn start_receiver_thread(
    rx: mpsc::Receiver<(usize, SimulationSnapshot)>, // (simulation_id, snapshot)
    mut collectors: HashMap<usize, FrameCollector>,
) -> thread::JoinHandle<Result<(), String>> {
    thread::spawn(move || {
        // Ensure simulation directory exists
        crate::io::ensure_data_directories().map_err(|e| e.to_string())?;

        // Process each simulation snapshot as it arrives
        while let Ok((simulation_id, snapshot)) = rx.recv() {
            if let Some(collector) = collectors.get_mut(&simulation_id) {
                collector.add_snapshot(snapshot);
            } else {
                eprintln!("Warning: Received snapshot for unknown simulation ID: {}", simulation_id);
            }
        }

        // Finalize and save all collected simulations
        for (_simulation_id, collector) in collectors {
            let simulation_result = collector.finalize();
            
            // Save to file
            save_data(
                &simulation_result, 
                &get_data_path(DataType::Simulation, &simulation_result.tag,&simulation_result.id)
                ).map_err(|e| e.to_string())?;

            println!(
                "Simulation '{}' (ID: {}) saved successfully with {} snapshots ({} steps, {:.2}s)",
                simulation_result.tag,
                simulation_result.id,
                simulation_result.snapshots.len(),
                simulation_result.total_steps,
                simulation_result.duration_seconds
            );
        }

        Ok(())
    })
}

/// Alternative receiver thread that creates collectors dynamically
/// 
/// This version is more flexible as it doesn't require pre-configured collectors.
/// Instead, it expects the first message for each simulation to contain metadata.
/// 
/// # Arguments
/// 
/// * `rx` - MPSC receiver for either metadata or snapshot messages
/// 
/// # Returns
/// 
/// A join handle for the spawned receiver thread
pub fn start_dynamic_receiver_thread(
    rx: mpsc::Receiver<SimulationMessage>,
) -> thread::JoinHandle<Result<(), String>> {
    thread::spawn(move || {
        // Ensure simulation directory exists
        crate::io::ensure_data_directories().map_err(|e| e.to_string())?;

        let mut collectors: HashMap<usize, FrameCollector> = HashMap::new();

        // Process each message as it arrives
        while let Ok(message) = rx.recv() {
            match message {
                SimulationMessage::Init { id, tag, ensemble_id, params } => {
                    let collector = FrameCollector::new(id, tag, ensemble_id, params);
                    collectors.insert(id, collector);
                }
                SimulationMessage::Snapshot { simulation_id, snapshot } => {
                    if let Some(collector) = collectors.get_mut(&simulation_id) {
                        collector.add_snapshot(snapshot);
                    } else {
                        eprintln!("Warning: Received snapshot for uninitialized simulation ID: {}", simulation_id);
                    }
                }
                SimulationMessage::Finalize { simulation_id } => {
                    if let Some(collector) = collectors.remove(&simulation_id) {
                        let simulation_result = collector.finalize();
                        
                        // Save to file
                        save_data(
                            &simulation_result,
                            &get_data_path(DataType::Simulation, &simulation_result.tag, &simulation_result.id)
                        ).map_err(|e| e.to_string())?;

                        println!(
                            "Simulation '{}' (ID: {}) saved successfully with {} snapshots ({} steps, {:.2}s)",
                            simulation_result.tag,
                            simulation_result.id,
                            simulation_result.snapshots.len(),
                            simulation_result.total_steps,
                            simulation_result.duration_seconds
                        );
                    }
                }
            }
        }

        // Finalize any remaining simulations
        for (_simulation_id, collector) in collectors {
            let simulation_result = collector.finalize();
            save_data(
                &simulation_result,
                &get_data_path(DataType::Simulation, &simulation_result.tag, &simulation_result.id)
            ).map_err(|e| e.to_string())?;
            
            println!(
                "Simulation '{}' (ID: {}) auto-finalized with {} snapshots",
                simulation_result.tag,
                simulation_result.id,
                simulation_result.snapshots.len()
            );
        }

        Ok(())
    })
}

/// Message types for the dynamic receiver thread
#[derive(Debug, Clone)]
pub enum SimulationMessage {
    /// Initialize a new simulation collector
    Init {
        id: usize,
        tag: String,
        ensemble_id: usize,
        params: SimulationParams,
    },
    /// Add a snapshot to an existing simulation
    Snapshot {
        simulation_id: usize,
        snapshot: SimulationSnapshot,
    },
    /// Finalize and save a simulation
    Finalize {
        simulation_id: usize,
    },
}

/// Lists all simulation files and extracts their tags and IDs
/// 
/// This function scans the `./data/simulation/` directory for all `.bin` files,
/// parses their filenames to extract tag and ID information, and validates
/// each file by loading it. Only successfully loadable simulations are included
/// in the results.
/// 
/// The function expects filenames in the format `{tag}-{id}.bin` and will skip
/// any files that don't match this pattern. Files that cannot be deserialized
/// will cause the function to panic (expected behavior for data validation).
///
/// # Returns
/// 
/// * `Ok(Vec<(String, usize)>)` - A vector of tuples containing (tag, id) for each valid simulation
/// * `Err(Box<dyn std::error::Error>)` - Error if directory cannot be read
/// 
/// # Panics
/// 
/// This function will panic if it encounters corrupted simulation files that cannot
/// be deserialized. This is the expected behavior for data integrity validation.
pub fn list_simulation_tags_and_ids() -> Result<Vec<(String, usize)>, Box<dyn std::error::Error>> {
    let simulation_dir = Path::new("./data/simulation");
    
    if !simulation_dir.exists() {
        return Ok(Vec::new());
    }

    let mut results = Vec::new();
    
    for entry in fs::read_dir(simulation_dir)? {
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

        // Load the simulation to verify it's valid and get the actual tag and id
        match load_simulation(&tag, &id) {
            Ok(simulation) => {
                results.push((simulation.tag, simulation.id));
            }
            Err(_) => {
                unreachable!("Failed to load simulation")
            }
        }
    }
    
    Ok(results)
}

/// Loads simulation data from a binary file
/// 
/// This function deserializes a SimulationResult from disk using the standardized
/// file path format. It performs existence checks and handles file IO errors
/// gracefully while allowing deserialization errors to panic (expected behavior
/// for data integrity validation).
///
/// # Arguments
/// 
/// * `tag` - Tag name of the simulation to load
/// * `id` - ID of the simulation to load
/// 
/// # Returns
/// 
/// * `Ok(SimulationResult)` - Successfully loaded and deserialized simulation data
/// * `Err(Box<dyn std::error::Error>)` - File not found or IO error
/// 
/// # Panics
/// 
/// This function will panic if the file exists but contains corrupted data that
/// cannot be deserialized. This is the expected behavior for data integrity validation.
pub fn load_simulation(tag: &str, id: &usize) -> Result<SimulationResult, Box<dyn std::error::Error>> {
    let file_path = get_data_path(DataType::Simulation, tag, id);
    load_data(&file_path)
}

/// Lists all available simulation results by tag
/// 
/// Groups simulations by their tag for easy discovery and batch processing.
/// 
/// # Returns
/// 
/// * `Ok(HashMap<String, Vec<usize>>)` - Map from tag to list of simulation IDs
/// * `Err(Box<dyn std::error::Error>)` - Error if directory cannot be read
pub fn list_simulations_by_tag() -> Result<std::collections::HashMap<String, Vec<usize>>, Box<dyn std::error::Error>> {
    let tag_id_pairs = list_simulation_tags_and_ids()?;
    let mut results = std::collections::HashMap::new();
    
    for (tag, id) in tag_id_pairs {
        results.entry(tag).or_insert_with(Vec::new).push(id);
    }
    
    Ok(results)
}
