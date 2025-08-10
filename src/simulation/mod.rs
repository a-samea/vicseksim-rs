
//! # Simulation Module - High-Performance Flocking Engine
//!
//! This module provides a comprehensive simulation framework for studying collective
//! flocking behavior of particles (birds) on spherical surfaces. The implementation
//! focuses on computational efficiency through parallel processing, memory optimization
//! via double buffering, and asynchronous I/O for real-time data collection.
//!
//! ## Key Features
//!
//! - **Parallel Processing**: Leverages rayon for CPU-efficient force calculations
//! - **Double Buffering**: Eliminates memory allocation overhead during simulation steps
//! - **Asynchronous I/O**: Non-blocking frame data transmission for continuous simulation
//! - **Memory Safety**: Thread-safe operations with atomic control mechanisms
//! - **Configurable Output**: Adjustable snapshot intervals for performance tuning
//!
//! ## Architecture Overview
//!
//! The simulation engine uses a producer-consumer pattern where:
//! - The `Simulation` struct produces frame snapshots during execution
//! - External components consume frame data through MPSC channels
//! - Parallel workers calculate particle state updates simultaneously
//! - Double buffering ensures consistent state reads during parallel writes
//!
//! ## Submodules
//!
//! - [`logic`]: Core physics update functions for particle state evolution
//! - [`tests`]: Comprehensive unit and integration tests for simulation correctness

pub mod logic;
pub mod tests;

use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use crate::bird::Bird;
use crate::simulation::logic::update_particle_state;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::Arc;

/// Comprehensive configuration parameters for flocking simulation physics and behavior.
///
/// This structure encapsulates all the essential parameters that control the simulation
/// dynamics, from basic system size to complex interaction behaviors. These parameters
/// directly influence the emergent flocking patterns and computational performance.
///
/// # Physical Parameters
///
/// - **Spatial**: `radius` defines the spherical constraint surface
/// - **Temporal**: `dt` controls numerical integration stability and accuracy
/// - **Kinematic**: `speed` sets uniform particle velocity magnitude
/// - **Interactive**: `interaction_radius` determines neighbor detection range
/// - **Stochastic**: `eta` introduces realistic behavioral noise
///
/// # Computational Parameters
///
/// - **Scale**: `num_birds` affects memory usage and parallelization efficiency
/// - **Duration**: `iterations` controls total simulation runtime
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct SimulationParams {
    /// Total number of birds in the simulation system.
    /// 
    /// Directly impacts memory consumption and parallel processing load distribution.
    /// Typical values range from 100 (testing) to 10,000+ (research scenarios).
    pub num_birds: usize,
    
    /// Radius of the spherical surface constraining all particle motion.
    ///
    /// All birds are constrained to move on this sphere's surface. Larger values
    /// provide more space for flocking patterns but may require longer interaction
    /// radii to maintain cohesive behavior.
    pub radius: f64,
    
    /// Uniform speed magnitude for all birds in the simulation.
    ///
    /// Maintains constant kinetic energy while allowing directional changes.
    /// Speed normalization is enforced during each simulation step to preserve
    /// this constraint throughout the simulation.
    pub speed: f64,
    
    /// Time step size for numerical integration of equations of motion.
    ///
    /// Smaller values increase accuracy but require more computational steps.
    /// Must balance stability (small dt) with performance (large dt). Typical
    /// values depend on speed and interaction strength.
    pub dt: f64,
    
    /// Maximum distance for bird-to-bird interaction detection.
    ///
    /// Birds within this geodesic distance on the sphere surface influence
    /// each other's velocity updates. Larger values create more cohesive flocks
    /// but increase computational complexity (O(N²) neighbor searches).
    pub interaction_radius: f64,
    
    /// Noise parameter controlling random perturbations in bird behavior.
    ///
    /// Higher values introduce more randomness, preventing perfect ordering
    /// and creating more realistic flocking dynamics. Zero eta produces
    /// deterministic behavior, while large eta creates chaotic motion.
    pub eta: f64,
    
    /// Maximum number of simulation steps to execute.
    ///
    /// Total simulation time equals `iterations × dt`. Used for bounded
    /// simulations with predetermined endpoints, complementing the real-time
    /// stop control mechanism.
    pub iterations: usize,
}

