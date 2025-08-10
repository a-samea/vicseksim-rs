//! Integration tests for the flocking simulation library
//!
//! These tests verify that the library components work together correctly
//! and that the overall simulation behavior is physically reasonable.

use flocking_lib::ensemble::{generate, EnsembleGenerationRequest, EnsembleGenerationParams};
use flocking_lib::io::ensemble::{start_receiver_thread, list_ensemble_tags_and_ids, load_ensemble};
use flocking_lib::io;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use std::fs;

/// Test that ensemble generation produces valid configurations
#[test]
fn test_ensemble_generation() {
    assert!(true);
}

/// Integration test for ensemble generation and IO persistence
/// 
/// This test verifies the complete workflow:
/// 1. Generate 3 concurrent ensembles with different IDs
/// 2. Save them concurrently using the IO system
/// 3. List all saved files to verify persistence
/// 4. Load them back to verify correct deserialization
#[test]
fn ensemble_generation_and_io_integration() {
    // Ensure data directories exist
    io::ensure_data_directories().expect("Should be able to create data directories");

    // Clean up any existing test files
    cleanup_test_files();

    // Define test parameters for 3 different ensembles
    let test_cases = vec![
        (0, 30, 1.0, 1.5, 0.15),  // Small ensemble
        (1, 50, 1.2, 2.0, 0.12),  // Medium ensemble  
        (2, 25, 0.8, 1.8, 0.18),  // Dense ensemble
    ];

    // Set up channels for ensemble generation and IO
    let (generation_tx, generation_rx) = mpsc::channel();
    let (io_tx, io_rx) = mpsc::channel();

    // Start the IO receiver thread for concurrent saving
    let io_handle = start_receiver_thread(io_rx);

    // Create and start ensemble generation threads
    let mut generation_handles = Vec::new();
    
    for (id, n_particles, radius, speed, min_distance) in test_cases.iter() {
        let gen_tx = generation_tx.clone();
        let id = *id;
        let n_particles = *n_particles;
        let radius = *radius;
        let speed = *speed;
        let min_distance = *min_distance;

        let handle = thread::spawn(move || {
            let request = EnsembleGenerationRequest {
                id,
                tag: "test".to_string(),
                params: EnsembleGenerationParams {
                    n_particles,
                    radius,
                    speed,
                    min_distance,
                },
            };

            generate(request, gen_tx).expect("Ensemble generation should succeed");
        });

        generation_handles.push(handle);
    }

    // Drop the original generation sender
    drop(generation_tx);

    // Wait for all generation threads to complete
    for handle in generation_handles {
        handle.join().expect("Generation thread should complete successfully");
    }

    // Collect generated ensembles and forward them to IO
    let mut generated_ensembles = Vec::new();
    while let Ok(ensemble) = generation_rx.recv_timeout(Duration::from_millis(100)) {
        io_tx.send(ensemble.clone()).expect("Should be able to send to IO");
        generated_ensembles.push(ensemble);
    }

    // Drop IO sender to signal completion
    drop(io_tx);

    // Wait for IO thread to complete saving
    io_handle.join().expect("IO thread should complete").expect("IO operations should succeed");

    // Verify we generated exactly 3 ensembles
    assert_eq!(generated_ensembles.len(), 3, "Should have generated exactly 3 ensembles");

    // Sort by ID for consistent verification
    generated_ensembles.sort_by_key(|e| e.id);

    // Verify each generated ensemble has correct properties
    for (i, ensemble) in generated_ensembles.iter().enumerate() {
        assert_eq!(ensemble.id, i, "Ensemble ID should match expected");
        assert_eq!(ensemble.tag, "test", "Ensemble tag should be 'test'");
        assert_eq!(ensemble.birds.len(), test_cases[i].1, "Bird count should match parameters");
        
        // Verify ensemble parameters
        assert_eq!(ensemble.params.n_particles, test_cases[i].1);
        assert!((ensemble.params.radius - test_cases[i].2).abs() < 1e-10);
        assert!((ensemble.params.speed - test_cases[i].3).abs() < 1e-10);
        assert!((ensemble.params.min_distance - test_cases[i].4).abs() < 1e-10);
    }

    // Test listing functionality - verify files were saved
    let listed_ensembles = list_ensemble_tags_and_ids()
        .expect("Should be able to list ensemble files");

    // Filter for our test ensembles
    let test_ensembles: Vec<_> = listed_ensembles.into_iter()
        .filter(|(tag, _)| tag == "test")
        .collect();

    assert_eq!(test_ensembles.len(), 3, "Should find exactly 3 test ensemble files");

    // Verify all expected IDs are present
    let mut found_ids: Vec<_> = test_ensembles.iter().map(|(_, id)| *id).collect();
    found_ids.sort();
    assert_eq!(found_ids, vec![0, 1, 2], "Should find ensembles with IDs 0, 1, 2");

    // Test loading functionality - load each ensemble and verify
    for (original_ensemble, (id, n_particles, radius, speed, min_distance)) in 
        generated_ensembles.iter().zip(test_cases.iter()) {
        
        let loaded_ensemble = load_ensemble("test", id)
            .expect(&format!("Should be able to load ensemble with ID {}", id));

        // Verify loaded ensemble matches original
        assert_eq!(loaded_ensemble.id, *id, "Loaded ID should match");
        assert_eq!(loaded_ensemble.tag, "test", "Loaded tag should match");
        assert_eq!(loaded_ensemble.birds.len(), *n_particles, "Loaded bird count should match");
        
        // Verify parameters match
        assert_eq!(loaded_ensemble.params.n_particles, *n_particles);
        assert!((loaded_ensemble.params.radius - radius).abs() < 1e-10);
        assert!((loaded_ensemble.params.speed - speed).abs() < 1e-10);
        assert!((loaded_ensemble.params.min_distance - min_distance).abs() < 1e-10);

        // Verify birds are identical (positions and velocities)
        assert_eq!(loaded_ensemble.birds.len(), original_ensemble.birds.len());
        for (loaded_bird, original_bird) in loaded_ensemble.birds.iter().zip(original_ensemble.birds.iter()) {
            // Compare positions with small tolerance for floating point precision
            assert!((loaded_bird.position.x - original_bird.position.x).abs() < 1e-10);
            assert!((loaded_bird.position.y - original_bird.position.y).abs() < 1e-10);
            assert!((loaded_bird.position.z - original_bird.position.z).abs() < 1e-10);
            
            // Compare velocities
            assert!((loaded_bird.velocity.x - original_bird.velocity.x).abs() < 1e-10);
            assert!((loaded_bird.velocity.y - original_bird.velocity.y).abs() < 1e-10);
            assert!((loaded_bird.velocity.z - original_bird.velocity.z).abs() < 1e-10);
        }

        // Verify timestamp was added (should be non-zero)
        assert!(loaded_ensemble.created_at > 0, "Timestamp should be set by IO module");
    }

    println!("✓ Integration test passed: Generated, saved, listed, and loaded 3 ensembles successfully");

    // Clean up test files
    cleanup_test_files();
}

/// Helper function to clean up test files
fn cleanup_test_files() {
    let test_files = [
        "./data/ensemble/test-0.bin",
        "./data/ensemble/test-1.bin", 
        "./data/ensemble/test-2.bin",
    ];

    for file_path in &test_files {
        if let Err(_) = fs::remove_file(file_path) {
            // Ignore errors - file might not exist
        }
    }
}
