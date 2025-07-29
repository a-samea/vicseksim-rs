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
//! to separate IO handling, enabling asynchronous processing and file operations.
//!
//! ## Usage Example
//!
//! ```rust
//! use std::sync::mpsc;
//! use flocking_lib::ensemble;
//!
//! let (tx, rx) = mpsc::channel();
//!
//! // Generate 1000 birds on unit sphere with speed 2.0 and minimum separation 0.1
//! ensemble::generate(1000, 1.0, 2.0, 0.1, tx).unwrap();
//!
//! // Receive the generated ensemble
//! let birds = rx.recv().unwrap();
//! ```

use crate::bird::Bird;
use rand::prelude::*;
use rand_distr::Uniform;
use std::f64::consts::PI;
use std::sync::mpsc;

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
/// * `n_particles` - Number of birds to generate in the ensemble
/// * `radius` - Radius of the spherical surface on which birds are positioned
/// * `speed` - Magnitude of velocity vectors for all birds (uniform speed)
/// * `min_distance` - Minimum geodesic distance allowed between any two birds
/// * `tx` - MPSC sender channel for transmitting the completed ensemble
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
/// All random number generation uses thread-local RNG for safety.
///
/// # Examples
///
/// ```rust
/// use std::sync::mpsc;
/// use flocking_lib::ensemble;
///
/// let (tx, rx) = mpsc::channel();
///
/// // Generate sparse ensemble (large minimum distance)
/// ensemble::generate(100, 1.0, 1.5, 0.2, tx.clone()).unwrap();
/// let sparse_ensemble = rx.recv().unwrap();
///
/// // Generate dense ensemble (small minimum distance)  
/// ensemble::generate(500, 2.0, 1.0, 0.05, tx).unwrap();
/// let dense_ensemble = rx.recv().unwrap();
/// ```
pub fn generate(
    n_particles: usize,
    radius: f64,
    speed: f64,
    min_distance: f64,
    tx: mpsc::Sender<Vec<Bird>>,
) -> Result<(), String> {
    let mut rng = rand::rng();
    let mut birds = Vec::with_capacity(n_particles);

    while birds.len() < n_particles {
        let angle_distribution = Uniform::new(0.0, 2.0 * PI).unwrap();
        let cos_distribution = Uniform::new(-1.0, 1.0).unwrap();
        // Generate uniform random spherical coordinates
        let phi = angle_distribution.sample(&mut rng); // azimuthal angle [0, 2π]
        let alpha = angle_distribution.sample(&mut rng); // velocity direction [0, 2π]
        let cos_theta: f64 = cos_distribution.sample(&mut rng); // uniform cos(θ) [-1, 1]
        let theta = cos_theta.acos(); // polar angle [0, π]

        // Create new bird from spherical coordinates
        let candidate_bird = Bird::from_spherical(radius, theta, phi, speed, alpha);

        // Check if this bird is too close to any existing bird
        let too_close = birds.iter().any(|existing_bird| {
            candidate_bird.distance_from(existing_bird, radius) < min_distance
        });

        // If not too close, add to ensemble
        if !too_close {
            birds.push(candidate_bird);
        }
    }

    // Send the complete ensemble via MPSC to IO
    tx.send(birds).map_err(|e| e.to_string())?;

    Ok(())
}