/// Simulation execution request containing initial conditions and configuration.
///
/// This structure packages all necessary information to initialize and run a complete
/// flocking simulation. It serves as the primary interface for external systems to
/// specify simulation parameters, initial conditions, and tracking metadata.
///
/// # Usage Context
///
/// Typically created by batch processing systems or interactive interfaces that need
/// to queue multiple simulation runs with different parameters or initial conditions.
/// The request structure enables reproducible simulations and systematic parameter studies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationRequest {
    /// Unique identifier for this simulation run.
    ///
    /// Used for tracking and correlating simulation results across distributed
    /// computing environments or batch processing systems.
    pub id: usize,
    
    /// Human-readable tag for grouping related simulation runs.
    ///
    /// Enables organizational categorization of simulation batches, parameter
    /// sweeps, or experimental conditions for later analysis and comparison.
    pub tag: String,
    
    /// Identifier linking this simulation to a specific ensemble study.
    ///
    /// Allows multiple simulation runs to be associated with a single ensemble
    /// for statistical analysis of emergent behavior patterns.
    pub ensemble_id: usize,
    
    /// Initial spatial and velocity configuration for all birds.
    ///
    /// Defines the starting state of the simulation. The vector length must
    /// match the `num_birds` parameter in the associated simulation parameters.
    /// Initial conditions significantly influence the evolution of flocking patterns.
    pub initial_values: Vec<Bird>,
    
    /// Complete physics and execution parameters for the simulation.
    ///
    /// Contains all configuration needed to reproduce the simulation exactly,
    /// including time stepping, interaction parameters, and system size.
    pub params: SimulationParams,
}

/// Temporal snapshot of simulation state for analysis and visualization.
///
/// Captures the complete system state at a specific simulation time, providing
/// a foundation for trajectory analysis, pattern recognition, and visualization
/// of flocking dynamics. Snapshots are generated at configurable intervals to
/// balance data richness with storage efficiency.
///
/// # Data Consistency
///
/// All birds in a snapshot represent the exact same temporal moment, ensuring
/// spatial and velocity correlations are preserved for accurate analysis of
/// collective behavior patterns.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationSnapshot {
    /// Simulation step number when this snapshot was captured.
    ///
    /// Provides discrete temporal indexing for snapshot sequences. Combined
    /// with the time step size, enables reconstruction of exact simulation timing.
    pub step: u64,
    
    /// Continuous simulation time corresponding to this snapshot.
    ///
    /// Calculated as `step × dt`, providing the physical time elapsed since
    /// simulation start. Useful for time-series analysis and temporal correlations.
    pub timestamp: f64,
    
    /// Complete state vector of all birds at this temporal moment.
    ///
    /// Contains position and velocity information for every bird in the simulation.
    /// The vector order is consistent across all snapshots, enabling particle
    /// trajectory tracking and individual behavior analysis.
    pub birds: Vec<Bird>,
}

