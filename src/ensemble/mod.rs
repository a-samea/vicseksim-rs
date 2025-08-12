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
pub struct EnsembleEntryResult {
    /// Unique identifier for this entry
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
pub struct EnsembleEntryGenerationRequest {
    /// Unique identifier for this entry
    pub id: usize,
    /// Tag name for the ensemble (used for file naming)
    pub tag: String,
    /// Generation parameters
    pub params: EnsembleGenerationParams,
}

/// Unit tests for the ensemble module
pub mod tests;

/// Generates an ensemble entry of N birds uniformly distributed on a spherical surface.
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
pub fn generate_entry(
    request: EnsembleEntryGenerationRequest,
    tx: mpsc::Sender<EnsembleEntryResult>,
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
    let result = EnsembleEntryResult {
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


/// Generates multiple ensemble entries in parallel with intelligent thread management
///
/// This function creates M ensemble entries using a maximum of N parallel threads,
/// where the thread count is intelligently managed based on CPU capabilities and
/// the requested parallel_threads parameter. It uses a thread pool approach to
/// avoid creating excessive threads for large ensemble counts.
///
/// # Algorithm
///
/// 1. **Thread Management**: Uses `std::cmp::min(parallel_threads, available_parallelism)`
/// 2. **Work Distribution**: Distributes ensemble generation across worker threads
/// 3. **Concurrent I/O**: Uses a dedicated I/O thread for saving completed ensembles
/// 4. **Progress Reporting**: Provides real-time progress updates via CLI output
///
/// # Arguments
///
/// * `tag` - Base tag name for ensemble file naming (will be suffixed with entry IDs)
/// * `number_of_entries` - Total number of ensemble entries to generate (M)
/// * `parallel_threads` - Maximum number of threads to use for parallel generation
/// * `params` - Ensemble generation parameters (particle count, physics, etc.)
///
/// # Returns
///
/// * `Ok(())` - All ensemble entries generated and saved successfully
/// * `Err(String)` - Error with descriptive message suitable for CLI display
///
/// # Thread Safety & Performance
///
/// - Automatically determines optimal thread count based on CPU capabilities
/// - Uses thread pool pattern to avoid excessive thread creation overhead
/// - Employs MPSC channels for lock-free communication between threads
/// - I/O operations are handled by dedicated thread to prevent blocking generation
///
/// # Examples
///
/// ```rust
/// use flocking_lib::ensemble::{self, EnsembleGenerationParams};
///
/// let params = EnsembleGenerationParams {
///     n_particles: 100,
///     radius: 1.0,
///     speed: 1.5,
///     min_distance: 0.1,
/// };
///
/// // Generate 50 ensembles using up to 8 threads
/// ensemble::generate("experiment".to_string(), 50, 8, params)?;
/// ```
pub fn generate(
    tag: String,
    number_of_entries: usize, 
    parallel_threads: usize, 
    params: EnsembleGenerationParams
) -> Result<(), String> {
    use std::time::Instant;
    
    println!("--- Parallel Ensemble Generation ---");
    println!("Generating {} ensemble entries with tag '{}'", number_of_entries, tag);
    
    // Intelligently determine the optimal number of threads
    let available_parallelism = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4); // Fallback to 4 if detection fails
    
    let effective_threads = std::cmp::min(parallel_threads, available_parallelism);
    let effective_threads = std::cmp::min(effective_threads, number_of_entries); // Don't use more threads than entries
    
    println!("Using {} threads (requested: {}, available: {}, entries: {})", 
             effective_threads, parallel_threads, available_parallelism, number_of_entries);
    
    println!("Configuration: n_particles={}, radius={}, speed={}, min_distance={}", 
             params.n_particles, params.radius, params.speed, params.min_distance);

    // Ensure data directories exist
    crate::io::ensure_data_directories()
        .map_err(|e| format!("Failed to create data directories: {}", e))?;

    let start_time = Instant::now();

    // Create channels for ensemble generation and I/O
    let (ensemble_tx, ensemble_rx) = mpsc::channel();
    let (io_tx, io_rx) = mpsc::channel();

    // Start I/O receiver thread for concurrent saving
    let io_handle = crate::io::ensemble::start_receiver_thread(io_rx);

    // Create worker threads with work distribution
    let mut handles = Vec::new();
    let entries_per_thread = (number_of_entries + effective_threads - 1) / effective_threads; // Ceiling division

    for thread_id in 0..effective_threads {
        let start_entry = thread_id * entries_per_thread;
        let end_entry = std::cmp::min(start_entry + entries_per_thread, number_of_entries);

        if start_entry >= number_of_entries {
            break; // No more work for this thread
        }

        let tx = ensemble_tx.clone();
        let thread_tag = tag.clone();
        let thread_params = params;

        let handle = std::thread::spawn(move || {
            println!("Thread {} starting: generating entries {} to {}", 
                     thread_id, start_entry, end_entry - 1);

            for entry_id in start_entry..end_entry {

                // Create the ensemble generation request
                let request = EnsembleEntryGenerationRequest {
                    id: entry_id,
                    tag: thread_tag.clone(),
                    params: thread_params,
                };

                // Generate the ensemble entry
                match generate_entry(request, tx.clone()) {
                    Ok(()) => {
                        println!("Thread {}: Generated ensemble entry {} ({})", 
                                 thread_id, entry_id, thread_tag);
                    }
                    Err(e) => {
                        eprintln!("Thread {}: Failed to generate entry {}: {}", 
                                  thread_id, entry_id, e);
                        return Err(format!("Thread {}: Generation failed for entry {}: {}", 
                                           thread_id, entry_id, e));
                    }
                }
            }

            println!("Thread {} completed successfully", thread_id);
            Ok::<(), String>(())
        });

        handles.push(handle);
    }

    // Drop the original sender so the receiver will know when all threads are done
    drop(ensemble_tx);

    // Collect and forward ensembles for I/O as they complete
    let mut completed_count = 0;
    while let Ok(ensemble_result) = ensemble_rx.recv() {
        // Send ensemble to I/O thread for saving
        if let Err(e) = io_tx.send(ensemble_result.clone()) {
            return Err(format!("Failed to send ensemble for saving: {}", e));
        }

        completed_count += 1;
        println!("Submitted ensemble {} for saving ({}/{} completed)", 
                 ensemble_result.tag, completed_count, number_of_entries);
    }

    // Drop I/O sender to signal completion
    drop(io_tx);

    // Wait for all generation threads to complete and check for errors
    for (thread_id, handle) in handles.into_iter().enumerate() {
        match handle.join() {
            Ok(Ok(())) => {
                // Thread completed successfully
            }
            Ok(Err(e)) => {
                return Err(format!("Generation thread {} failed: {}", thread_id, e));
            }
            Err(_) => {
                return Err(format!("Generation thread {} panicked", thread_id));
            }
        }
    }

    // Wait for I/O thread to complete saving
    match io_handle.join() {
        Ok(Ok(())) => {
            println!("All ensemble entries saved successfully");
        }
        Ok(Err(e)) => {
            return Err(format!("I/O thread failed: {}", e));
        }
        Err(_) => {
            return Err("I/O thread panicked".to_string());
        }
    }

    let duration = start_time.elapsed();
    println!("\n--- Generation Complete ---");
    println!("Successfully generated {} ensemble entries", completed_count);
    println!("Total time: {:.2} seconds", duration.as_secs_f64());
    println!("Average time per entry: {:.3} seconds", 
             duration.as_secs_f64() / number_of_entries as f64);
    println!("Ensemble entries saved to: ./data/ensemble/");

    if completed_count != number_of_entries {
        return Err(format!("Generated {} entries but expected {}", 
                           completed_count, number_of_entries));
    }

    Ok(())
}