//! # Simulation Module - High-Performance Flocking Engine
//!
//! create!

pub mod io;
pub mod logic;
pub mod tests;

use crate::bird::Bird;
use serde::{Deserialize, Serialize};
use std::sync::atomic::AtomicBool;
use std::sync::mpsc;
use std::sync::Arc;

/// Comprehensive configuration parameters for flocking simulation physics and behavior.
///
/// This structure encapsulates all the essential parameters that control the simulation
/// dynamics, from basic system size to complex interaction behaviors. These parameters
/// directly influence the emergent flocking patterns and computational performance.
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
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
    pub iterations: usize,
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    /// Final configuration of all birds at simulation termination.
    pub final_state: Vec<Bird>,
    /// Unix timestamp indicating when this simulation was executed.
    pub created_at: usize,
    /// Total number of simulation steps completed.
    pub total_steps: usize,
}

/// High-performance flocking simulation engine with parallel processing and memory optimization.
///
/// The `Simulation` struct implements a sophisticated computational framework for studying
/// collective flocking behavior on spherical surfaces. It combines several advanced techniques
/// to achieve optimal performance while maintaining scientific accuracy:
///
/// ## Performance Optimizations
///
/// - **Double Buffering**: Eliminates dynamic memory allocation during simulation steps
/// - **Parallel Processing**: Leverages rayon for CPU-efficient force calculations
/// - **Memory Locality**: Optimized data layout for cache-friendly access patterns
/// - **Asynchronous I/O**: Non-blocking frame transmission prevents simulation stalls
///
/// ## Concurrency Architecture
///
/// The simulation uses a carefully designed concurrency model:
/// - **Read-Only Access**: All threads read current state immutably (thread-safe)
/// - **Exclusive Writes**: Each thread writes to distinct memory locations (no contention)
/// - **Atomic Control**: Stop conditions use atomic operations for immediate response
/// - **Buffer Swapping**: Cheap pointer swaps enable seamless state transitions
///
/// ## Memory Management
///
/// Double buffering strategy eliminates allocation overhead:
/// ```text
/// Step N:   Read from A → Compute → Write to B → Swap(A,B)
/// Step N+1: Read from B → Compute → Write to A → Swap(A,B)
/// ```
///
/// This approach ensures consistent memory usage regardless of simulation length and
/// eliminates garbage collection pauses that could disrupt real-time performance.
pub struct Simulation {
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
    /// Interval controlling snapshot capture frequency.
    frame_interval: usize,
    /// Thread-safe flag enabling graceful simulation termination.
    should_stop: Arc<AtomicBool>,
}

impl Drop for Simulation {
    /// Ensures graceful shutdown when the simulation instance is destroyed.
    ///
    /// This destructor automatically sets the stop flag when the simulation
    /// goes out of scope, providing a fail-safe mechanism to prevent runaway
    /// threads or resource leaks in case of unexpected termination scenarios.
    ///
    /// The implementation ensures that any external threads monitoring the
    /// stop flag will be notified of the simulation's termination, enabling
    /// coordinated cleanup of associated resources.
    fn drop(&mut self) {
        self.stop();
    }
}
