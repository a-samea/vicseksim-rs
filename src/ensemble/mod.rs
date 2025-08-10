//! # Ensemble Module - Initial Particle Distribution Generation
//!
//! This module provides functionality for generating initial ensembles of birds (particles)
//! distributed uniformly on a spherical surface for flocking simulations. The module implements
//! a rejection sampling algorithm to ensure proper spatial separation between particles while
//! maintaining uniform distribution on the sphere.
//!
//! ## Core Functionality
//!
//! The primary function `generate` creates N particles positioned on a sphere of radius R,
//! each with speed S, using spherical coordinates with uniform random distributions:
//!
//! - **Azimuthal angle (φ)**: Uniform distribution [0, 2π]
//! - **Velocity direction (α)**: Uniform distribution [0, 2π]
//! - **Polar angle (θ)**: Derived from uniform cos(θ) ∈ [-1, 1] to ensure uniform surface distribution
//!
//! ## Collision Avoidance
//!
//! The module implements a rejection sampling strategy to maintain minimum distance constraints
//! between particles. When a new particle is generated too close to existing particles, it is
//! discarded and a new one is generated until the minimum distance requirement is satisfied.
//!
//! ## Thread Communication
//!
//! The generated ensemble is transmitted via MPSC (Multi-Producer, Single-Consumer) channels
//! to separate IO handling, enabling asynchronous processing and file operations. Each ensemble
//! includes identification metadata (ID and tag) to ensure proper tracking in multithreaded
//! environments where multiple ensembles are generated concurrently.
//!
//! ## Multithreaded Design
//!
//! The function signature is optimized for multithreaded ensemble generation:
//! - Uses structured request/result pattern for clear data flow
//! - Includes unique IDs to prevent mixing of ensemble data
//! - Maintains all generation parameters in the result for traceability
//!
//! ## Usage Example
//!
//! ```rust
//! use std::sync::mpsc;
//! use flocking_lib::ensemble::{self, EnsembleGenerationRequest, EnsembleGenerationParams};
//!
//! let (tx, rx) = mpsc::channel();
//!
//! let request = EnsembleGenerationRequest {
//!     id: 0,
//!     tag: "test_ensemble".to_string(),
//!     params: EnsembleGenerationParams {
//!         n_particles: 1000,
//!         radius: 1.0,
//!         speed: 2.0,
//!         min_distance: 0.1,
//!     },
//! };
//!
//! // Generate ensemble in a separate thread
//! ensemble::generate(request, tx).unwrap();
//!
//! // Receive the generated ensemble with metadata
//! let result = rx.recv().unwrap();
//! println!("Generated ensemble '{}' with {} birds", result.tag, result.birds.len());
//! ```

use crate::bird::Bird;
use rand::prelude::*;
use rand_distr::Uniform;
use std::f64::consts::PI;
use std::sync::mpsc;

pub mod tests;

/// Ensemble generation result containing the generated birds and metadata
#[derive(Debug, Clone)]
pub struct EnsembleResult {
    /// Unique identifier for this ensemble
    pub id: usize,
    /// Tag name for the ensemble (used for file naming)
    pub tag: String,
    /// Generated birds
    pub birds: Vec<Bird>,
    /// Generation parameters for reference
    pub params: EnsembleGenerationParams,
}

/// Parameters used for ensemble generation
#[derive(Debug, Clone)]
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
/// use flocking_lib::ensemble::{EnsembleGenerationRequest, EnsembleGenerationParams};
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
        let candidate_bird = Bird::from_spherical(request.params.radius, theta, phi, request.params.speed, alpha);

        // Check if this bird is too close to any existing bird
        let too_close = birds.iter().any(|existing_bird| {
            candidate_bird.distance_from(existing_bird, request.params.radius) < request.params.min_distance
        });

        // If not too close, add to ensemble
        if !too_close {
            birds.push(candidate_bird);
        }
    }

    // Create the ensemble result with metadata
    let result = EnsembleResult {
        id: request.id,
        tag: request.tag,
        birds,
        params: request.params,
    };

    // Send the complete ensemble result via MPSC to IO
    tx.send(result).map_err(|e| e.to_string())?;

    Ok(())
}
