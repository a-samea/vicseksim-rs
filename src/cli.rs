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
    /// Generate a set of initial particle positions.
    Generate {
        /// Number of particles to generate.
        #[arg(short, long, default_value_t = 512)]
        num_particles: usize,

        /// Minimum allowed distance between particles (on unit sphere).
        #[arg(short, long, default_value_t = 0.1)]
        min_distance: f64,

        /// Output file path for the generated spherical coordinates.
        #[arg(short, long, default_value = "initial_coords.json")]
        output: PathBuf,
    },

    /// Run a simulation from an initial state and output data for visualization.
    Simulate {
        /// Path to the input file with initial spherical coordinates.
        #[arg(short, long)]
        input: PathBuf,

        /// Output CSV file path for visualization data.
        #[arg(short, long, default_value = "viz_data.csv")]
        output: PathBuf,

        /// Total number of time steps to run.
        #[arg(short, long, default_value_t = 2000)]
        steps: u64,

        /// Constant speed of the particles.
        #[arg(long, default_value_t = 0.03)]
        speed: f64,

        /// Interaction radius for alignment.
        #[arg(long, default_value_t = 1.0)]
        interaction_radius: f64,

        /// Noise parameter (e.g., width of uniform dist or std dev of normal dist).
        #[arg(short, long, default_value_t = 0.5)]
        noise: f64,
    },

    /// Perform analysis on saved simulation snapshots.
    Analyze {
        /// Path to a directory containing snapshot .bin files.
        #[arg(short, long)]
        snapshot_dir: PathBuf,

        /// Output file for the analysis results.
        #[arg(short, long, default_value = "analysis_results.csv")]
        output: PathBuf,
    },
}