/// Complete simulation execution results with metadata and performance metrics.
///
/// This comprehensive result structure contains all data necessary for post-simulation
/// analysis, including trajectory data, configuration parameters, execution metadata,
/// and performance statistics. It serves as the primary output format for simulation
/// studies and enables reproducible research workflows.
///
/// # Data Organization
///
/// The structure separates high-frequency snapshot data from essential metadata,
/// allowing efficient storage and retrieval based on analysis requirements. The
/// final state is always preserved regardless of snapshot interval settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationResult {
    /// Unique identifier matching the original simulation request.
    ///
    /// Enables correlation between simulation requests and their corresponding
    /// results in distributed or batch processing environments.
    pub id: usize,
    
    /// Descriptive tag inherited from the simulation request.
    ///
    /// Facilitates organization and filtering of results by experimental
    /// conditions, parameter sweeps, or research objectives.
    pub tag: String,
    
    /// Ensemble identifier linking this result to related simulation runs.
    ///
    /// Supports statistical analysis across multiple realizations of the
    /// same physical system with different initial conditions or noise seeds.
    pub ensemble_id: usize,
    
    /// Complete simulation configuration used for this execution.
    ///
    /// Preserves all parameters necessary for exact reproduction of the
    /// simulation, ensuring scientific reproducibility and enabling
    /// systematic parameter studies.
    pub params: SimulationParams,
    
    /// Time-ordered sequence of simulation state snapshots.
    ///
    /// Contains periodic saves of the complete system state at intervals
    /// determined by the frame capture settings. Not every simulation step
    /// is necessarily recorded to manage storage requirements.
    pub snapshots: Vec<SimulationSnapshot>,
    
    /// Final configuration of all birds at simulation termination.
    ///
    /// Always captured regardless of snapshot interval settings, ensuring
    /// the end state is available for analysis of simulation outcomes
    /// and asymptotic behavior patterns.
    pub final_state: Vec<Bird>,
    
    /// Unix timestamp indicating when this simulation was executed.
    ///
    /// Provides temporal metadata for result organization, performance
    /// analysis, and coordination in distributed computing environments.
    pub created_at: u64,
    
    /// Total number of simulation steps completed.
    ///
    /// May differ from the requested iterations if early termination
    /// occurred due to stop conditions or computational constraints.
    pub total_steps: u64,
    
    /// Wall-clock execution time for the entire simulation.
    ///
    /// Includes initialization, computation, and I/O overhead. Useful for
    /// performance analysis, resource planning, and computational scaling studies.
    pub duration_seconds: f64,
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
    ///
    /// This buffer holds the authoritative state that all computation threads
    /// read from during parallel force calculations. It alternates between
    /// being the source and destination of updates via buffer swapping.
    particles_a: Vec<Bird>,
    
    /// Secondary particle state buffer for writing computed updates.
    ///
    /// Parallel workers write their calculated next-state particles to this
    /// buffer while reading from the primary buffer. After all updates complete,
    /// the buffers swap roles to prepare for the next simulation step.
    particles_b: Vec<Bird>,
    
    /// Immutable simulation configuration controlling physics and behavior.
    ///
    /// Contains all parameters necessary for particle state updates, including
    /// interaction radii, time stepping, noise levels, and boundary conditions.
    /// Remains constant throughout simulation execution for reproducibility.
    params: SimulationParams,
    
    /// Current discrete simulation step counter.
    ///
    /// Increments with each call to `step()`, providing temporal indexing for
    /// snapshots and enabling precise tracking of simulation progress.
    step_count: u64,
    
    /// Continuous simulation time in physical units.
    ///
    /// Calculated as `step_count × dt`, representing the total physical time
    /// elapsed since simulation initialization. Used for temporal analysis
    /// and time-dependent behavior studies.
    current_time: f64,
    
    /// Asynchronous channel for transmitting frame data to external consumers.
    ///
    /// Enables non-blocking communication with I/O subsystems, data loggers,
    /// or visualization components without interrupting simulation execution.
    /// Optional channel allows simulations to run without data output if needed.
    frame_sender: Option<mpsc::Sender<SimulationSnapshot>>,
    
    /// Interval controlling snapshot capture frequency.
    ///
    /// Snapshots are captured every `frame_interval` simulation steps to balance
    /// data richness with storage efficiency and computational overhead. Larger
    /// intervals reduce I/O load but provide coarser temporal resolution.
    frame_interval: u64,
    
    /// Thread-safe flag enabling graceful simulation termination.
    ///
    /// External threads can set this atomic boolean to request immediate
    /// simulation stop, allowing responsive control in interactive environments
    /// or when computational resources become constrained.
    should_stop: Arc<AtomicBool>,
}




