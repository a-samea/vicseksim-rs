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
//!
//! ## Usage Example
//!
//! ```rust
//! use flocking_lib::simulation::Simulation;
//! use flocking_lib::bird::Bird;
//! use std::sync::mpsc;
//!
//! // Create I/O channel for frame data
//! let (frame_sender, frame_receiver) = mpsc::channel();
//!
//! // Initialize simulation with birds
//! let birds = vec![/* initial bird positions */];
//! let mut sim = Simulation::new(birds, frame_sender, 100);
//!
//! // Run simulation for specific number of steps
//! sim.run_for_steps(1000);
//!
//! // Or run until stopped
//! sim.run_until_stopped();
//! ```
pub mod logic;
pub mod tests;

use crate::bird::Bird;
use rayon::prelude::*;
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
    /// * `frame_sender` - Channel sender for asynchronous frame data output
    /// * `frame_interval` - Save frame data every N simulation steps
    ///
    /// # Examples
    ///
    /// ```rust
    /// use flocking_lib::simulation::Simulation;
    /// use flocking_lib::bird::Bird;
    /// use std::sync::mpsc;
    ///
    /// let (tx, rx) = mpsc::channel();
    /// let birds = vec![Bird::new(/* position */, /* velocity */)];
    /// let sim = Simulation::new(birds, tx, 10);
    /// ```
    pub fn new(
        initial_birds: Vec<Bird>,
        frame_sender: mpsc::Sender<FrameData>,
        frame_interval: u64,
    ) -> Self {
        let num_particles = initial_birds.len();

        Simulation {
            particles_a: initial_birds,
            particles_b: vec![
                Bird::new(
                    crate::vector::Vec3::new(0.0, 0.0, 0.0),
                    crate::vector::Vec3::new(0.0, 0.0, 0.0)
                );
                num_particles
            ],
            params: SimulationParams::default(),
            step_count: 0,
            current_time: 0.0,
            frame_sender: Some(frame_sender),
            frame_interval,
            should_stop: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Creates a new simulation without frame output capability
    ///
    /// Useful for benchmarking or when frame data is not needed.
    pub fn new_no_output(initial_birds: Vec<Bird>) -> Self {
        let num_particles = initial_birds.len();

        Simulation {
            particles_a: initial_birds,
            particles_b: vec![
                Bird::new(
                    crate::vector::Vec3::new(0.0, 0.0, 0.0),
                    crate::vector::Vec3::new(0.0, 0.0, 0.0)
                );
                num_particles
            ],
            params: SimulationParams::default(),
            step_count: 0,
            current_time: 0.0,
            frame_sender: None,
            frame_interval: 1,
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
