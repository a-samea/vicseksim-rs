//! # Flocking Simulation Library
//!
//! A high-performance Rust library for simulating collective motion and flocking behavior
//! on spherical surfaces using the Vicsek model. This library provides a complete framework
//! for studying statistical physics of flocking, including phase transitions, cluster formation,
//! and non-equilibrium dynamics.
//!
//! ## Overview
//!
//! The library implements a sophisticated particle-based simulation where "birds" (particles)
//! move on the surface of a sphere, following local alignment rules with configurable noise.
//! This approach avoids boundary effects and provides a topologically closed environment
//! ideal for studying collective behavior.
//!
//! ## Key Features
//!
//! - **Spherical Geometry**: Particles constrained to sphere surface with geodesic motion
//! - **Robust Physics**: 3D Cartesian vectors avoid pole singularities
//! - **High Performance**: Parallel processing with `rayon` and double-buffered updates
//! - **Flexible Noise Models**: Configurable noise distributions for phase transition studies
//! - **Collision Avoidance**: Short-range repulsive forces prevent particle overlap
//! - **Ensemble Generation**: Tools for large-scale statistical studies
//! - **Analysis Tools**: Built-in order parameters and cluster detection
//!
//! ## Module Organization
//!
//! The library is organized into focused modules:
//!
//! - [`bird`]: Core particle representation and physics
//! - [`vector`]: 3D vector mathematics optimized for performance
//! - [`simulation`]: High-performance simulation engine with parallel processing
//! - [`ensemble`]: Initial condition generation with uniform sphere distribution
//! - [`analysis`]: Order parameters, clustering, and statistical analysis
//! - [`io`]: Serialization and data persistence in multiple formats
//! - [`cli`]: Command-line interface definitions (for binary usage)
//!
//! ## Physics Implementation
//!
//! The simulation implements several key physical concepts:
//!
//! ### Spherical Constraint
//! Particles are constrained to move on the surface of a sphere using:
//! - Geodesic motion for position updates
//! - Parallel transport for velocity comparisons
//! - Proper spherical geometry to avoid coordinate singularities
//!
//! ### Alignment Mechanism
//! Birds align their velocities with nearby neighbors using:
//! - Distance-based neighbor detection
//! - Weighted velocity averaging
//! - Configurable alignment strength
//!
//! ### Noise Models
//! Multiple noise models support phase transition studies:
//! - Uniform angular noise
//! - Gaussian noise
//! - Configurable noise strength
//!
//! ## Performance Characteristics
//!
//! The library is designed for high-performance simulations:
//! - **Parallel Processing**: Automatic CPU core utilization via `rayon`
//! - **Memory Efficiency**: Double-buffered updates minimize allocations
//! - **SIMD Optimization**: Vector operations leverage CPU SIMD instructions
//! - **Cache Locality**: Data structures optimize memory access patterns
//!
//! ## Error Handling
//!
//! The library uses Rust's `Result` type for error handling. Common error scenarios:
//! - Ensemble generation failures (overcrowding, invalid parameters)
//! - I/O errors during data persistence
//! - Invalid simulation parameters
//!
//! ## Thread Safety
//!
//! Most types in this library are `Send` but not `Sync`, designed for owned usage
//! in parallel contexts via `rayon`. The simulation engine handles thread safety
//! internally for parallel bird updates.

// Declare all modules to make them part of the library
pub mod bird;
pub mod ensemble;
pub mod io;
pub mod simulation;
pub mod vector;

pub mod analysis;
pub mod cli;
