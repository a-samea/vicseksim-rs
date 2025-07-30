use clap::Parser;
use flocking_lib::cli::{Cli, Commands};
use flocking_lib::{ensemble, io};
use std::sync::mpsc;
use std::thread;
use std::time::Instant;

fn main() -> Result<(), String> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::GenerateEnsembles {
            ensemble_count,
            birds_per_ensemble,
            threads,
            radius,
            speed,
            min_distance,
            tag,
        } => {
            println!("--- Parallel Ensemble Generation ---");
            println!(
                "Generating {} ensembles with {} birds each",
                ensemble_count, birds_per_ensemble
            );
            println!("Using {} threads for parallel generation", threads);
            println!(
                "Configuration: radius={}, speed={}, min_distance={}",
                radius, speed, min_distance
            );

            // Ensure data directories exist
            io::ensure_data_directories()
                .map_err(|e| format!("Failed to create data directories: {}", e))?;

            let start_time = Instant::now();

            // Create a channel for collecting completed ensembles
            let (ensemble_tx, ensemble_rx) = mpsc::channel();

            // Create worker threads
            let mut handles = Vec::new();
            let ensembles_per_thread = (*ensemble_count + threads - 1) / threads; // Ceiling division

            for thread_id in 0..*threads {
                let start_ensemble = thread_id * ensembles_per_thread;
                let end_ensemble =
                    std::cmp::min(start_ensemble + ensembles_per_thread, *ensemble_count);

                if start_ensemble >= *ensemble_count {
                    break; // No more work for this thread
                }

                let tx = ensemble_tx.clone();
                let birds_per_ensemble = *birds_per_ensemble;
                let radius = *radius;
                let speed = *speed;
                let min_distance = *min_distance;
                let base_tag = tag.clone();

                let handle = thread::spawn(move || {
                    println!(
                        "Thread {} starting: generating ensembles {} to {}",
                        thread_id,
                        start_ensemble,
                        end_ensemble - 1
                    );

                    for ensemble_id in start_ensemble..end_ensemble {
                        let ensemble_tag = format!("{}_{:04}", base_tag, ensemble_id);

                        // Create a channel for this specific ensemble generation
                        let (gen_tx, gen_rx) = mpsc::channel();

                        // Generate the ensemble
                        match ensemble::generate(
                            birds_per_ensemble,
                            radius,
                            speed,
                            min_distance,
                            gen_tx,
                        ) {
                            Ok(()) => {
                                // Receive the generated birds
                                match gen_rx.recv() {
                                    Ok(birds) => {
                                        // Send the ensemble data to the main thread for saving
                                        if let Err(e) = tx.send((ensemble_id, ensemble_tag, birds))
                                        {
                                            eprintln!(
                                                "Thread {}: Failed to send ensemble {}: {}",
                                                thread_id, ensemble_id, e
                                            );
                                        } else {
                                            println!(
                                                "Thread {}: Completed ensemble {} ({} birds)",
                                                thread_id, ensemble_id, birds_per_ensemble
                                            );
                                        }
                                    }
                                    Err(e) => {
                                        eprintln!(
                                            "Thread {}: Failed to receive ensemble {}: {}",
                                            thread_id, ensemble_id, e
                                        );
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!(
                                    "Thread {}: Failed to generate ensemble {}: {}",
                                    thread_id, ensemble_id, e
                                );
                            }
                        }
                    }

                    println!("Thread {} completed", thread_id);
                });

                handles.push(handle);
            }

            // Drop the original sender so the receiver will know when all threads are done
            drop(ensemble_tx);

            // Collect and save ensembles as they complete
            let mut completed_count = 0;
            while let Ok((ensemble_id, ensemble_tag, birds)) = ensemble_rx.recv() {
                // Start a receiver for saving this ensemble
                let (save_tx, save_rx) = mpsc::channel();

                // Send the birds to be saved
                if let Err(e) = save_tx.send(birds) {
                    eprintln!("Failed to send birds for saving: {}", e);
                    continue;
                }

                // Start the receiver to save the ensemble
                match io::ensemble::start_receiver(
                    save_rx,
                    ensemble_tag.clone(),
                    *radius,
                    *speed,
                    *min_distance,
                ) {
                    Ok(_) => {
                        completed_count += 1;
                        println!(
                            "Saved ensemble {} ({}/{} completed)",
                            ensemble_tag, completed_count, ensemble_count
                        );
                    }
                    Err(e) => {
                        eprintln!("Failed to save ensemble {}: {}", ensemble_tag, e);
                    }
                }
            }

            // Wait for all threads to complete
            for handle in handles {
                if let Err(e) = handle.join() {
                    eprintln!("Thread panicked: {:?}", e);
                }
            }

            let duration = start_time.elapsed();
            println!("\n--- Generation Complete ---");
            println!("Successfully generated {} ensembles", completed_count);
            println!("Total time: {:.2} seconds", duration.as_secs_f64());
            println!(
                "Average time per ensemble: {:.2} seconds",
                duration.as_secs_f64() / *ensemble_count as f64
            );
            println!("Ensembles saved to: ./data/ensemble/");

            Ok(())
        }

        Commands::Simulate {
            input_dir,
            output_dir,
            steps,
            interaction_radius,
            noise,
        } => {
            println!("--- Simulation Mode ---");
            println!("Input directory: {:?}", input_dir);
            println!("Output directory: {:?}", output_dir);
            println!("Total steps: {}", steps);
            println!("Interaction radius: {}", interaction_radius);
            println!("Noise parameter: {}", noise);

            // Implementation will be added in Stage 2
            unimplemented!("Call simulation logic here.");
        }

        Commands::Analyze {
            input_dir,
            output_dir,
            analysis_types,
        } => {
            println!("--- Analyze Mode ---");
            println!("Analyzing results from directory: {:?}", input_dir);
            println!("Saving analysis results to: {:?}", output_dir);
            println!("Analysis types: {:?}", analysis_types);
            // Implementation will be added in Stage 3
            unimplemented!("Call analysis logic here.");
        }
    };

    Ok(())
}
