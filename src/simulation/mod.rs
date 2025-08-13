//! # Simulation Module - High-Performance Flocking Engine
//!
//! create!

pub mod io;
pub mod logic;
pub mod tests;

use crate::bird::Bird;
use log::debug;
use std::sync::mpsc;

/// Comprehensive configuration parameters for flocking simulation physics and behavior.
///
/// This structure encapsulates all the essential parameters that control the simulation
/// dynamics, from basic system size to complex interaction behaviors. These parameters
/// directly influence the emergent flocking patterns and computational performance.
#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
pub struct SimulationParams {
    /// Total number of birds in the simulation system.
    pub num_birds: usize,
    /// Radius of the spherical surface constraining all particle motion.
    pub radius: f64,
    /// Uniform speed magnitude for all birds in the simulation.
    pub speed: f64,
    /// Time step size for numerical integration of equations of motion.
    pub dt: f64,
    /// Maximum distance for bird-to-bird interaction detection.
    pub interaction_radius: f64,
    /// Noise parameter controlling random perturbations in bird behavior.
    pub eta: f64,
    /// Maximum number of simulation steps to execute.
    pub total_iterations: usize,
    /// Interval controlling snapshot capture frequency.
    pub frame_interval: usize,
}

/// Simulation execution request containing initial conditions and configuration.
///
/// This structure packages all necessary information to initialize and run a complete
/// flocking simulation. It serves as the primary interface for external systems to
/// specify simulation parameters, initial conditions, and tracking metadata.
#[derive(Debug, Clone)]
pub struct SimulationRequest {
    /// Unique identifier for this simulation run.
    pub id: usize,
    /// Human-readable tag for grouping related simulation runs.
    pub tag: usize,
    /// Identifier linking this simulation to a specific ensemble study.
    pub ensemble_entry_id: usize,
    /// Initial spatial and velocity configuration for all birds.
    pub initial_values: Vec<Bird>,
    /// Complete physics and execution parameters for the simulation.
    pub params: SimulationParams,
}

/// Temporal snapshot of simulation state for analysis and visualization.
///
/// Captures the complete system state at a specific simulation time, providing
/// a foundation for trajectory analysis, pattern recognition, and visualization
/// of flocking dynamics. Snapshots are generated at configurable intervals to
/// balance data richness with storage efficiency.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SimulationSnapshot {
    /// Simulation step number when this snapshot was captured.
    pub step: usize,
    /// Continuous simulation time corresponding to this snapshot.
    pub timestamp: f64,
    /// Complete state vector of all birds at this temporal moment.
    pub birds: Vec<Bird>,
}

/// Complete simulation execution results with metadata.
///
/// This comprehensive result structure contains all data necessary for post-simulation
/// analysis, including trajectory data, configuration parameters, execution metadata,
/// and performance statistics. It serves as the primary output format for simulation
/// studies and enables reproducible research workflows.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SimulationResult {
    /// Unique identifier matching the original simulation request.
    pub id: usize,
    /// Descriptive tag inherited from the simulation request.
    pub tag: usize,
    /// Ensemble identifier linking this result to related simulation runs.
    pub ensemble_entry_id: usize,
    /// Complete simulation configuration used for this execution.
    pub params: SimulationParams,
    /// Time-ordered sequence of simulation state snapshots.
    pub snapshots: Vec<SimulationSnapshot>,
}

/// High-performance flocking simulation engine with parallel processing and memory optimization.
///

pub struct Engine {
    /// Primary particle state buffer containing current simulation state.
    particles_a: Vec<Bird>,
    /// Secondary particle state buffer for writing computed updates.
    particles_b: Vec<Bird>,
    /// Immutable simulation configuration controlling physics and behavior.
    params: SimulationParams,
    /// Current discrete simulation step counter.
    step_count: usize,
    /// Continuous simulation time in physical units.
    current_timestamp: f64,
    /// Asynchronous channel for transmitting frame data to external consumers.
    frame_sender: mpsc::Sender<SimulationSnapshot>,
}

pub fn run(request: SimulationRequest) -> Result<(), String> {
    debug!(
        "Starting simulation run: id={}, tag={}, ensemble_entry_id={}",
        request.id, request.tag, request.ensemble_entry_id
    );

    let (frame_tx, frame_rx) = mpsc::channel();

    let io_handle = io::start_receiver_thread(
        frame_rx,
        request.params,
        request.id,
        request.tag,
        request.ensemble_entry_id,
    );

    let mut engine = Engine::new(request, frame_tx);
    engine.run();

    match io_handle.join() {
        Ok(Ok(())) => {
            debug!("Simulation completed successfully");
            Ok(())
        }
        Ok(Err(e)) => Err(format!("I/O thread failed: {}", e)),
        Err(_) => Err("I/O thread panicked".to_string()),
    }
}
