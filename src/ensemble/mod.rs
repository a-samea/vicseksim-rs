//! # Ensemble Generation Module
//!
//! This module provides functionality for generating ensembles of birds (particles) for flocking
//! simulations. An ensemble represents a collection of birds positioned on a spherical surface
//! with specified constraints and initial conditions.
//!
//! ## Overview
//!
//! The ensemble module is responsible for:
//! - Generating collections of birds with uniform spatial distribution on spherical surfaces
//! - Enforcing minimum distance constraints between particles to prevent overlapping
//! - Providing structured data types for ensemble metadata and generation parameters
//! - Supporting concurrent ensemble generation through MPSC channels
//! - Integrating with the IO system for persistence and batch processing
//!
//! ## Key Concepts
//!
//! ### Spherical Distribution
//! Birds are distributed uniformly on the surface of a sphere using proper spherical coordinate
//! sampling. This ensures no clustering around poles and maintains rotational symmetry.
//!
//! ### Rejection Sampling
//! To maintain minimum distance constraints, the module uses rejection sampling - candidate
//! birds that are too close to existing birds are discarded and new positions are generated.
//!
//! ### Ensemble Metadata
//! Each ensemble includes comprehensive metadata including unique identifiers, generation
//! parameters, timestamps, and tags for organization and batch processing.
//!
//! ## Usage Patterns
//!
//! ### Single Ensemble Generation
//! ```rust
//! use std::sync::mpsc;
//! use flocking_lib::ensemble;
//! use flocking_lib::ensemble::{EnsembleGenerationRequest, EnsembleGenerationParams};
//!
//! let (tx, rx) = mpsc::channel();
//!
//! let request = EnsembleGenerationRequest {
//!     id: 1,
//!     tag: "test_ensemble".to_string(),
//!     params: EnsembleGenerationParams {
//!         n_particles: 50,
//!         radius: 1.0,
//!         speed: 1.5,
//!         min_distance: 0.1,
//!     },
//! };
//!
//! // Generate ensemble in background thread
//! std::thread::spawn(move || {
//!     ensemble::generate(request, tx).unwrap();
//! });
//!
//! // Receive completed ensemble
//! let result = rx.recv().unwrap();
//! println!("Generated {} birds", result.birds.len());
//! ```
//!
//! ## Performance Considerations
//!
//! - **Time Complexity**: O(n²) worst case due to distance checking during rejection sampling
//! - **Memory Usage**: Pre-allocated vectors minimize memory fragmentation
//! - **Parallelization**: Thread-safe design allows multiple ensembles to be generated concurrently
//! - **Distance Constraints**: Tighter `min_distance` values increase generation time exponentially
//!
//! ## Integration Points
//!
//! - **Bird Module**: Uses `Bird::from_spherical()` and `Bird::distance_from()` methods
//! - **IO Module**: Provides `EnsembleResult` for persistence and loading
//! - **Simulation Module**: Generated ensembles serve as initial conditions for simulations
//! - **Analysis Module**: Ensemble metadata enables batch analysis and comparison

use crate::bird::Bird;
use rand::prelude::*;
use rand_distr::Uniform;
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;
use std::sync::mpsc;

/// Ensemble generation result containing the generated birds and metadata
/// This is the unified structure used by both ensemble generation and IO persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnsembleResult {
    /// Unique identifier for this ensemble
    pub id: usize,
    /// Tag name for the ensemble (used for file naming, batch processing)
    pub tag: String,
    /// Generated birds
    pub birds: Vec<Bird>,
    /// Generation parameters for reference
    pub params: EnsembleGenerationParams,
    /// Timestamp when ensemble was created
    pub created_at: u64,
}

/// Parameters used for ensemble generation
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct EnsembleGenerationParams {
    pub n_particles: usize,
    pub radius: f64,
    pub speed: f64,
    pub min_distance: f64,
}

/// Request for ensemble generation containing all necessary parameters
#[derive(Debug, Clone)]
pub struct EnsembleGenerationRequest {
    /// Unique identifier for this ensemble
    pub id: usize,
    /// Tag name for the ensemble (used for file naming)
    pub tag: String,
    /// Generation parameters
    pub params: EnsembleGenerationParams,
}

/// Unit tests for the ensemble module
pub mod tests;

