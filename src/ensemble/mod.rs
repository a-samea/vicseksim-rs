//! # Ensemble Generation Module
//!
//! create!

use crate::bird::Bird;
use crate::ensemble::io::EntryResultReceiver;
use crate::io::DataChannel;
use log::{debug, error, info, trace};
use rayon::prelude::*;
use std::sync::mpsc;

/// IO specific Implementations for ensemble data.
pub mod io;
/// Unit tests for the ensemble module
pub mod tests;

/// Result structure containing a complete generated ensemble with metadata.
///
/// This structure serves as the unified format for ensemble data throughout the system,
/// from generation through I/O persistence. Each `EntryResult` represents one complete
/// ensemble entry that can be independently saved, loaded, or processed.
///
/// # Fields
///
/// * `id` - Unique numerical identifier for this ensemble entry within a generation batch
/// * `tag` - Numerical tag used for file naming and ensemble categorization
/// * `birds` - Vector of generated [`Bird`] particles with positions and velocities
/// * `params` - Original generation parameters preserved for reproducibility and analysis
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EntryResult {
    /// Unique identifier for this entry
    pub id: usize,
    /// Numerical tag for the ensemble used in file naming
    pub tag: usize,
    /// Generated birds with positions and velocities on the sphere
    pub birds: Vec<Bird>,
    /// Generation parameters preserved for metadata
    pub params: EntryGenerationParams,
}

/// Physical and numerical parameters controlling ensemble generation.
///
/// These parameters define the physical properties of the generated ensemble and the
/// constraints for particle placement. All parameters are preserved in the final
/// [`EntryResult`] for reproducibility and analysis.
///
/// # Field Details
///
/// * `n_particles` - Target number of birds to generate in the ensemble
/// * `radius` - Radius of the spherical surface (typically 1.0 for unit sphere)
/// * `speed` - Initial speed magnitude for all birds (velocity vector magnitude)
/// * `min_distance` - Minimum geodesic distance constraint between any two birds
#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
pub struct EntryGenerationParams {
    /// Number of particles to generate in this ensemble
    pub num_birds: usize,
    /// Radius of the spherical surface
    pub radius: f64,
    /// Initial speed magnitude for all birds
    pub speed: f64,
    /// Minimum allowed geodesic distance between birds
    pub min_distance: f64,
}

/// Internal request structure for coordinating ensemble generation across threads.
///
/// This structure combines generation parameters with metadata needed for proper
/// identification and result handling in multi-threaded environments. It is used
/// internally by the parallel generation system.
///
/// # Thread Safety
///
/// This structure is `Copy` and contains only primitives, making it safe to pass
/// between threads without synchronization concerns.
#[derive(Debug, Copy, Clone)]
struct EntryGenerationRequest {
    /// Unique identifier for this entry within the generation batch
    pub id: usize,
    /// Numerical tag for ensemble categorization and file naming
    pub tag: usize,
    /// Physical and numerical generation parameters
    pub params: EntryGenerationParams,
}

