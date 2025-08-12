//! Core simulation logic for flocking behavior on spherical surfaces.
//!

use super::*;
use crate::bird::Bird;
use crate::vector::Vec3;
use log::error;
use rayon::prelude::*;
use std::sync::mpsc;

impl Engine {
    /// Creates a new simulation instance from a request with optimized memory allocation.
    ///
    pub fn new(
        request: SimulationRequest,
        tx: mpsc::Sender<SimulationSnapshot>,
        frame_interval: usize,
    ) -> Self {
        if request.params.num_birds < 1 {
            panic!("Simulation requires at least one bird")
        }
        Engine {
            particles_a: request.initial_values,
            particles_b: vec![Bird::default(); request.params.num_birds],
            params: request.params,
            step_count: 0,
            current_timestamp: 0.0,
            frame_sender: tx,
            frame_interval,
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

    /// Executes the complete simulation with responsive stop control and frame capture.
    ///
    pub fn run(&mut self) {
        for _ in 0..self.params.iterations {
            self.step();

            // Send frame data if interval reached
            if self.step_count % self.frame_interval == 0 {
                self.send_frame_data();
            }
        }
    }

    /// Advances the simulation by one time step using optimized parallel processing.
    ///
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
    /// clones vector (bottleneck) and sends it to the receiver.
    fn send_frame_data(&self) {
        let sender = &self.frame_sender;

        let frame = SimulationSnapshot {
            step: self.step_count,
            timestamp: self.current_timestamp,
            birds: self.particles_a.clone(),
        };

        // Non-blocking send - if receiver is gone, just continue
        let _ = sender.send(frame).unwrap_or_else(|err| {
            error!("Failed to send frame data: {}", err);
        });
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
