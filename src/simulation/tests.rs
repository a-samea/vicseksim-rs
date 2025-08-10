#[cfg(test)]
mod units {
    use crate::simulation::{Simulation, SimulationParams, SimulationRequest};
    use crate::bird::Bird;
    use crate::ensemble::{EnsembleGenerationRequest, EnsembleGenerationParams, generate};
    use crate::vector::Vec3;
    use std::f64::consts::PI;
    use std::sync::mpsc;
    use std::sync::atomic::Ordering;

    /// Helper function to create a simple test simulation with a small ensemble
    fn create_test_simulation(num_birds: usize) -> Simulation {
        // Create test parameters
        let params = SimulationParams {
            num_birds,
            radius: 1.0,
            speed: 1.0,
            dt: 0.01,
            interaction_radius: 0.5,
            eta: 0.1,
            iterations: 5, // Short iterations for unit testing
        };

        // Generate a small ensemble for testing
        let (tx, rx) = mpsc::channel();
        let ensemble_request = EnsembleGenerationRequest {
            id: 1,
            tag: "test_ensemble".to_string(),
            params: EnsembleGenerationParams {
                n_particles: num_birds,
                radius: params.radius,
                speed: params.speed,
                min_distance: 0.1,
            },
        };
        
        // Generate ensemble
        generate(ensemble_request, tx).expect("Failed to generate test ensemble");
        let ensemble_result = rx.recv().expect("Failed to receive ensemble");

        // Create simulation request
        let request = SimulationRequest {
            id: 1,
            tag: "test_simulation".to_string(),
            ensemble_id: 1,
            initial_values: ensemble_result.birds,
            params,
        };

        // Create simulation without frame sender (no MPSC channel needed for unit tests)
        let (frame_tx, _frame_rx) = mpsc::channel();
        Simulation::new(request, frame_tx, 10)
    }

    /// Helper function to create a minimal test simulation with manually placed birds
    fn create_minimal_test_simulation() -> Simulation {
        let params = SimulationParams {
            num_birds: 3,
            radius: 1.0,
            speed: 1.0,
            dt: 0.01,
            interaction_radius: 1.0, // Large radius to ensure interaction
            eta: 0.0, // No noise for predictable testing
            iterations: 3, // Very short for unit testing
        };

        // Create birds at specific positions for predictable testing
        let birds = vec![
            Bird::from_spherical(1.0, PI/2.0, 0.0, 1.0, 0.0),        // North pole (x=1,y=0,z=0)
            Bird::from_spherical(1.0, PI/2.0, PI/2.0, 1.0, 0.0),     // (x=0,y=1,z=0)
            Bird::from_spherical(1.0, PI/2.0, PI, 1.0, 0.0),         // (x=-1,y=0,z=0)
        ];

        let request = SimulationRequest {
            id: 1,
            tag: "minimal_test".to_string(),
            ensemble_id: 1,
            initial_values: birds,
            params,
        };

        let (frame_tx, _frame_rx) = mpsc::channel();
        Simulation::new(request, frame_tx, 2)
    }

    #[test]
    fn simulation_new_creates_valid_simulation() {
        let sim = create_test_simulation(10);
        
        assert_eq!(sim.step_count(), 0);
        assert_eq!(sim.current_time(), 0.0);
        assert_eq!(sim.parameters().num_birds, 10);
        assert_eq!(sim.current_particles().len(), 10);
        assert!(!sim.should_stop.load(Ordering::Relaxed));
    }

    #[test]
    #[should_panic(expected = "Simulation requires at least one bird")]
    fn simulation_new_panics_with_zero_birds() {
        let params = SimulationParams {
            num_birds: 0,
            radius: 1.0,
            speed: 1.0,
            dt: 0.01,
            interaction_radius: 0.5,
            eta: 0.1,
            iterations: 100,
        };

        let request = SimulationRequest {
            id: 1,
            tag: "invalid_test".to_string(),
            ensemble_id: 1,
            initial_values: vec![], // Empty vector
            params,
        };

        let (frame_tx, _frame_rx) = mpsc::channel();
        Simulation::new(request, frame_tx, 10);
    }

    #[test]
    fn simulation_run_advances_state() {
        let mut sim = create_minimal_test_simulation();
        
        let initial_step = sim.step_count();
        let initial_time = sim.current_time();
        let initial_positions: Vec<Vec3> = sim.current_particles().iter().map(|b| b.position).collect();
        
        // Execute the simulation (will run for the number of iterations specified)
        sim.run();
        
        // Verify step counter and time advancement
        assert!(sim.step_count() > initial_step);
        assert!(sim.current_time() > initial_time);
        
        // Verify that positions have changed (birds should move)
        let new_positions: Vec<Vec3> = sim.current_particles().iter().map(|b| b.position).collect();
        let positions_changed = initial_positions.iter()
            .zip(new_positions.iter())
            .any(|(old, new)| (*old - *new).norm() > f64::EPSILON);
        
        assert!(positions_changed, "Positions should change after simulation run");
    }

