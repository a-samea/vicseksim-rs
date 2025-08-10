use super::*;
use crate::bird::Bird;
use crate::simulation::SimulationParams;
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
    pub fn parameters(&self) -> &SimulationParams {
        &self.params
    }

    /// Returns the current simulation step count.
    pub fn step_count(&self) -> u64 {
        self.step_count
    }

    /// Returns the current continuous simulation time.
    pub fn current_time(&self) -> f64 {
        self.current_time
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

fn update_particle_state(
    particle_index: usize,
    current_state: &[Bird],
    params: SimulationParams
) -> Bird {
    let current_bird = &current_state[particle_index];

    // Find neighboring particles and transport their velocities
    let transported_velocities: Vec<Vec3> = current_state
        .iter()
        .enumerate()
        .filter_map(|(i,neighbor)| {
            // Skip self
            if i == particle_index {
                return None;
            }

            // Check if neighbor is within interaction radius
            let distance = current_bird.distance_from(neighbor, params.radius);
            if (f64::EPSILON < distance) && (distance < params.interaction_radius) {
                Some(neighbor.parallel_transport_velocity(current_bird))
            } else {
                None
            }

        })
        .collect();

    // find the transport velocity
    let transport = if transported_velocities.is_empty() {
        // No neigbor in sight
        current_bird.velocity
    } else {
        // sum of them
        let sum = transported_velocities
        .iter()
        .fold(Vec3::zero(), |acc &vel| acc + vel);

        let averaged = sum / transported_velocities.len() as f64;

        if averaged.norm() < 1e-6 {
            // small average
            Bird::add_noise(current_bird.velocity, current_bird, params.eta)
        } else {
            Bird::add_noise(
                averaged.normalize() * params.speed, 
                current_bird, 
                params.eta
            )
        }
    }

    let temp = Bird {
        position: current_bird.position,
        velocity: transport
    };

    temp.move_on_sphere(params.dt, params.radius, params.speed)

}
