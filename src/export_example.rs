// Example of how to export simulation data to JSON for Python visualization
// Add this to your main.rs or create a separate export example

use flocking_lib::io::simulation::export_to_json;
use std::path::Path;

/// Example function showing how to export simulation data to JSON
/// 
/// This function demonstrates the process of:
/// 1. Running a simulation (you already have this)
/// 2. Exporting the results to JSON format
/// 3. Making the data available for Python visualization
/// 
/// # Usage
/// 
/// After running your simulation and saving it with a tag and ID,
/// call this function to export the data:
/// 
/// ```rust
/// export_simulation_for_python("my_experiment", &1)?;
/// ```
pub fn export_simulation_for_python(tag: &str, id: &usize) -> Result<(), Box<dyn std::error::Error>> {
    // Define the output path in the data/simulation directory
    let filename = format!("{}-{}.json", tag, id);
    let output_path = Path::new("./data/simulation").join(filename);
    
    // Export the simulation data to JSON
    export_to_json(tag, id, &output_path)?;
    
    println!("Simulation data exported to: {}", output_path.display());
    println!("You can now visualize this data using the Python scripts in ./plots/");
    println!("Run: python plots/simple_example.py {}", output_path.display());
    
    Ok(())
}

/// Example of exporting multiple simulations at once
pub fn export_all_simulations_with_tag(tag: &str) -> Result<(), Box<dyn std::error::Error>> {
    use flocking_lib::io::simulation::list_simulation_tags_and_ids;
    
    // Get all simulations with the specified tag
    let all_simulations = list_simulation_tags_and_ids()?;
    let matching_simulations: Vec<usize> = all_simulations
        .into_iter()
        .filter_map(|(sim_tag, id)| {
            if sim_tag == tag {
                Some(id)
            } else {
                None
            }
        })
        .collect();
    
    if matching_simulations.is_empty() {
        println!("No simulations found with tag: {}", tag);
        return Ok(());
    }
    
    println!("Found {} simulations with tag '{}'", matching_simulations.len(), tag);
    
    // Export each simulation
    for id in matching_simulations {
        match export_simulation_for_python(tag, &id) {
            Ok(()) => println!("✓ Exported simulation {}-{}", tag, id),
            Err(e) => println!("✗ Failed to export simulation {}-{}: {}", tag, id, e),
        }
    }
    
    Ok(())
}

#[cfg(feature = "export_example")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Example usage - replace with your actual simulation tag and ID
    
    // Export a single simulation
    export_simulation_for_python("test", &1)?;
    
    // Or export all simulations with a specific tag
    export_all_simulations_with_tag("experiment_1")?;
    
    Ok(())
}
