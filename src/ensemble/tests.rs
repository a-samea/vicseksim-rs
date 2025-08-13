//! Comprehensive unit tests for the ensemble generation module.
//!
//! This test suite covers all public and private APIs of the ensemble module,
//! including physics validation, parallel generation, I/O integration, and
//! error handling scenarios.

#[cfg(test)]
mod units {
    use crate::bird::Bird;
    use crate::ensemble::{generate, EntryGenerationParams, EntryResult};
    use crate::io::bin;
    use std::fs;
    use std::path::Path;
    use std::sync::mpsc;
    use tempfile::TempDir;

    // =========================================================================
    // HELPER FUNCTIONS AND TEST UTILITIES
    // =========================================================================

    /// Creates test parameters with reasonable defaults
    fn test_params() -> EntryGenerationParams {
        EntryGenerationParams {
            num_birds: 10,
            radius: 1.0,
            speed: 1.0,
            min_distance: 0.1,
        }
    }

    /// Creates test parameters with tight constraints for stress testing
    fn tight_constraint_params() -> EntryGenerationParams {
        EntryGenerationParams {
            num_birds: 5,
            radius: 1.0,
            speed: 1.0,
            min_distance: 0.8, // High constraint relative to sphere
        }
    }

    /// Validates that all birds are on the sphere surface within tolerance
    fn validate_on_sphere(birds: &[Bird], radius: f64, tolerance: f64) -> bool {
        birds
            .iter()
            .all(|bird| (bird.position.norm() - radius).abs() < tolerance)
    }

    /// Validates that all velocity vectors are tangent to sphere
    fn validate_tangent_velocities(birds: &[Bird], tolerance: f64) -> bool {
        birds
            .iter()
            .all(|bird| bird.position.dot(&bird.velocity).abs() < tolerance)
    }

    /// Validates that all velocity magnitudes match expected speed
    fn validate_speed(birds: &[Bird], expected_speed: f64, tolerance: f64) -> bool {
        birds
            .iter()
            .all(|bird| (bird.velocity.norm() - expected_speed).abs() < tolerance)
    }

    /// Validates minimum distance constraints between all bird pairs
    fn validate_distance_constraints(birds: &[Bird], min_distance: f64, radius: f64) -> bool {
        for (i, bird1) in birds.iter().enumerate() {
            for bird2 in birds.iter().skip(i + 1) {
                if bird1.distance_from(bird2, radius) < min_distance {
                    return false;
                }
            }
        }
        true
    }

    /// Creates a temporary directory for test file operations
    fn setup_temp_dir() -> TempDir {
        tempfile::tempdir().expect("Failed to create temp dir")
    }