/// Generates an ensemble of N birds uniformly distributed on a spherical surface.
///
/// This function creates a specified number of birds positioned on a sphere using rejection
/// sampling to ensure minimum distance constraints. The birds are generated with uniform
/// random spherical coordinates and transmitted via MPSC channel for further processing.
///
/// # Algorithm Details
///
/// 1. **Uniform Spherical Distribution**:
///    - φ (azimuthal) ~ Uniform[0, 2π]
///    - α (velocity direction) ~ Uniform[0, 2π]
///    - cos(θ) ~ Uniform[-1, 1], then θ = arccos(cos(θ))
///
/// 2. **Bird Creation**: Uses `Bird::from_spherical(radius, θ, φ, speed, α)`
///
/// 3. **Collision Detection**: Checks geodesic distance using `Bird::distance_from`
///
/// 4. **Rejection Sampling**: Discards birds closer than `min_distance` to existing birds
///
/// # Arguments
///
/// * `request` - Ensemble generation request containing all parameters and metadata
/// * `tx` - MPSC sender channel for transmitting the completed ensemble result
///
/// # Returns
///
/// * `Ok(())` - Successfully generated and transmitted ensemble
/// * `Err(String)` - Error during generation or transmission with descriptive message
///
/// # Performance Considerations
///
/// - Time complexity depends on `min_distance`: smaller values may require many rejection iterations
/// - Memory pre-allocation uses `Vec::with_capacity(n_particles)` for efficiency
/// - Distance calculations are O(n) for each candidate bird, making overall complexity O(n²) in worst case
///
/// # Thread Safety
///
/// This function is designed to run in a separate thread and communicates results via MPSC channels.
/// All random number generation uses thread-local RNG for safety. The ensemble ID and tag ensure
/// proper identification when multiple threads are generating ensembles concurrently.
///
/// # Examples
///
/// ```rust
/// use std::sync::mpsc;
/// use flocking_lib::ensemble::{self, EnsembleGenerationRequest, EnsembleGenerationParams};
///
/// let (tx, rx) = mpsc::channel();
///
/// let request = EnsembleGenerationRequest {
///     id: 0,
///     tag: "sparse".to_string(),
///     params: EnsembleGenerationParams {
///         n_particles: 100,
///         radius: 1.0,
///         speed: 1.5,
///         min_distance: 0.2,
///     },
/// };
///
/// ensemble::generate(request, tx).unwrap();
/// let result = rx.recv().unwrap();
/// println!("Generated ensemble '{}' with {} birds", result.tag, result.birds.len());
/// ```
pub fn generate(
    request: EnsembleGenerationRequest,
    tx: mpsc::Sender<EnsembleResult>,
) -> Result<(), String> {
    let mut rng = rand::rng();
    let mut birds = Vec::with_capacity(request.params.n_particles);

    while birds.len() < request.params.n_particles {
        let angle_distribution = Uniform::new(0.0, 2.0 * PI).unwrap();
        let cos_distribution = Uniform::new(-1.0, 1.0).unwrap();
        // Generate uniform random spherical coordinates
        let phi = angle_distribution.sample(&mut rng); // azimuthal angle [0, 2π]
        let alpha = angle_distribution.sample(&mut rng); // velocity direction [0, 2π]
        let cos_theta: f64 = cos_distribution.sample(&mut rng); // uniform cos(θ) [-1, 1]
        let theta = cos_theta.acos(); // polar angle [0, π]

        // Create new bird from spherical coordinates
        let candidate_bird = Bird::from_spherical(
            request.params.radius,
            theta,
            phi,
            request.params.speed,
            alpha,
        );

        // Check if this bird is too close to any existing bird
        let too_close = birds.iter().any(|existing_bird| {
            candidate_bird.distance_from(existing_bird, request.params.radius)
                < request.params.min_distance
        });

        // If not too close, add to ensemble
        if !too_close {
            birds.push(candidate_bird);
        }
    }

    // Create the ensemble result with metadata (timestamps will be added by IO module)
    let result = EnsembleResult {
        id: request.id,
        tag: request.tag,
        birds,
        params: request.params,
        created_at: 0, // Will be set by IO module
    };

    // Send the complete ensemble result via MPSC to IO
    tx.send(result).map_err(|e| e.to_string())?;

    Ok(())
}
