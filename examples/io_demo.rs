use flocking_lib::simulation::SimulationResult;
use flocking_lib::ensemble::EntryResult;
use flocking_lib::io::{list_binary_files, load_binary_file};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Listing simulation binary files:");
    let sim_files = list_binary_files::<SimulationResult>()?;
    for file in &sim_files {
        println!("  {}", file.display());
    }

    println!("\nListing ensemble binary files:");
    let ensemble_files = list_binary_files::<EntryResult>()?;
    for file in &ensemble_files {
        println!("  {}", file.display());
    }

    // Example of loading a file (if any exist)
    if let Some(first_sim) = sim_files.first() {
        println!("\nLoading simulation file: {}", first_sim.display());
        let sim_result: SimulationResult = load_binary_file(first_sim)?;
        println!("Loaded simulation with {} snapshots", sim_result.snapshots.len());
    }

    if let Some(first_ensemble) = ensemble_files.first() {
        println!("\nLoading ensemble file: {}", first_ensemble.display());
        let ensemble_result: EntryResult = load_binary_file(first_ensemble)?;
        println!("Loaded ensemble with {} birds", ensemble_result.birds.len());
    }

    Ok(())
}
