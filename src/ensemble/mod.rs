//! Ensemble generation module

use crate::bird::Bird;
use rayon::prelude::*;
use std::f64::consts::PI;
use std::sync::mpsc;

/// Ensemble generation result containing the generated birds and metadata
/// This is the unified structure used by both ensemble generation and IO persistence
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EntryResult {
    /// Unique identifier for this entry
    pub id: usize,
    /// Tag name for the ensemble
    pub tag: usize,
    /// Generated birds
    pub birds: Vec<Bird>,
    /// Generation parameters for reference
    pub params: EntryGenerationParams,
}

/// Parameters used for entry generation in an ensemble
#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
pub struct EntryGenerationParams {
    pub n_particles: usize,
    pub radius: f64,
    pub speed: f64,
    pub min_distance: f64,
}

/// Request for ensemble generation containing all necessary parameters
#[derive(Debug, Copy, Clone)]
pub struct EntryGenerationRequest {
    /// Unique identifier for this entry
    pub id: usize,
    /// Tag name for the ensemble
    pub tag: usize,
    /// Generation parameters
    pub params: EntryGenerationParams,
}

/// Unit tests for the ensemble module
pub mod tests;

/// Generates a random bird
fn random_bird() -> (f64, f64, f64) {
    use rand::prelude::*;
    use rand_distr::Uniform;

    let mut rng = rand::rng();
    let angle_distribution = Uniform::new(0.0, 2.0 * PI).unwrap();
    let cos_distribution = Uniform::new(-1.0, 1.0).unwrap();
    // Generate uniform random spherical coordinates
    let phi = angle_distribution.sample(&mut rng); // azimuthal angle [0, 2π]
    let alpha = angle_distribution.sample(&mut rng); // velocity direction [0, 2π]
    let cos_theta: f64 = cos_distribution.sample(&mut rng); // uniform cos(θ) [-1, 1]
    let theta = cos_theta.acos(); // polar angle [0, π]
    (theta, phi, alpha)
}

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
/// use flocking_lib::ensemble::{self, EntryGenerationRequest, EntryGenerationParams};
///
/// let (tx, rx) = mpsc::channel();
///
/// let request = EntryGenerationRequest {
///     id: 0,
///     tag: "sparse".to_string(),
///     params: EntryGenerationParams {
///         n_particles: 100,
///         radius: 1.0,
///         speed: 1.5,
///         min_distance: 0.2,
///     },
/// };
///
/// ensemble::generate_entry(request, tx).unwrap();
/// let result = rx.recv().unwrap();
/// println!("Generated ensemble '{}' with {} birds", result.tag, result.birds.len());
/// ```
fn generate_entry(
    request: EntryGenerationRequest,
    tx: mpsc::Sender<EntryResult>,
) -> Result<(), String> {
    let mut birds = Vec::with_capacity(request.params.n_particles);

    while birds.len() < request.params.n_particles {
        let (theta, phi, alpha) = random_bird();

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

    // Create the ensemble result with metadata
    let result = EntryResult {
        id: request.id,
        tag: request.tag,
        birds,
        params: request.params,
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
/// use flocking_lib::ensemble::{self, EntryGenerationParams};
///
/// let params = EntryGenerationParams {
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
    tag: usize,
    number_of_entries: usize,
    params: EntryGenerationParams,
) -> Result<(), String> {
    println!("--- Parallel Ensemble Generation ---");
    println!(
        "Generating {} ensemble entries with tag '{}'",
        number_of_entries, tag

    println!(
        "Configuration: n_particles={}, radius={}, speed={}, min_distance={}",
        params.n_particles, params.radius, params.speed, params.min_distance
    );
    );
    // rayon

    // initialization of channels
    let (entry_tx, entry_rx) = mpsc::channel();
    let (io_tx, io_rx) = mpsc::channel();

    // create work items
    let requests = (0..number_of_entries)
        .map(|id| EntryGenerationRequest { id, tag, params })
        .collect::<Vec<EntryGenerationRequest>>();

    // parallel run
    requests
        .par_iter()
        .for_each_with(entry_tx.clone(), |entry_tx, request| {
            match generate_entry(*request, entry_tx.clone()) {
                Ok(()) => {
                    println!("Successfully generated entry");
                }
                Err(e) => {
                    eprintln!("Failed to generate entry {}: {}", request.id, e);
                }
            }
        });

    // Ensure data directories exist
    crate::io::ensure_data_directories()
        .map_err(|e| format!("Failed to create data directories: {}", e))?;
    // Start I/O receiver thread for concurrent saving
    let io_handle = crate::io::ensemble::start_receiver_thread(io_rx);

    // Drop the original sender so the receiver will know when all threads are done
    drop(entry_tx);

    // Collect and forward ensembles for I/O as they complete
    let mut completed_count = 0;
    while let Ok(entry_result) = entry_rx.recv() {
        // forward the entry result to the I/O thread
        if let Err(e) = io_tx.send(entry_result.clone()) {
            return Err(format!("Failed to send entry for saving: {}", e));
        }

        completed_count += 1;
        println!(
            "Submitted entry {} for saving ({}/{} completed)",
            entry_result.id, completed_count, number_of_entries
        );
    }

    // Drop I/O sender to signal completion
    drop(io_tx);

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

    println!("\n--- Generation Complete ---");
    println!(
        "Successfully generated {} ensemble entries",
        completed_count
    );
    println!("Ensemble entries saved to: ./data/ensemble/");

    Ok(())
}
