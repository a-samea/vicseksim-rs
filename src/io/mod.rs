//! # IO Module - Data Persistence and Pipeline Management
//!
//! This module provides data persistence functionality for the flocking simulation pipeline.
//! It handles saving and loading of ensemble data, simulation results, and analysis outputs
//! to support the three-stage pipeline:
//!
//! 1. **Ensemble Generation** → Save to `./data/ensemble/`
//! 2. **Simulation Execution** → Save to `./data/simulation/`
//! 3. **Analysis Processing** → Save to `./data/analysis/` (future implementation)
//!
//! All data is stored in the `./data/` directory with organized subdirectories for each
//! pipeline stage.

pub mod analysis;
// Completed integration
pub mod ensemble;
//pub mod simulation;

use std::fs;

/// Ensures the data directory structure exists
pub fn ensure_data_directories() -> Result<(), std::io::Error> {
    fs::create_dir_all("./data/ensemble")?;
    fs::create_dir_all("./data/simulation")?;
    fs::create_dir_all("./data/analysis")?;
    Ok(())
}