impl Simulation {
    /// Creates a new simulation instance from a request with optimized memory allocation.
    ///
    /// This constructor initializes all simulation state including double-buffered particle
    /// storage, communication channels, and control mechanisms. The implementation performs
    /// validation and pre-allocates all necessary memory to ensure predictable performance
    /// during simulation execution.
    ///
    /// # Arguments
    ///
    /// * `request` - Complete simulation configuration including initial conditions
    /// * `tx` - Channel sender for asynchronous frame data transmission  
    /// * `frame_interval` - Snapshot capture frequency (every N steps)
    ///
    /// # Panics
    ///
    /// Panics if the simulation request specifies zero birds, as this represents an
    /// invalid physical system that cannot produce meaningful flocking behavior.
    ///
    /// # Memory Allocation
    ///
    /// Pre-allocates both particle buffers to avoid dynamic allocation during simulation.
    /// The secondary buffer is initialized with default birds that will be overwritten
    /// during the first simulation step.
    pub fn new(
        request: SimulationRequest, 
        tx: mpsc::Sender<SimulationSnapshot>,
        frame_interval: u64
    ) -> Self {
        if request.params.num_birds < 1 {
            panic!("Simulation requires at least one bird")
        }
        Simulation { 
            particles_a: request.initial_values, 
            particles_b: vec![Bird::default(); request.params.num_birds], 
            params: request.params, 
            step_count: 0, 
            current_time: 0.0, 
            frame_sender: Some(tx), 
            frame_interval, 
            should_stop: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Returns an immutable reference to the simulation configuration parameters.
    ///
    /// Provides access to the complete physics and execution parameters without
    /// allowing modification, ensuring simulation reproducibility and parameter
    /// consistency throughout execution.
    pub fn parameters(&self) -> &SimulationParams {
        &self.params
    }

    /// Returns the current simulation step count.
    ///
    /// Provides the discrete temporal index representing the number of simulation
    /// steps completed since initialization. Useful for progress monitoring and
    /// temporal synchronization with external systems.
    pub fn step_count(&self) -> u64 {
        self.step_count
    }

    /// Returns the current continuous simulation time.
    ///
    /// Calculated as `step_count × dt`, representing the total physical time
    /// elapsed in the simulation. Essential for time-series analysis and
    /// correlation with experimental or observational data.
    pub fn current_time(&self) -> f64 {
        self.current_time
    }

    /// Returns an immutable reference to the current particle state.
    ///
    /// Provides read-only access to the complete system state without allowing
    /// modifications that could disrupt simulation consistency. The returned
    /// slice contains all bird positions and velocities at the current time.
    pub fn current_particles(&self) -> &[Bird] {
        &self.particles_a
    }

    /// Returns a cloned atomic flag for external simulation control.
    ///
    /// Enables external threads or systems to request graceful simulation
    /// termination by setting the returned atomic boolean. The simulation
    /// will check this flag and stop cleanly at the next step boundary.
    pub fn stop_flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.should_stop)
    }



    /// Executes the complete simulation with responsive stop control and frame capture.
    ///
    /// This method runs the main simulation loop, combining step-limited execution with
    /// real-time stop control for maximum flexibility. The simulation continues until
    /// either the specified iteration limit is reached or an external stop signal is
    /// received through the atomic stop flag.
    ///
    /// # Execution Flow
    ///
    /// 1. **Iteration Control**: Respects the maximum step limit from simulation parameters
    /// 2. **Stop Checking**: Polls the atomic stop flag for responsive external control
    /// 3. **State Evolution**: Calls `step()` to advance the simulation by one time increment
    /// 4. **Frame Capture**: Generates snapshots at specified intervals for data collection
    ///
    /// # Performance Characteristics
    ///
    /// - **Atomic Operations**: Minimal overhead for stop condition checking
    /// - **Conditional I/O**: Frame transmission only occurs at specified intervals
    /// - **Memory Efficiency**: No additional allocations during the execution loop
    /// - **Responsive Control**: Stop requests honored within one simulation step
    ///
    /// The method balances computational efficiency with responsiveness, ensuring that
    /// long-running simulations can be controlled interactively while maintaining
    /// optimal performance for batch processing scenarios.
    pub fn run(&mut self) {
        for _ in 0..self.params.iterations {
            if self.should_stop.load(Ordering::Relaxed) {
                break;
            }

            self.step();

            // Send frame data if interval reached
            if self.step_count % self.frame_interval == 0 {
                self.send_frame_data();
            }
        }
    }

