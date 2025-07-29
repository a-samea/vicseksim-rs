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
use std::sync::atomic::AtomicBool;
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
}

impl Drop for Simulation {
    /// Ensures graceful shutdown when simulation is dropped
    fn drop(&mut self) {
        self.stop();
    }
}
