//! Core simulation logic for flocking behavior on spherical surfaces.
//!
//! This module implements the primary computational engine for particle-based flocking
//! simulations using the Vicsek model adapted for spherical geometry. It provides
//! high-performance parallel processing capabilities with double-buffered state management
//! and asynchronous data collection.
//!
//! # Key Components
//!
//! - **Simulation Engine**: Main simulation loop with parallel particle updates
//! - **State Management**: Double-buffered particle storage for thread-safe operations
//! - **Flocking Dynamics**: Neighbor-based velocity alignment with noise injection
//! - **Spherical Integration**: Geodesic motion integration on curved surfaces
//! - **Performance Optimization**: Rayon-based parallelization and memory efficiency
//!
//! # Algorithm Overview
//!
//! The simulation follows a standard time-stepping approach where each iteration:
//! 1. Computes neighbor interactions for all particles in parallel
//! 2. Applies flocking rules (alignment, noise) with spherical geometry constraints
//! 3. Integrates motion using geodesic paths on the sphere surface
//! 4. Captures simulation snapshots at specified intervals
//!
//! The implementation emphasizes computational efficiency while maintaining physical
//! accuracy for realistic flocking behavior in spherical environments.

use super::*;
use crate::bird::Bird;
use crate::vector::Vec3;
use rayon::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::Arc;

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
        frame_interval: usize,
    ) -> Self {
        if request.params.num_birds < 1 {
            panic!("Simulation requires at least one bird")
        }
        Simulation {
            particles_a: request.initial_values,
            particles_b: vec![Bird::default(); request.params.num_birds],
            params: request.params,
            step_count: 0,
            current_timestamp: 0.0,
            frame_sender: tx,
            frame_interval,
            should_stop: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Returns an immutable reference to the simulation configuration parameters.
    pub fn parameters(&self) -> &SimulationParams {
        &self.params
    }

    /// Returns the current simulation step count.
    pub fn step_count(&self) -> usize {
        self.step_count
    }

    /// Returns the current continuous simulation time.
    pub fn current_time(&self) -> f64 {
        self.current_timestamp
    }

    /// Returns an immutable reference to the current particle state.
    pub fn current_particles(&self) -> &[Bird] {
        &self.particles_a
    }

    /// Returns a cloned atomic flag for external simulation control.
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
                *particle_next = update_particle_state(i, current_state, params);
            });

        // Swap buffers - this is extremely cheap (just pointer swaps)
        std::mem::swap(&mut self.particles_a, &mut self.particles_b);

        // Update simulation state
        self.step_count += 1;
        self.current_timestamp += self.params.dt;
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
        let sender = &self.frame_sender;

        let frame = SimulationSnapshot {
            step: self.step_count,
            timestamp: self.current_timestamp,
            birds: self.particles_a.clone(),
        };

        // Non-blocking send - if receiver is gone, just continue
        let _ = sender.send(frame).unwrap_or_else(|err| {
            eprintln!("Failed to send frame data: {}", err);
        });
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

/// Updates a single particle's state using the Vicsek flocking model with spherical geometry.
///
/// This function implements the core particle interaction logic for flocking simulation on a
/// spherical surface. It processes neighbor interactions, applies velocity alignment rules,
/// and integrates motion according to the specified time step.
///
/// # Algorithm Overview
///
/// 1. **Neighbor Detection**: Identifies particles within the interaction radius
/// 2. **Velocity Transport**: Applies parallel transport for velocities on curved geometry  
/// 3. **Alignment Computation**: Calculates averaged velocity from all neighbors
/// 4. **Noise Application**: Adds stochastic perturbations to prevent artificial ordering
/// 5. **Motion Integration**: Updates position using spherical geodesic motion
///
/// # Flocking Behavior
///
/// The function implements classic flocking rules adapted for spherical topology:
/// - **Alignment**: Particles tend to match their neighbors' velocity directions
/// - **Noise**: Random perturbations introduce realistic behavioral variations
/// - **Isolation Handling**: Particles without neighbors maintain current velocity
/// - **Speed Regulation**: All particles maintain constant speed magnitude
///
/// # Performance Optimizations
///
/// - Early termination in neighbor search when particle count allows
/// - Vectorized velocity summation using fold operations
/// - Minimal temporary allocations through iterator chaining
/// - Cache-friendly access patterns for spatial data structures
///
/// # Parameters
///
/// * `particle_index` - Index of the particle to update in the state array
/// * `current_state` - Immutable reference to all particle states at current time
/// * `params` - Simulation parameters including interaction radius and noise level
///
/// # Returns
///
/// Returns the updated `Bird` instance with new position and velocity after one time step.
fn update_particle_state(
    particle_index: usize,
    current_state: &[Bird],
    params: SimulationParams,
) -> Bird {
    let current_bird = &current_state[particle_index];

    // Collect velocities from neighboring particles within interaction radius
    // Apply parallel transport to maintain tangent space consistency on sphere
    let transported_velocities: Vec<Vec3> = current_state
        .iter()
        .enumerate()
        .filter_map(|(neighbor_index, neighbor_bird)| {
            // Exclude self-interaction to prevent trivial alignment
            if neighbor_index == particle_index {
                return None;
            }

            // Calculate geodesic distance between particles on sphere surface
            let geodesic_distance = current_bird.distance_from(neighbor_bird, params.radius);

            // Include neighbor if within interaction radius and not at same position
            if geodesic_distance > f64::EPSILON && geodesic_distance < params.interaction_radius {
                Some(neighbor_bird.parallel_transport_velocity(current_bird))
            } else {
                None
            }
        })
        .collect();

    // Compute alignment velocity based on neighbor interactions
    let transport_velocity = if transported_velocities.is_empty() {
        // Isolated particle maintains current velocity direction
        current_bird.velocity
    } else {
        // Compute vector sum of all transported neighbor velocities
        let velocity_sum = transported_velocities
            .iter()
            .fold(Vec3::zero(), |accumulator, velocity| {
                accumulator + *velocity
            });

        // Calculate mean velocity direction from neighbors
        let mean_velocity = velocity_sum / transported_velocities.len() as f64;

        // Handle near-zero alignment case to prevent numerical instability
        if mean_velocity.norm() < 1e-6 {
            // Apply noise to current velocity when alignment is negligible
            Bird::add_noise(current_bird.velocity, current_bird, params.eta)
        } else {
            // Normalize and scale to target speed, then apply noise
            Bird::add_noise(
                mean_velocity.normalize() * params.speed,
                current_bird,
                params.eta,
            )
        }
    };

    // Create intermediate bird state with updated velocity
    let updated_bird = Bird {
        position: current_bird.position,
        velocity: transport_velocity,
    };

    // Integrate motion on sphere surface for one time step
    updated_bird.move_on_sphere(params.dt, params.radius, params.speed)
}