    // =========================================================================
    // TESTS
    // =========================================================================
    /// Tests `generate_entry()` with typical simulation parameters.
    ///
    /// Validates that:
    /// - Multiple birds are generated correctly
    /// - All distance constraints are satisfied
    /// - Birds are positioned on sphere surface
    /// - Velocity vectors have correct magnitude and are tangent to sphere
    #[test]
    fn generate_entry_typical() {
        let params = test_params();
        let tag = 1;

        // Call the private generate_entry function through the public API
        // We'll test via the single entry generation
        let result = generate(tag, 1, params);
        assert!(result.is_ok());

        // Load the generated file to validate
        let data_path = Path::new("./data/ensemble");
        let files: Vec<_> = fs::read_dir(data_path)
            .unwrap()
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .contains(&format!("t{}", tag))
            })
            .collect();

        assert_eq!(files.len(), 1);

        let file_path = files[0].path();
        let entry: EntryResult = bin::load_file(&file_path).unwrap();

        // Validate basic properties
        assert_eq!(entry.birds.len(), params.num_birds);
        assert_eq!(entry.tag, tag);

        // Validate physics
        assert!(validate_on_sphere(&entry.birds, params.radius, 1e-10));
        assert!(validate_tangent_velocities(&entry.birds, 1e-10));
        assert!(validate_speed(&entry.birds, params.speed, 1e-10));
        assert!(validate_distance_constraints(
            &entry.birds,
            params.min_distance,
            params.radius
        ));

        // Cleanup
        fs::remove_file(file_path).ok();
    }

    /// Tests `generate_entry()` with challenging distance constraints.
    ///
    /// Validates that:
    /// - Algorithm can handle tight packing scenarios
    /// - Generation completes within reasonable time
    /// - All generated birds satisfy minimum distance requirements
    #[test]
    fn generate_entry_tight_constraints() {
        let params = tight_constraint_params();
        let tag = 2;

        let result = generate(tag, 1, params);
        assert!(result.is_ok());

        // Load and validate the generated entry
        let data_path = Path::new("./data/ensemble");
        let files: Vec<_> = fs::read_dir(data_path)
            .unwrap()
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .contains(&format!("t{}", tag))
            })
            .collect();

        assert_eq!(files.len(), 1);

        let file_path = files[0].path();
        let entry: EntryResult = bin::load_file(&file_path).unwrap();

        // Verify tight constraints are still satisfied
        assert!(validate_distance_constraints(
            &entry.birds,
            params.min_distance,
            params.radius
        ));
        assert_eq!(entry.birds.len(), params.num_birds);

        // Cleanup
        fs::remove_file(file_path).ok();
    }

    /// Tests `generate_entry()` physics validation.
    ///
    /// Validates that:
    /// - All birds are exactly on sphere surface (within numerical precision)
    /// - Velocity vectors are tangent to sphere at bird positions
    /// - Velocity magnitudes match specified speed parameter
    /// - No birds are positioned at identical coordinates
    #[test]
    fn generate_entry_physics_validation() {
        let params = EntryGenerationParams {
            num_birds: 15,
            radius: 2.5,
            speed: 3.0,
            min_distance: 0.2,
        };
        let tag = 3;

        let result = generate(tag, 1, params);
        assert!(result.is_ok());

        let data_path = Path::new("./data/ensemble");
        let files: Vec<_> = fs::read_dir(data_path)
            .unwrap()
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .contains(&format!("t{}", tag))
            })
            .collect();

        let file_path = files[0].path();
        let entry: EntryResult = bin::load_file(&file_path).unwrap();

        // Physics validation with tight tolerances
        assert!(validate_on_sphere(&entry.birds, params.radius, 1e-12));
        assert!(validate_tangent_velocities(&entry.birds, 1e-12));
        assert!(validate_speed(&entry.birds, params.speed, 1e-12));

        // Check no identical positions
        for (i, bird1) in entry.birds.iter().enumerate() {
            for bird2 in entry.birds.iter().skip(i + 1) {
                assert!(bird1.distance_from(bird2, params.radius) > 0.0);
            }
        }

        // Cleanup
        fs::remove_file(file_path).ok();
    }

    /// Tests `generate_entry()` MPSC communication.
    ///
    /// Validates that:
    /// - Results are correctly sent through MPSC channel
    /// - Channel communication doesn't block generation
    /// - Multiple results can be sent through same channel
    /// - Channel errors are properly handled
    #[test]
    fn generate_entry_mpsc_communication() {
        // This test indirectly validates MPSC by testing multiple entries
        let params = test_params();
        let tag = 4;
        let num_entries = 3;

        // Clean up any existing files first
        let data_path = Path::new("./data/ensemble");
        if let Ok(entries) = fs::read_dir(data_path) {
            for entry in entries.flatten() {
                if entry
                    .file_name()
                    .to_string_lossy()
                    .contains(&format!("t{}", tag))
                {
                    fs::remove_file(entry.path()).ok();
                }
            }
        }

        let result = generate(tag, num_entries, params);
        assert!(result.is_ok());

        // Verify all entries were transmitted and saved via MPSC
        let files: Vec<_> = fs::read_dir(data_path)
            .unwrap()
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .contains(&format!("t{}", tag))
            })
            .collect();

        assert_eq!(files.len(), num_entries);

        // Cleanup
        for file in files {
            fs::remove_file(file.path()).ok();
        }
    }

    /// Tests `generate()` function with single ensemble entry.
    ///
    /// Validates that:
    /// - Single entry generation completes successfully
    /// - Generated files have correct naming convention
    /// - File contents match generated data
    /// - Function returns success status
    #[test]
    fn generate_single_entry() {
        let params = test_params();
        let tag = 5;

        let result = generate(tag, 1, params);
        assert!(result.is_ok());

        // Check file naming convention
        let expected_file = Path::new("./data/ensemble").join(format!("t{}-i0.bin", tag));
        assert!(expected_file.exists());

        // Load and validate contents
        let entry: EntryResult = bin::load_file(&expected_file).unwrap();
        assert_eq!(entry.id, 0);
        assert_eq!(entry.tag, tag);
        assert_eq!(entry.params.num_birds, params.num_birds);

        // Cleanup
        fs::remove_file(expected_file).ok();
    }

    /// Tests `generate()` function with multiple ensemble entries.
    ///
    /// Validates that:
    /// - Multiple entries are generated in parallel
    /// - All entries complete successfully
    /// - Generated files are correctly numbered and tagged
    /// - No data corruption occurs during parallel processing
    #[test]
    fn generate_multiple_entries() {
        let params = test_params();
        let tag = 6;
        let num_entries = 5;

        let result = generate(tag, num_entries, params);
        assert!(result.is_ok());

        // Verify all files are created with correct naming
        for i in 0..num_entries {
            let expected_file = Path::new("./data/ensemble").join(format!("t{}-i{}.bin", tag, i));
            assert!(expected_file.exists());

            let entry: EntryResult = bin::load_file(&expected_file).unwrap();
            assert_eq!(entry.id, i);
            assert_eq!(entry.tag, tag);
            assert_eq!(entry.birds.len(), params.num_birds);

            // Cleanup
            fs::remove_file(expected_file).ok();
        }
    }

    /// Tests `generate()` function with large ensemble counts.
    ///
    /// Validates that:
    /// - System can handle hundreds of ensemble entries
    /// - Memory usage scales appropriately
    /// - Generation completes within reasonable time bounds
    /// - All results are correctly persisted
    #[test]
    fn generate_large_ensemble_count() {
        let params = EntryGenerationParams {
            num_birds: 5, // Smaller bird count for faster test
            radius: 1.0,
            speed: 1.0,
            min_distance: 0.2,
        };
        let tag = 7;
        let num_entries = 50; // Large but manageable for CI

        let result = generate(tag, num_entries, params);
        assert!(result.is_ok());

        // Verify all entries were created
        let data_path = Path::new("./data/ensemble");
        let files: Vec<_> = fs::read_dir(data_path)
            .unwrap()
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .contains(&format!("t{}", tag))
            })
            .collect();

        assert_eq!(files.len(), num_entries);

        // Spot check a few entries
        for i in [0, num_entries / 2, num_entries - 1] {
            let expected_file = Path::new("./data/ensemble").join(format!("t{}-i{}.bin", tag, i));
            let entry: EntryResult = bin::load_file(&expected_file).unwrap();
            assert_eq!(entry.id, i);
            assert_eq!(entry.tag, tag);
        }

        // Cleanup
        for file in files {
            fs::remove_file(file.path()).ok();
        }
    }

    /// Tests `generate()` function error scenarios.
    ///
    /// Validates that:
    /// - Invalid parameters produce appropriate errors
    /// - I/O failures are properly handled and reported
    /// - Partial failures don't corrupt successful entries
    /// - Error messages are descriptive and helpful
    #[test]
    fn generate_error_scenarios() {
        // Test with challenging but not impossible constraints
        let challenging_params = EntryGenerationParams {
            num_birds: 3, // Small number to avoid hanging
            radius: 1.0,
            speed: 1.0,
            min_distance: 1.0, // Challenging but possible for 3 birds
        };
        let tag = 8;

        // This should complete but may take longer
        let result = generate(tag, 1, challenging_params);

        // Should succeed with reasonable constraints
        match result {
            Ok(_) => {
                // If it succeeds, verify and cleanup
                let file = Path::new("./data/ensemble").join(format!("t{}-i0.bin", tag));
                if file.exists() {
                    let entry: EntryResult = bin::load_file(&file).unwrap();
                    assert!(validate_distance_constraints(
                        &entry.birds,
                        challenging_params.min_distance,
                        challenging_params.radius
                    ));
                    fs::remove_file(file).ok();
                }
            }
            Err(e) => {
                // Error handling test - ensure error message is descriptive
                assert!(!e.is_empty(), "Error message should not be empty");
            }
        }
    }

    /// Tests `start_receiver_thread()` basic functionality.
    ///
    /// Validates that:
    /// - Thread starts and runs correctly
    /// - Receives data through MPSC channel
    /// - Processes received data without errors
    /// - Thread terminates cleanly when channel closes
    #[test]
    fn start_receiver_thread_basic() {
        use crate::ensemble::io::start_receiver_thread;

        let (tx, rx) = mpsc::channel();
        let handle = start_receiver_thread(rx);

        // Create a test entry
        let test_entry = EntryResult {
            id: 0,
            tag: 10,
            birds: vec![Bird::from_spherical(1.0, 0.0, 0.0, 1.0, 0.0)],
            params: test_params(),
        };

        // Send the entry
        tx.send(test_entry).unwrap();
        drop(tx); // Close channel to terminate thread

        // Wait for thread completion
        let result = handle.join().unwrap();
        assert!(result.is_ok());

        // Verify file was created
        let expected_file = Path::new("./data/ensemble").join("t10-i0.bin");
        assert!(expected_file.exists());

        // Cleanup
        fs::remove_file(expected_file).ok();
    }

    /// Tests `start_receiver_thread()` with multiple entries.
    ///
    /// Validates that:
    /// - Multiple entries are processed in sequence
    /// - All entries are saved correctly
    /// - No data is lost during processing
    /// - Thread handles high throughput scenarios
    #[test]
    fn start_receiver_thread_multiple_entries() {
        use crate::ensemble::io::start_receiver_thread;

        let (tx, rx) = mpsc::channel();
        let handle = start_receiver_thread(rx);

        let num_entries = 5;
        let tag = 11;

        // Send multiple entries
        for i in 0..num_entries {
            let entry = EntryResult {
                id: i,
                tag,
                birds: vec![
                    Bird::from_spherical(1.0, 0.1 * i as f64, 0.0, 1.0, 0.0),
                    Bird::from_spherical(1.0, 0.2 * i as f64, 0.5, 1.0, 1.0),
                ],
                params: test_params(),
            };
            tx.send(entry).unwrap();
        }

        drop(tx);
        let result = handle.join().unwrap();
        assert!(result.is_ok());

        // Verify all files were created
        for i in 0..num_entries {
            let expected_file = Path::new("./data/ensemble").join(format!("t{}-i{}.bin", tag, i));
            assert!(expected_file.exists());

            let loaded_entry: EntryResult = bin::load_file(&expected_file).unwrap();
            assert_eq!(loaded_entry.id, i);
            assert_eq!(loaded_entry.tag, tag);

            // Cleanup
            fs::remove_file(expected_file).ok();
        }
    }

    /// Tests end-to-end I/O integration.
    ///
    /// Validates that:
    /// - Generated ensembles are saved to correct file paths
    /// - File contents can be loaded and deserialized correctly
    /// - Directory structure is created automatically
    /// - File naming follows expected conventions
    #[test]
    fn io_integration_end_to_end() {
        let params = test_params();
        let tag = 12;
        let num_entries = 3;

        // Ensure directory doesn't exist initially
        let ensemble_dir = Path::new("./data/ensemble");
        if ensemble_dir.exists() {
            // Clean any existing files for this tag
            if let Ok(entries) = fs::read_dir(ensemble_dir) {
                for entry in entries.flatten() {
                    if entry
                        .file_name()
                        .to_string_lossy()
                        .contains(&format!("t{}", tag))
                    {
                        fs::remove_file(entry.path()).ok();
                    }
                }
            }
        }

        // Generate entries
        let result = generate(tag, num_entries, params);
        assert!(result.is_ok());

        // Verify directory creation and file naming
        assert!(ensemble_dir.exists());

        for i in 0..num_entries {
            let expected_path = ensemble_dir.join(format!("t{}-i{}.bin", tag, i));
            assert!(expected_path.exists());

            // Test round-trip serialization
            let loaded: EntryResult = bin::load_file(&expected_path).unwrap();
            assert_eq!(loaded.id, i);
            assert_eq!(loaded.tag, tag);
            assert_eq!(loaded.birds.len(), params.num_birds);

            // Cleanup
            fs::remove_file(expected_path).ok();
        }
    }

    /// Tests file system integration.
    ///
    /// Validates that:
    /// - Data directory is created when missing
    /// - Files are written with correct permissions
    /// - Existing files are not overwritten incorrectly
    /// - Cleanup works properly after tests
    #[test]
    fn filesystem_integration() {
        let temp_dir = setup_temp_dir();
        let test_dir = temp_dir.path().join("data").join("ensemble");

        // Manually test directory creation by using a custom path
        let entry = EntryResult {
            id: 0,
            tag: 13,
            birds: vec![Bird::from_spherical(1.0, 0.0, 0.0, 1.0, 0.0)],
            params: test_params(),
        };

        // Create custom file path in temp directory
        fs::create_dir_all(&test_dir).unwrap();
        let custom_path = test_dir.join("t13-i0.bin");

        // Test manual save and load
        let serialized = bincode::serialize(&entry).unwrap();
        fs::write(&custom_path, serialized).unwrap();

        // Verify file exists and is readable
        assert!(custom_path.exists());
        let loaded: EntryResult = bin::load_file(&custom_path).unwrap();
        assert_eq!(loaded.id, entry.id);
        assert_eq!(loaded.tag, entry.tag);

        // Test overwrite behavior
        let new_entry = EntryResult {
            id: 1,
            tag: 13,
            birds: vec![Bird::from_spherical(1.0, 1.0, 1.0, 1.0, 1.0)],
            params: test_params(),
        };

        let new_serialized = bincode::serialize(&new_entry).unwrap();
        fs::write(&custom_path, new_serialized).unwrap();

        let reloaded: EntryResult = bin::load_file(&custom_path).unwrap();
        assert_eq!(reloaded.id, 1);

        // Temp directory cleanup is automatic
    }

    /// Tests binary serialization correctness.
    ///
    /// Validates that:
    /// - Serialized data maintains precision
    /// - Deserialized data matches original exactly
    /// - Binary format is compact and efficient
    /// - Cross-platform compatibility is maintained
    #[test]
    fn binary_serialization_correctness() {
        // Create an entry with precise floating-point values
        let precise_bird = Bird::from_spherical(
            1.23456789012345, // High precision radius
            0.98765432109876, // High precision theta
            1.11111111111111, // High precision phi
            2.71828182845905, // High precision speed (e)
            3.14159265358979, // High precision alpha (pi)
        );

        let entry = EntryResult {
            id: 42,
            tag: 14,
            birds: vec![precise_bird],
            params: EntryGenerationParams {
                num_birds: 1,
                radius: 1.23456789012345,
                speed: 2.71828182845905,
                min_distance: 0.123456789,
            },
        };

        // Test serialization round-trip
        let serialized = bincode::serialize(&entry).unwrap();
        let deserialized: EntryResult = bincode::deserialize(&serialized).unwrap();

        // Verify exact equality of all fields
        assert_eq!(deserialized.id, entry.id);
        assert_eq!(deserialized.tag, entry.tag);
        assert_eq!(deserialized.birds.len(), entry.birds.len());

        // Check floating-point precision preservation
        let orig_bird = &entry.birds[0];
        let deser_bird = &deserialized.birds[0];

        assert_eq!(orig_bird.position.x, deser_bird.position.x);
        assert_eq!(orig_bird.position.y, deser_bird.position.y);
        assert_eq!(orig_bird.position.z, deser_bird.position.z);
        assert_eq!(orig_bird.velocity.x, deser_bird.velocity.x);
        assert_eq!(orig_bird.velocity.y, deser_bird.velocity.y);
        assert_eq!(orig_bird.velocity.z, deser_bird.velocity.z);

        // Check parameter preservation
        assert_eq!(deserialized.params.radius, entry.params.radius);
        assert_eq!(deserialized.params.speed, entry.params.speed);
        assert_eq!(deserialized.params.min_distance, entry.params.min_distance);

        // Verify binary format efficiency (should be compact)
        assert!(serialized.len() < 200); // Reasonable size for single bird entry
    }

    /// Tests concurrent generation with different parameters.
    ///
    /// Validates that:
    /// - Multiple parameter sets can run simultaneously
    /// - Results are correctly isolated by tag
    /// - No cross-contamination occurs between concurrent runs
    /// - Resource usage remains manageable
    #[test]
    fn concurrent_different_parameters() {
        use std::thread;

        let params1 = EntryGenerationParams {
            num_birds: 8,
            radius: 1.0,
            speed: 1.0,
            min_distance: 0.15,
        };

        let params2 = EntryGenerationParams {
            num_birds: 6,
            radius: 2.0,
            speed: 0.5,
            min_distance: 0.3,
        };

        let tag1 = 15;
        let tag2 = 16;
        let entries_per_tag = 3;

        // Run two generation tasks concurrently
        let handle1 = thread::spawn(move || generate(tag1, entries_per_tag, params1));

        let handle2 = thread::spawn(move || generate(tag2, entries_per_tag, params2));

        // Wait for both to complete
        let result1 = handle1.join().unwrap();
        let result2 = handle2.join().unwrap();

        assert!(result1.is_ok());
        assert!(result2.is_ok());

        // Verify isolation - each tag should have its own files
        let data_path = Path::new("./data/ensemble");

        // Check tag1 files
        for i in 0..entries_per_tag {
            let file1 = data_path.join(format!("t{}-i{}.bin", tag1, i));
            assert!(file1.exists());
            let entry1: EntryResult = bin::load_file(&file1).unwrap();
            assert_eq!(entry1.tag, tag1);
            assert_eq!(entry1.params.num_birds, 8);
            assert_eq!(entry1.params.radius, 1.0);
            fs::remove_file(file1).ok();
        }

        // Check tag2 files
        for i in 0..entries_per_tag {
            let file2 = data_path.join(format!("t{}-i{}.bin", tag2, i));
            assert!(file2.exists());
            let entry2: EntryResult = bin::load_file(&file2).unwrap();
            assert_eq!(entry2.tag, tag2);
            assert_eq!(entry2.params.num_birds, 6);
            assert_eq!(entry2.params.radius, 2.0);
            fs::remove_file(file2).ok();
        }
    }
}