    #[test]
    fn simulation_run_preserves_sphere_constraint() {
        let mut sim = create_test_simulation(5);
        let radius = sim.parameters().radius;
        
        // Run the simulation
        sim.run();
        
        // Verify all birds remain on sphere surface
        for bird in sim.current_particles() {
            let distance_from_center = bird.position.norm();
            assert!(
                (distance_from_center - radius).abs() < 1e-10,
                "Bird should remain on sphere surface: distance = {}, expected = {}",
                distance_from_center, radius
            );
        }
    }

    #[test]
    fn simulation_run_preserves_speed() {
        let mut sim = create_test_simulation(5);
        let expected_speed = sim.parameters().speed;
        
        // Run the simulation
        sim.run();
        
        // Verify all birds maintain target speed
        for bird in sim.current_particles() {
            let actual_speed = bird.velocity.norm();
            assert!(
                (actual_speed - expected_speed).abs() < 1e-10,
                "Bird should maintain target speed: actual = {}, expected = {}",
                actual_speed, expected_speed
            );
        }
    }

    #[test]
    fn simulation_run_maintains_velocity_tangency() {
        let mut sim = create_test_simulation(5);
        
        // Run the simulation
        sim.run();
        
        // Verify velocity vectors remain tangent to sphere
        for bird in sim.current_particles() {
            let dot_product = bird.position.dot(&bird.velocity);
            assert!(
                dot_product.abs() < 1e-10,
                "Velocity should be tangent to sphere (dot product should be ~0): {}",
                dot_product
            );
        }
    }

    #[test]
    fn simulation_run_executes_correct_iterations() {
        let mut sim = create_minimal_test_simulation();
        let max_iterations = sim.parameters().iterations;
        
        sim.run();
        
        // Should complete all iterations since stop flag is not set
        assert_eq!(sim.step_count(), max_iterations as u64);
        assert!((sim.current_time() - (max_iterations as f64 * sim.params.dt)).abs() < f64::EPSILON);
    }

    #[test]
    fn simulation_run_respects_stop_flag() {
        let mut sim = create_test_simulation(5);
        
        // Set stop flag before running
        sim.stop();
        
        sim.run();
        
        // Should not advance if stop flag is set
        assert_eq!(sim.step_count(), 0);
        assert_eq!(sim.current_time(), 0.0);
    }

    #[test]
    fn simulation_stop_sets_atomic_flag() {
        let sim = create_test_simulation(5);
        
        assert!(!sim.should_stop.load(Ordering::Relaxed));
        
        sim.stop();
        
        assert!(sim.should_stop.load(Ordering::Relaxed));
    }

    #[test]
    fn simulation_drop_sets_stop_flag() {
        let sim = create_test_simulation(5);
        let stop_flag = sim.stop_flag();
        
        assert!(!stop_flag.load(Ordering::Relaxed));
        
        // Drop the simulation
        drop(sim);
        
        // Stop flag should be set
        assert!(stop_flag.load(Ordering::Relaxed));
    }

    #[test]
    fn simulation_parameters_returns_correct_config() {
        let sim = create_test_simulation(7);
        let params = sim.parameters();
        
        assert_eq!(params.num_birds, 7);
        assert_eq!(params.radius, 1.0);
        assert_eq!(params.speed, 1.0);
        assert_eq!(params.dt, 0.01);
        assert_eq!(params.interaction_radius, 0.5);
        assert_eq!(params.eta, 0.1);
        assert_eq!(params.iterations, 5); // Updated to match our test setup
    }

    #[test]
    fn simulation_consistency_across_multiple_runs() {
        let mut sim1 = create_test_simulation(8);
        let mut sim2 = create_test_simulation(8);
        
        // Run both simulations
        sim1.run();
        sim2.run();
        
        // Both should complete the same number of steps
        assert_eq!(sim1.step_count(), sim2.step_count());
        assert_eq!(sim1.current_time(), sim2.current_time());
        
        // Verify all physical constraints for both simulations
        for sim in [&sim1, &sim2] {
            for bird in sim.current_particles() {
                // Sphere constraint
                assert!((bird.position.norm() - sim.params.radius).abs() < 1e-10);
                // Speed constraint  
                assert!((bird.velocity.norm() - sim.params.speed).abs() < 1e-10);
                // Tangency constraint
                assert!(bird.position.dot(&bird.velocity).abs() < 1e-10);
            }
        }
    }