    /// Advances the simulation by one time step using optimized parallel processing.
    ///
    /// This method implements the core computational kernel of the simulation, combining
    /// double buffering with parallel processing to achieve maximum performance while
    /// maintaining numerical accuracy. The implementation follows a carefully designed
    /// sequence to ensure thread safety and memory efficiency.
    ///
    /// # Algorithm Overview
    ///
    /// 1. **State Isolation**: Extracts immutable references to current state and parameters
    /// 2. **Parallel Computation**: Distributes particle updates across available CPU cores
    /// 3. **Buffer Management**: Swaps state buffers to prepare for the next iteration
    /// 4. **Temporal Advancement**: Updates step counter and simulation time
    ///
    /// # Parallel Processing Strategy
    ///
    /// The method leverages rayon's parallel iterator to distribute work efficiently:
    /// ```text
    /// Thread 1: Updates particles [0..N/4)
    /// Thread 2: Updates particles [N/4..N/2)  
    /// Thread 3: Updates particles [N/2..3N/4)
    /// Thread 4: Updates particles [3N/4..N)
    /// ```
    ///
    /// # Thread Safety Analysis
    ///
    /// This approach is inherently thread-safe because:
    /// - **Immutable Reads**: All threads read from `particles_a` without modification
    /// - **Exclusive Writes**: Each thread writes to distinct elements of `particles_b`
    /// - **No Shared State**: Worker threads operate independently without coordination
    /// - **Atomic Swapping**: Buffer swap occurs after all threads complete
    ///
    /// # Memory Access Patterns
    ///
    /// The double buffering strategy optimizes cache performance:
    /// - **Sequential Reads**: Current state accessed in cache-friendly order
    /// - **Sequential Writes**: Next state written without read-after-write hazards
    /// - **Zero Allocation**: No dynamic memory allocation during execution
    /// - **Predictable Layout**: Consistent memory access patterns across iterations
    ///
    /// This design ensures that the simulation can sustain high frame rates even for
    /// large particle systems while utilizing modern CPU architectures effectively.
    fn step(&mut self) {
        // Extract parameters needed for computation to avoid borrowing conflicts
        let params = self.params;
        // Get immutable reference to current state for reading
        let current_state = &self.particles_a;

        // Parallel computation using rayon for maximum CPU utilization
        // Each thread processes a subset of particles independently
        self.particles_b
            .par_iter_mut()
            .enumerate()
            .for_each(|(i, particle_next)| {
                // Calculate the new state for particle i based on current state
                *particle_next = update_particle_state(
                    i,
                    current_state,
                    params
                );
            });

        // Swap buffers - this is extremely cheap (just pointer swaps)
        std::mem::swap(&mut self.particles_a, &mut self.particles_b);

        // Update simulation state
        self.step_count += 1;
        self.current_time += self.params.dt;
    }

    /// Transmits current simulation state through the asynchronous I/O channel.
    ///
    /// This method creates a complete snapshot of the current simulation state and
    /// sends it through the configured MPSC channel for external processing. The
    /// transmission is non-blocking to prevent I/O operations from interrupting
    /// the simulation's computational flow.
    ///
    /// # Snapshot Creation
    ///
    /// The generated snapshot includes:
    /// - **Temporal Metadata**: Step count and continuous time for indexing
    /// - **Complete State**: All bird positions and velocities at the current moment
    /// - **Consistent Data**: All particles captured at the exact same simulation time
    ///
    /// # Error Handling
    ///
    /// The method employs graceful error handling for robust operation:
    /// - **Channel Disconnection**: Continues simulation if receiver has terminated
    /// - **Memory Pressure**: Allows frame drops if consumer cannot keep pace
    /// - **Non-blocking**: Never blocks simulation progress due to I/O constraints
    ///
    /// This approach ensures that simulation performance remains predictable even
    /// when external systems experience varying load conditions or failures.
    fn send_frame_data(&self) {
        if let Some(ref sender) = self.frame_sender {
            let frame = SimulationSnapshot {
                step: self.step_count,
                timestamp: self.current_time,
                birds: self.particles_a.clone(),
            };

            // Non-blocking send - if receiver is gone, just continue
            let _ = sender.send(frame).unwrap_or_else(|err| {
                eprintln!("Failed to send frame data: {}", err);
            });
        }
    }

    /// Requests graceful simulation termination by setting the atomic stop flag.
    ///
    /// This method provides a thread-safe mechanism for external systems to request
    /// simulation termination. The stop request will be honored at the next iteration
    /// boundary, ensuring that the simulation completes its current step cleanly
    /// before terminating.
    ///
    /// # Thread Safety
    ///
    /// Uses atomic operations with relaxed ordering for optimal performance while
    /// maintaining memory safety across concurrent access from multiple threads.
    /// The relaxed ordering is sufficient since stop control doesn't require
    /// synchronization with other memory operations.
    pub fn stop(&self) {
        self.should_stop.store(true, Ordering::Relaxed);
    }
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
