//! # Simulation Module - High-Performance Flocking Simulation Engine
//!
//! This module provides a high-performance simulation engine for flocking behavior on spherical
//! surfaces. It implements a double-buffered, parallelized simulation system using `rayon` for
//! maximum CPU utilization and channels for asynchronous I/O operations.
//!
//! ## Key Features
//!
//! - **Double Buffering**: Uses `std::mem::swap` for zero-copy buffer switching between simulation steps
//! - **Parallel Processing**: Leverages `rayon` for parallel particle updates across multiple CPU cores
//! - **Asynchronous I/O**: Non-blocking frame data transmission to disk through channels
//! - **Memory Efficient**: Minimizes allocations and memory copies during simulation
//! - **Configurable**: Flexible simulation parameters and output control
//!
//! ## Architecture
//!
//! The simulation uses a time-stepping approach where each step depends only on the previous state.
//! This enables efficient parallelization and memory management:
//!
//! ```text
//! Step N:   Read from Buffer A  →  Write to Buffer B  →  Swap A↔B
//! Step N+1: Read from Buffer B  →  Write to Buffer A  →  Swap A↔B
//! ```
pub mod logic;
pub mod tests;

use crate::bird::Bird;
use crate::simulation::logic::calculate_new_particle_state;
use rayon::iter::IntoParallelRefMutIterator;
use rayon::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::Arc;

/// Simulation parameters controlling the flocking behavior and physics
#[derive(Debug, Clone)]
pub struct SimulationParams {
    /// Number of Birds
    pub num_birds: usize,
    /// Sphere radius
    pub radius: f64,
    /// Speed of all birds
    pub speed: f64,
    /// Time step size for numerical integration
    pub dt: f64,
    /// Interaction radius for flocking forces
    pub interaction_radius: f64,
    /// Noise parameter
    pub eta: f64,
    /// Number of steps to run
    pub iterations: usize,
}

/// Frame data structure sent through the I/O channel
#[derive(Debug, Clone)]
pub struct FrameData {
    /// Simulation step number
    pub step: u64,
    /// Timestamp of the frame
    pub timestamp: f64,
    /// Snapshot of all bird states at this frame
    pub birds: Vec<Bird>,
}

/// High-performance flocking simulation engine with double buffering and parallel processing
///
/// The `Simulation` struct manages the complete simulation lifecycle, including:
/// - Double-buffered particle state management
/// - Parallel force calculations using rayon
/// - Asynchronous frame data output
/// - Simulation control and monitoring
pub struct Simulation {
    /// Primary particle buffer (current state)
    particles_a: Vec<Bird>,
    /// Secondary particle buffer (next state)
    particles_b: Vec<Bird>,
    /// Simulation parameters controlling physics and behavior
    params: SimulationParams,
    /// Current simulation step counter
    step_count: u64,
    /// Current simulation time
    current_time: f64,
    /// Channel sender for frame data output
    frame_sender: Option<mpsc::Sender<FrameData>>,
    /// Frame output interval (save every N steps)
    frame_interval: u64,
    /// Atomic flag for graceful simulation stopping
    should_stop: Arc<AtomicBool>,
}

