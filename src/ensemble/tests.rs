/// # Unit tests for the ensemble module
#[cfg(test)]
mod units {
    use crate::ensemble::{generate_entry, EntryGenerationParams, EntryGenerationRequest};
    use std::sync::mpsc;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn generate_concurrent_ensemble_creation() {
        // Test parameters for multiple concurrent ensemble generations
        let test_cases = vec![
            (50, 1.0, 1.5, 0.1),   // Small ensemble
            (100, 2.0, 2.0, 0.15), // Medium ensemble
            (25, 1.5, 1.0, 0.2),   // Dense packing test
        ];

        let mut handles = Vec::new();
        let (tx, rx) = mpsc::channel();

        // Spawn concurrent threads for ensemble generation
        for (i, (n_particles, radius, speed, min_distance)) in test_cases.into_iter().enumerate() {
            let tx_clone = tx.clone();

            let handle = thread::spawn(move || {
                let request = EntryGenerationRequest {
                    id: i,
                    tag: format!("concurrent_test_{}", i),
                    params: EntryGenerationParams {
                        n_particles,
                        radius,
                        speed,
                        min_distance,
                    },
                };

                // Call the generate function
                let result = generate_entry(request, tx_clone);
                assert!(
                    result.is_ok(),
                    "Ensemble generation failed for test case {}",
                    i
                );
            });

            handles.push(handle);
        }

        // Drop the original sender to allow receiver to detect completion
        drop(tx);

        // Wait for all threads to complete
        for handle in handles {
            handle
                .join()
                .expect("Thread panicked during ensemble generation");
        }

        // Collect all results
        let mut results = Vec::new();
        while let Ok(result) = rx.try_recv() {
            results.push(result);
        }

        // Verify we received exactly 3 results (one for each test case)
        assert_eq!(
            results.len(),
            3,
            "Should receive exactly 3 ensemble results"
        );

        // Verify each result has the correct number of birds and metadata
        let expected_particle_counts = [50, 100, 25];
        let expected_tags = [
            "concurrent_test_0",
            "concurrent_test_1",
            "concurrent_test_2",
        ];

        // Sort results by ID to ensure consistent ordering
        results.sort_by_key(|r| r.id);

        for (i, result) in results.iter().enumerate() {
            // Check correct number of birds
            assert_eq!(
                result.birds.len(),
                expected_particle_counts[i],
                "Ensemble {} should have {} birds, but has {}",
                i,
                expected_particle_counts[i],
                result.birds.len()
            );

            // Check metadata integrity
            assert_eq!(result.id, i, "Ensemble ID should match");
            assert_eq!(result.tag, expected_tags[i], "Ensemble tag should match");
            assert_eq!(
                result.params.n_particles, expected_particle_counts[i],
                "Params should match"
            );

            // Verify all birds are on the sphere surface (within tolerance)
            for (j, bird) in result.birds.iter().enumerate() {
                let distance_from_origin =
                    (bird.position.x.powi(2) + bird.position.y.powi(2) + bird.position.z.powi(2))
                        .sqrt();
                let radius_tolerance = 1e-10;
                assert!(
                    (distance_from_origin - result.params.radius).abs() < radius_tolerance,
                    "Bird {} in ensemble {} is not on sphere surface. Expected radius: {}, actual: {}",
                    j,
                    i,
                    result.params.radius,
                    distance_from_origin
                );
            }

            // Verify minimum distance constraint is satisfied
            for (j, bird1) in result.birds.iter().enumerate() {
                for (k, bird2) in result.birds.iter().enumerate() {
                    if j != k {
                        let distance = bird1.distance_from(bird2, result.params.radius);
                        assert!(
                            distance >= result.params.min_distance,
                            "Birds {} and {} in ensemble {} are too close: {} < {}",
                            j,
                            k,
                            i,
                            distance,
                            result.params.min_distance
                        );
                    }
                }
            }

            println!(
                "✓ Ensemble {}: Generated {} birds with tag '{}' successfully",
                i,
                result.birds.len(),
                result.tag
            );
        }
    }

    #[test]
    fn generate_single_thread_correctness() {
        // Test single-threaded generation for baseline verification
        let (tx, rx) = mpsc::channel();

        let request = EntryGenerationRequest {
            id: 42,
            tag: "single_thread_test".to_string(),
            params: EntryGenerationParams {
                n_particles: 75,
                radius: 1.0,
                speed: 2.5,
                min_distance: 0.12,
            },
        };

        // Generate ensemble
        let generation_result = generate_entry(request.clone(), tx);
        assert!(
            generation_result.is_ok(),
            "Single-threaded generation should succeed"
        );

        // Receive result
        let result = rx
            .recv_timeout(Duration::from_secs(10))
            .expect("Should receive result within 10 seconds");

        // Verify correctness
        assert_eq!(result.id, 42, "ID should be preserved");
        assert_eq!(result.tag, "single_thread_test", "Tag should be preserved");
        assert_eq!(result.birds.len(), 75, "Should generate exactly 75 birds");
        assert_eq!(result.params.n_particles, 75, "Params should match request");
        assert_eq!(result.params.radius, 1.0, "Radius should match request");
        assert_eq!(result.params.speed, 2.5, "Speed should match request");
        assert_eq!(
            result.params.min_distance, 0.12,
            "Min distance should match request"
        );

        println!(
            "✓ Single-threaded test: Generated {} birds successfully",
            result.birds.len()
        );
    }

    #[test]
    fn generate_stress_test_concurrent_load() {
        // Stress test with many concurrent generations
        const NUM_THREADS: usize = 8;
        const BIRDS_PER_ENSEMBLE: usize = 30;

        let (tx, rx) = mpsc::channel();
        let mut handles = Vec::new();

        // Spawn many concurrent threads
        for i in 0..NUM_THREADS {
            let tx_clone = tx.clone();

            let handle = thread::spawn(move || {
                let request = EntryGenerationRequest {
                    id: i,
                    tag: format!("stress_test_{}", i),
                    params: EntryGenerationParams {
                        n_particles: BIRDS_PER_ENSEMBLE,
                        radius: 1.0,
                        speed: 1.0,
                        min_distance: 0.15,
                    },
                };

                generate_entry(request, tx_clone).expect("Generation should succeed");
            });

            handles.push(handle);
        }

        drop(tx);

        // Wait for all threads
        for handle in handles {
            handle.join().expect("Thread should complete successfully");
        }

        // Collect and verify all results
        let mut results = Vec::new();
        while let Ok(result) = rx.try_recv() {
            results.push(result);
        }

        assert_eq!(
            results.len(),
            NUM_THREADS,
            "Should receive results from all threads"
        );

        // Verify each result has correct number of birds
        for result in results {
            assert_eq!(
                result.birds.len(),
                BIRDS_PER_ENSEMBLE,
                "Each ensemble should have exactly {} birds",
                BIRDS_PER_ENSEMBLE
            );
        }

        println!(
            "✓ Stress test: {} concurrent ensembles, {} birds each, all successful",
            NUM_THREADS, BIRDS_PER_ENSEMBLE
        );
    }
}