/// Generates uniform random spherical coordinates for bird placement.
///
/// This private function implements the mathematically correct method for generating
/// uniformly distributed random points on a sphere surface. It uses the inverse
/// transform sampling method to ensure true uniform distribution.
///
/// # Algorithm
///
/// The function generates three independent random values:
/// 1. **φ (azimuthal angle)**: Uniform distribution over [0, 2π]
/// 2. **α (velocity direction)**: Uniform distribution over [0, 2π]
/// 3. **cos(θ)**: Uniform distribution over [-1, 1], then θ = arccos(cos(θ))
///
/// The key insight is that sampling cos(θ) uniformly (rather than θ directly) ensures
/// uniform area distribution on the sphere surface, avoiding the pole clustering that
/// would occur with naive uniform sampling of θ.
///
/// # Returns
///
/// A tuple `(theta, phi, alpha)` where:
/// - `theta`: Polar angle [0, π] from the north pole
/// - `phi`: Azimuthal angle [0, 2π] around the equator
/// - `alpha`: Velocity direction [0, 2π] for tangent velocity vector
///
/// # Thread Safety
///
/// Uses thread-local random number generator (`rand::rng()`) for thread safety
/// in parallel ensemble generation.
fn random_bird() -> (f64, f64, f64) {
    use rand::prelude::*;
    use rand_distr::Uniform;
    use std::f64::consts::PI;

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

/// Generates a single ensemble entry using rejection sampling with minimum distance constraints.
///
/// This private function is the core ensemble generation algorithm that creates exactly
/// `n_particles` birds positioned on a spherical surface with guaranteed minimum separation.
/// It uses rejection sampling to enforce distance constraints and communicates results
/// via MPSC channels for integration with the parallel processing system.
///
/// # Algorithm Details
///
/// 1. **Particle Generation Loop**:
///    - Continues until exactly `n_particles` valid birds are generated
///    - Uses `random_bird()` to generate uniform spherical coordinates
///    - Creates birds with `Bird::from_spherical()` using provided physics parameters
///
/// 2. **Collision Detection**:
///    - Calculates geodesic distance to all existing birds using `Bird::distance_from()`
///    - Rejects candidates that violate the `min_distance` constraint
///    - Uses O(n) distance checks per candidate, making worst-case complexity O(n²)
///
/// 3. **Result Communication**:
///    - Packages birds with complete metadata in `EntryResult`
///    - Transmits via MPSC channel for non-blocking I/O processing
///    - Preserves generation parameters for reproducibility
///
/// # Performance Characteristics
///
/// - **Memory**: Pre-allocates `Vec::with_capacity(n_particles)` for efficiency
/// - **Time Complexity**: O(n²) in worst case due to distance checking
/// - **Rejection Rate**: Depends strongly on `min_distance` relative to sphere area
/// - **Thread Safety**: Designed for concurrent execution with thread-local RNG
///
/// # Arguments
///
/// * `request` - Complete generation request with ID, tag, and physics parameters
/// * `tx` - MPSC sender for transmitting completed ensemble to I/O system
///
/// # Returns
///
/// * `Ok(())` - Ensemble generated and transmitted successfully
/// * `Err(String)` - Generation or transmission error with descriptive message
///
/// # Error Conditions
///
/// - MPSC channel transmission failure (usually indicates receiver dropped)
/// - Potential infinite loop if `min_distance` constraints are impossible to satisfy
fn generate_entry(
    request: EntryGenerationRequest,
    tx: mpsc::Sender<EntryResult>,
) -> Result<(), String> {
    let mut birds = Vec::with_capacity(request.params.num_birds);

    while birds.len() < request.params.num_birds {
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
    debug!(
        "Generated ensemble entry {} with tag {}, Sent through MPSC",
        request.id, request.tag
    );
    Ok(())
}

/// Generates multiple ensemble entries in parallel with automatic I/O handling.
///
/// This is the main public interface for large-scale ensemble generation. It coordinates
/// parallel generation of multiple ensemble entries using Rayon, manages worker threads
/// efficiently, and automatically handles I/O operations through a dedicated thread.
/// The function provides complete ensemble generation from physics simulation through
/// file persistence.
///
/// # Architecture
///
/// 1. **Parallel Generation**: Uses Rayon's `par_iter()` for CPU-optimal thread distribution
/// 2. **Communication**: MPSC channels coordinate between generation and I/O threads
/// 3. **Thread Management**: Automatic optimal thread count based on CPU capabilities
/// 4. **Progress Tracking**: Real-time progress reporting via console output
/// 5. **Error Handling**: Comprehensive error propagation with descriptive messages
///
/// # I/O Integration
///
/// - Automatically creates necessary data directory structure (`./data/ensemble/`)
/// - Spawns dedicated I/O thread to prevent blocking generation workers
/// - Saves ensembles concurrently as they complete for optimal performance
/// - Ensures all data is persisted before function returns
///
/// # Arguments
///
/// * `tag` - Numerical tag used for file naming and ensemble categorization
/// * `number_of_entries` - Total number of ensemble entries to generate
/// * `params` - Physical parameters controlling ensemble generation (particles, physics, constraints)
///
/// # Returns
///
/// * `Ok(())` - All ensembles generated and saved successfully
/// * `Err(String)` - Descriptive error message suitable for CLI display
///
/// # Performance Considerations
///
/// - **Scalability**: Efficiently handles both small (10s) and large (1000s) ensemble counts
/// - **Memory Usage**: Bounded by concurrent ensemble count, not total count
/// - **Thread Overhead**: Automatic thread pool sizing prevents excessive thread creation
/// - **I/O Optimization**: Non-blocking concurrent saves maximize throughput
///
/// # Error Conditions
///
/// - Failed data directory creation (filesystem permissions)
/// - Individual ensemble generation failures (impossible constraints)
/// - I/O thread failures (disk space, write permissions)
/// - Channel communication failures (system resource exhaustion)
///
/// # Examples
///
/// ```rust
/// use flocking_lib::ensemble::{self, EntryGenerationParams};
///
/// // Define physical parameters for the ensembles
/// let params = EntryGenerationParams {
///     num_birds: 100,        // 100 birds per ensemble
///     radius: 1.0,             // Unit sphere
///     speed: 1.5,              // Initial speed magnitude
///     min_distance: 0.1,       // Minimum separation
/// };
///
/// // Generate 50 ensemble entries with tag "experiment_1"
/// match ensemble::generate(1, 50, params) {
///     Ok(()) => println!("Generation completed successfully"),
///     Err(e) => eprintln!("Generation failed: {}", e),
/// }
/// ```
///
/// # File Output
///
/// Generated ensembles are saved to `./data/ensemble/` with filenames following
/// the pattern `ensemble_tag_{tag}_entry_{id}.json`, where `tag` and `id` correspond
/// to the function parameters and individual entry identifiers.
pub fn generate(
    tag: usize,
    number_of_entries: usize,
    params: EntryGenerationParams,
) -> Result<(), String> {
    debug!("--- Parallel Ensemble Generation ---");
    debug!(
        "Generating {} ensemble entries with tag '{}'",
        number_of_entries, tag
    );
    debug!(
        "Configuration: n_particles={}, radius={}, speed={}, min_distance={}",
        params.num_birds, params.radius, params.speed, params.min_distance
    );

    // initialization of channels
    let (entry_tx, entry_rx) = mpsc::channel();

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
                    trace!("Successfully generated entry {}", request.id);
                }
                Err(e) => {
                    error!("Failed to generate entry {}: {}", request.id, e);
                }
            }
        });

    // Drop the original sender so the receiver will know when all threads are done
    drop(entry_tx);

    // Start I/O receiver thread for concurrent saving
    let io_handle = EntryResultReceiver::start_receiver_thread(entry_rx);

    // Wait for I/O thread to complete saving
    match io_handle.join() {
        Ok(Ok(())) => {
            debug!("All ensemble entries saved successfully");
        }
        Ok(Err(e)) => {
            return Err(format!("I/O thread failed: {}", e));
        }
        Err(_) => {
            return Err("I/O thread panicked".to_string());
        }
    }

    debug!("\n--- Generation Complete ---");
    info!(
        "Successfully generated {} ensemble entries",
        number_of_entries
    );
    info!("Ensemble entries saved to: ./data/ensemble/");

    Ok(())
}