impl Simulation {
    /// Creates a new simulation instance with the given initial bird configuration
    ///
    /// # Arguments
    ///
    /// * `initial_birds` - Vector of birds representing the initial state
    /// * `dt` - Time step size for numerical integration
    /// * `iterations` - Total number of simulation steps to run
    /// * `interaction_radius` - Radius for flocking interactions
    /// * `noise_param` - Noise parameter for stochastic behavior
    /// * `frame_sender` - Channel sender for asynchronous frame data output
    /// * `frame_interval` - Save frame data every N simulation steps
    ///
    /// # Returns
    ///
    /// A new `Simulation` instance configured with the provided parameters
    ///
    /// # Panics
    ///
    /// Panics if `initial_birds` is empty, as at least one bird is required to start the simulation.
    pub fn new(
        initial_birds: Vec<Bird>,
        dt: f64,
        iterations: usize,
        interaction_radius: f64,
        noise_param: f64,
        frame_sender: mpsc::Sender<FrameData>,
        frame_interval: u64,
    ) -> Self {
        let num_birds = initial_birds.len();
        if num_birds < 1 {
            panic!("Simulation requires at least one bird to start");
        }
        let radius = initial_birds.first().unwrap().position.norm();
        let speed = initial_birds.first().unwrap().velocity.norm();
        let params = SimulationParams {
            num_birds,
            radius,
            speed,
            dt,
            interaction_radius,
            eta: noise_param,
            iterations,
        };

        Simulation {
            particles_a: initial_birds,
            particles_b: vec![Bird::default(); num_birds],
            params,
            step_count: 0,
            current_time: 0.0,
            frame_sender: Some(frame_sender),
            frame_interval,
            should_stop: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Gets a reference to current simulation parameters
    pub fn parameters(&self) -> &SimulationParams {
        &self.params
    }

    /// Returns the current simulation step count
    pub fn step_count(&self) -> u64 {
        self.step_count
    }

    /// Returns the current simulation time
    pub fn current_time(&self) -> f64 {
        self.current_time
    }

    /// Gets a reference to the current particle state (read-only)
    pub fn current_particles(&self) -> &[Bird] {
        &self.particles_a
    }

    /// Returns a clone of the stop flag for external control
    pub fn stop_flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.should_stop)
    }
    /// Runs the simulation with a maximum step limit and stop condition
    ///
    /// Combines both approaches: runs until either the step limit is reached
    /// or the stop flag is set, whichever comes first.
    ///
    /// # Arguments
    ///
    /// * `max_steps` - Maximum number of steps to run
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

    /// Runs the simulation until the stop flag is set
    ///
    /// This method runs indefinitely until `should_stop` is set to true
    /// from another thread. Useful for interactive simulations or when
    /// the stopping condition is external.
    pub fn run_until_stopped(&mut self) {
        while !self.should_stop.load(Ordering::Relaxed) {
            self.step();

            // Send frame data if interval reached
            if self.step_count % self.frame_interval == 0 {
                self.send_frame_data();
            }
        }
    }

    /// Performs a single simulation step using double buffering and parallel processing
    ///
    /// This is the core simulation method that:
    /// 1. Reads the current state from one buffer
    /// 2. Calculates new states in parallel using rayon
    /// 3. Writes results to the other buffer
    /// 4. Swaps buffers for the next iteration
    fn step(&mut self) {
        // Extract parameters needed for computation to avoid borrowing conflicts
        let dt = self.params.dt;
        let interaction_radius = self.params.interaction_radius;
        let eta = self.params.eta;
        let speed = self.params.speed;
        let radius = self.params.radius;

        // Get immutable reference to current state for reading
        let current_state = &self.particles_a;

        // Parallel computation using rayon for maximum CPU utilization
        // Each thread processes a subset of particles independently
        self.particles_b
            .par_iter_mut()
            .enumerate()
            .for_each(|(i, particle_next)| {
                // Calculate the new state for particle *i* based on current state
                *particle_next = calculate_new_particle_state(
                    i,
                    current_state,
                    dt,
                    interaction_radius,
                    eta,
                    speed,
                    radius,
                );
            });

        // Swap buffers - this is extremely cheap (just pointer swaps)
        std::mem::swap(&mut self.particles_a, &mut self.particles_b);

        // Update simulation state
        self.step_count += 1;
        self.current_time += self.params.dt;
    }

    /// Sends current frame data through the I/O channel
    ///
    /// This method creates a snapshot of the current simulation state and
    /// sends it asynchronously for disk I/O processing.
    fn send_frame_data(&self) {
        if let Some(ref sender) = self.frame_sender {
            let frame = FrameData {
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

    /// Gracefully stops the simulation by setting the stop flag
    pub fn stop(&self) {
        self.should_stop.store(true, Ordering::Relaxed);
    }

    /// Resets the simulation to initial conditions
    ///
    /// # Arguments
    ///
    /// * `initial_birds` - New initial bird configuration
    pub fn reset(&mut self, initial_birds: Vec<Bird>) {
        let num_particles = initial_birds.len();

        self.particles_a = initial_birds;
        self.particles_b = vec![Bird::default(); num_particles];
        self.step_count = 0;
        self.current_time = 0.0;
        self.should_stop.store(false, Ordering::Relaxed);
    }
}

impl Drop for Simulation {
    /// Ensures graceful shutdown when simulation is dropped
    fn drop(&mut self) {
        self.stop();
    }
}
