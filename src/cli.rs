//! # CLI Module - Command Line Interface for Flocking Simulation Pipeline
//!
//! This module provides a command-line interface for the three-stage flocking simulation pipeline:
//!
//! ## Stage 1: Ensemble Generation (`generate-ensembles`)
//!
//! Creates multiple ensembles of birds in parallel using configurable threading. Each thread
//! generates its own ensemble independently and saves it to a separate file in `./data/ensemble/`.
//! This stage supports:
//! - Parallel ensemble generation using multiple threads
//! - Configurable ensemble size (number of ensembles to create)
//! - Configurable birds per ensemble
//! - Automatic file naming and organization
//! - Performance timing and reporting
//!
//! **Usage**:
//! ```bash
//! flok generate-ensembles --ensemble-count 10 --birds-per-ensemble 500 --threads 4
//! ```
//!
//! ## Stage 2: Main Simulation (`simulate`)
//!
//! Runs the main flocking simulation using pre-generated ensembles as initial conditions.
//! Processes multiple ensemble files and generates simulation trajectories for visualization
//! and analysis.
//!
//! **Usage**:
//! ```bash
//! flok simulate --input-dir ./data/ensemble/ --steps 2000 --output-dir ./data/simulation/
//! ```
//!
//! ## Stage 3: Post Analysis (`analyze`)
//!
//! Performs statistical analysis and visualization of simulation results. Generates
//! plots, statistical summaries, and other analytical outputs from simulation data.
//!
//! **Usage**:
//! ```bash
//! flok analyze --input-dir ./data/simulation/ --output-dir ./data/analysis/
//! ```
//!
//! ## Design Philosophy
//!
//! The CLI is designed around a pipeline approach where each stage operates on the outputs
//! of the previous stage. This enables:
//! - Reproducible research workflows
//! - Parallel processing at each stage
//! - Intermediate result inspection and debugging
//! - Flexible parameter exploration
//!
//! Each command includes timing information and progress reporting to help users
//! understand performance characteristics and optimize their workflows.

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Stage 1: Generate multiple ensembles in parallel for simulation input
    GenerateEnsembles {
        /// Number of ensembles to generate
        #[arg(short = 'e', long, default_value_t = 10)]
        ensemble_count: usize,

        /// Number of birds per ensemble
        #[arg(short = 'b', long, default_value_t = 500)]
        birds_per_ensemble: usize,

        /// Number of threads to use for parallel generation
        #[arg(short, long, default_value_t = 4)]
        threads: usize,

        /// Sphere radius for bird positioning
        #[arg(short, long, default_value_t = 1.0)]
        radius: f64,

        /// Speed of all birds in the ensemble
        #[arg(short, long, default_value_t = 2.0)]
        speed: f64,

        /// Minimum allowed distance between birds
        #[arg(short = 'd', long, default_value_t = 0.1)]
        min_distance: f64,

        /// Base tag for ensemble naming (will be appended with thread ID)
        #[arg(long, default_value = "ensemble")]
        tag: String,
    },

    /// Stage 2: Run simulations from generated ensembles
    Simulate {
        /// Directory containing ensemble files to simulate
        #[arg(short, long, default_value = "./data/ensemble/")]
        input_dir: PathBuf,

        /// Directory to save simulation results
        #[arg(short, long, default_value = "./data/simulation/")]
        output_dir: PathBuf,

        /// Total number of time steps to run
        #[arg(short, long, default_value_t = 2000)]
        steps: u64,

        /// Interaction radius for alignment
        #[arg(long, default_value_t = 1.0)]
        interaction_radius: f64,

        /// Noise parameter for random motion
        #[arg(long, default_value_t = 0.1)]
        noise: f64,
    },

    /// Stage 3: Analyze simulation results and generate visualizations
    Analyze {
        /// Directory containing simulation result files
        #[arg(short, long, default_value = "./data/simulation/")]
        input_dir: PathBuf,

        /// Directory to save analysis results and plots
        #[arg(short, long, default_value = "./data/analysis/")]
        output_dir: PathBuf,

        /// Types of analysis to perform
        #[arg(long, value_delimiter = ',', default_values = ["order", "clustering", "trajectory"])]
        analysis_types: Vec<String>,
    },
}