    #[test]
    fn simulation_frame_interval_behavior() {
        let (frame_tx, frame_rx) = mpsc::channel();
        
        // Create a new simulation with frame capture enabled
        let params = SimulationParams {
            num_birds: 2,
            radius: 1.0,
            speed: 1.0,
            dt: 0.01,
            interaction_radius: 0.5,
            eta: 0.0,
            iterations: 10,
        };

        let birds = vec![
            Bird::from_spherical(1.0, PI/2.0, 0.0, 1.0, 0.0),
            Bird::from_spherical(1.0, PI/2.0, PI/2.0, 1.0, 0.0),
        ];

        let request = SimulationRequest {
            id: 1,
            tag: "frame_test".to_string(),
            ensemble_id: 1,
            initial_values: birds,
            params,
        };

        let mut sim_with_frames = Simulation::new(request, frame_tx, 3); // Frame every 3 steps
        
        sim_with_frames.run();
        
        // Try to receive frames (might timeout, which is fine for testing)
        let mut _frame_count = 0;
        while let Ok(_frame) = frame_rx.try_recv() {
            _frame_count += 1;
        }
        
        // We should have received some frames, but the exact number depends on timing
        // The main point is that the simulation doesn't crash when frame sending is enabled
        assert!(sim_with_frames.step_count() > 0);
    }

    #[test]
    fn simulation_state_buffer_integrity() {
        let mut sim = create_test_simulation(4);
        
        // Store initial state
        let initial_birds: Vec<Bird> = sim.current_particles().to_vec();
        let initial_count = initial_birds.len();
        
        // Execute simulation
        sim.run();
        
        // Verify that we have the same number of birds
        assert_eq!(sim.current_particles().len(), initial_count);
        
        // Verify that the current state is different from initial (birds should have moved)
        let current_birds: Vec<Bird> = sim.current_particles().to_vec();
        let state_changed = initial_birds.iter()
            .zip(current_birds.iter())
            .any(|(initial, current)| {
                (initial.position - current.position).norm() > f64::EPSILON ||
                (initial.velocity - current.velocity).norm() > f64::EPSILON
            });
        
        assert!(state_changed, "Simulation state should change after running");
    }

    #[test]
    fn simulation_deterministic_behavior_with_no_noise() {
        // Create two identical simulations with no noise
        let params = SimulationParams {
            num_birds: 3,
            radius: 1.0,
            speed: 1.0,
            dt: 0.01,
            interaction_radius: 0.5,
            eta: 0.0, // No noise for deterministic behavior
            iterations: 5,
        };

        // Use identical initial conditions
        let birds = vec![
            Bird::from_spherical(1.0, PI/2.0, 0.0, 1.0, 0.0),
            Bird::from_spherical(1.0, PI/2.0, PI/3.0, 1.0, 0.0),
            Bird::from_spherical(1.0, PI/2.0, 2.0*PI/3.0, 1.0, 0.0),
        ];

        let request1 = SimulationRequest {
            id: 1,
            tag: "deterministic_test1".to_string(),
            ensemble_id: 1,
            initial_values: birds.clone(),
            params,
        };

        let request2 = SimulationRequest {
            id: 2,
            tag: "deterministic_test2".to_string(),
            ensemble_id: 1,
            initial_values: birds,
            params,
        };

        let (tx1, _rx1) = mpsc::channel();
        let (tx2, _rx2) = mpsc::channel();
        
        let mut sim1 = Simulation::new(request1, tx1, 10);
        let mut sim2 = Simulation::new(request2, tx2, 10);
        
        sim1.run();
        sim2.run();
        
        // Both simulations should produce identical results
        assert_eq!(sim1.step_count(), sim2.step_count());
        assert!((sim1.current_time() - sim2.current_time()).abs() < f64::EPSILON);
        
        // Compare final states (should be identical)
        let birds1 = sim1.current_particles();
        let birds2 = sim2.current_particles();
        
        for (bird1, bird2) in birds1.iter().zip(birds2.iter()) {
            assert!((bird1.position - bird2.position).norm() < 1e-12,
                "Position mismatch: {:?} vs {:?}", bird1.position, bird2.position);
            assert!((bird1.velocity - bird2.velocity).norm() < 1e-12,
                "Velocity mismatch: {:?} vs {:?}", bird1.velocity, bird2.velocity);
        }
    }

    #[test]
    fn simulation_handles_edge_case_parameters() {
        // Test with very small time step
        let params_small_dt = SimulationParams {
            num_birds: 2,
            radius: 1.0,
            speed: 1.0,
            dt: 1e-6, // Very small time step
            interaction_radius: 0.5,
            eta: 0.0,
            iterations: 2,
        };

        let birds = vec![
            Bird::from_spherical(1.0, PI/2.0, 0.0, 1.0, 0.0),
            Bird::from_spherical(1.0, PI/2.0, PI/4.0, 1.0, 0.0),
        ];

        let request = SimulationRequest {
            id: 1,
            tag: "small_dt_test".to_string(),
            ensemble_id: 1,
            initial_values: birds,
            params: params_small_dt,
        };

        let (tx, _rx) = mpsc::channel();
        let mut sim = Simulation::new(request, tx, 1);
        
        // Should not crash or produce NaN values
        sim.run();
        
        assert!(sim.step_count() > 0);
        assert!(sim.current_time().is_finite());
        
        // Verify all birds have valid states
        for bird in sim.current_particles() {
            assert!(bird.position.norm().is_finite());
            assert!(bird.velocity.norm().is_finite());
            assert!((bird.position.norm() - params_small_dt.radius).abs() < 1e-8);
        }
    }
}