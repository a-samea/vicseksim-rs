use clap::Parser;
use flocking_lib::cli::{Cli, Commands};
use flocking_lib::simulation::Simulation;
use std::fs::File;
use std::io::BufWriter;
use std::sync::mpsc;
use std::thread;
use std::time::Instant;

fn main() -> Result<(), String> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Generate {
            num_particles,
            min_distance,
            output,
        } => {
            println!("--- Generate Mode ---");
            println!("Generating {} particles...", num_particles);
            println!("Saving initial state to: {:?}", output);
            // Call functions from flocking_lib::ensemble and flocking_lib::io
            unimplemented!("Call ensemble generation and IO saving logic here.");
        }
        Commands::Simulate {
            input,
            output,
            steps,
            speed,
            interaction_radius,
            noise,
        } => {
            println!("--- Simulate Mode ---");
            println!("Loading initial state from: {:?}", input);
            println!("Running simulation for {} steps...", steps);
            println!("Saving visualization data to: {:?}", output);
            // 1. Load initial coords using flocking_lib::io
            // 2. Convert to particles using flocking_lib::ensemble
            // 3. Create FlockSimulation
            // 4. Loop for `steps`, calling sim.step() and io.append_frame...
            unimplemented!("Call full simulation loop logic here.");
        }
        Commands::Analyze {
            snapshot_dir,
            output,
        } => {
            println!("--- Analyze Mode ---");
            println!("Analyzing snapshots from directory: {:?}", snapshot_dir);
            println!("Saving analysis results to: {:?}", output);
            // 1. Find all snapshot files in the directory
            // 2. Loop through each file
            // 3. Load snapshot using flocking_lib::io
            // 4. Run analysis using flocking_lib::analysis
            // 5. Aggregate and save results.
            unimplemented!("Call analysis logic here.");
        }
    }

    Ok(())
}

fn main() {
    // --- Simulation Parameters ---
    let num_particles = 1_000;
    let total_steps = 200;
    let save_interval = 20; // Save data every 20 steps
    let output_filename = "simulation_output.txt";

    // --- I/O Thread Setup ---
    // 1. Create a multi-producer, single-consumer (mpsc) channel.
    let (tx, rx) = mpsc::channel::<Vec<Particle>>();

    // 2. Spawn a dedicated I/O thread.
    // The `move` keyword gives the thread ownership of `rx` and the filename.
    let io_thread_handle = thread::spawn(move || {
        println!("I/O thread started. Writing to '{}'.", output_filename);
        let file = File::create(output_filename).expect("Could not create output file.");
        let mut writer = BufWriter::new(file);

        // This loop will block until a message is received.
        // It will automatically exit when the `tx` (sender) is dropped.
        for (i, received_particles) in rx.iter().enumerate() {
            writeln!(writer, "--- Frame {} ---", i * save_interval).unwrap();
            for p in received_particles {
                writeln!(writer, "{:?}", p).unwrap();
            }
        }
        println!("I/O thread finished.");
    });

    // --- Simulation Setup and Execution ---
    let mut simulation = Simulation::new(num_particles, tx);

    println!(
        "Starting simulation with {} particles for {} steps.",
        num_particles, total_steps
    );
    let start_time = Instant::now();

    simulation.run(total_steps, save_interval);

    let duration = start_time.elapsed();
    println!("Total simulation time: {:?}", duration);

    // --- Graceful Shutdown ---
    // The sender (`tx`) is part of the `simulation` struct. When `simulation` goes
    // out of scope here at the end of `main`, `tx` is dropped. This closes the
    // channel, causing the `for` loop in the I/O thread to terminate.

    // We explicitly wait for the I/O thread to finish its work. This ensures
    // that all data is written to the file before the program exits.
    io_thread_handle
        .join()
        .expect("I/O thread panicked during execution.");

    println!("Program finished successfully.");
}
