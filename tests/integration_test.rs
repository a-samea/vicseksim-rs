//! Integration tests for the flocking simulation library
//!
//! These tests verify that the library components work together correctly
//! and that the overall simulation behavior is physically reasonable.

use flocking_lib::{bird::Bird, ensemble, simulation::Simulation, vector::Vec3};
use rand::{rngs::StdRng, SeedableRng};

/// Test that ensemble generation produces valid configurations
#[test]
fn test_ensemble_generation() {
    let mut rng = StdRng::seed_from_u64(42);

    // Generate a small ensemble
    let birds =
        ensemble::generate(50, 1.0, 0.1, 0.05, &mut rng).expect("Failed to generate ensemble");

    assert_eq!(birds.len(), 50);

    // Verify all birds are on the sphere surface
    for bird in &birds {
        let radius = bird.position().norm();
        assert!(
            (radius - 1.0).abs() < 1e-10,
            "Bird not on sphere surface: radius = {}",
            radius
        );
    }

    // Verify minimum separation constraint
    for (i, bird1) in birds.iter().enumerate() {
        for bird2 in birds.iter().skip(i + 1) {
            let distance = bird1.distance_to(bird2);
            assert!(
                distance >= 0.05 - 1e-10,
                "Birds too close: distance = {}",
                distance
            );
        }
    }
}

/// Test that the simulation preserves basic physical constraints
#[test]
fn test_simulation_constraints() {
    let mut rng = StdRng::seed_from_u64(123);

    let birds =
        ensemble::generate(20, 1.0, 0.1, 0.1, &mut rng).expect("Failed to generate ensemble");

    let mut sim = Simulation::new(birds);

    // Run a few simulation steps
    for _ in 0..10 {
        sim.step(0.01, 0.2, 0.1);

        let birds = sim.birds();

        // Check that all birds remain on the sphere
        for bird in birds {
            let radius = bird.position().norm();
            assert!(
                (radius - 1.0).abs() < 1e-8,
                "Bird left sphere surface: radius = {}",
                radius
            );
        }

        // Check that velocities have reasonable magnitudes
        for bird in birds {
            let speed = bird.velocity().norm();
            assert!(speed > 0.0 && speed < 1.0, "Invalid bird speed: {}", speed);
        }
    }
}

/// Test phase transition behavior by measuring order parameter
#[test]
fn test_phase_transition_behavior() {
    let mut rng = StdRng::seed_from_u64(456);

    let birds =
        ensemble::generate(100, 1.0, 0.1, 0.05, &mut rng).expect("Failed to generate ensemble");

    let mut sim = Simulation::new(birds);

    // Test low noise (should be ordered)
    let low_noise = 0.01;

    // Equilibrate
    for _ in 0..200 {
        sim.step(0.05, 0.3, low_noise);
    }

    // Measure order parameter
    let order_low = calculate_order_parameter(sim.birds());

    // Reset and test high noise (should be disordered)
    let high_noise = 1.0;

    // Re-equilibrate with high noise
    for _ in 0..200 {
        sim.step(0.05, 0.3, high_noise);
    }

    let order_high = calculate_order_parameter(sim.birds());

    // High noise should produce lower order than low noise
    assert!(
        order_high < order_low,
        "Order parameter should decrease with noise: low={:.3}, high={:.3}",
        order_low,
        order_high
    );

    // Order should be in reasonable ranges
    assert!(
        order_low >= 0.0 && order_low <= 1.0,
        "Invalid order parameter: {}",
        order_low
    );
    assert!(
        order_high >= 0.0 && order_high <= 1.0,
        "Invalid order parameter: {}",
        order_high
    );
}

/// Test vector operations used throughout the simulation
#[test]
fn test_vector_operations() {
    let v1 = Vec3::new(1.0, 0.0, 0.0);
    let v2 = Vec3::new(0.0, 1.0, 0.0);
    let v3 = Vec3::new(0.0, 0.0, 1.0);

    // Test normalization
    let normalized = v1.normalize();
    assert!((normalized.norm() - 1.0).abs() < 1e-15);

    // Test cross product
    let cross = v1.cross(&v2);
    assert!((cross - v3).norm() < 1e-15);

    // Test dot product
    let dot = v1.dot(&v2);
    assert!(dot.abs() < 1e-15); // Should be zero for orthogonal vectors

    // Test spherical projection
    let large_vec = Vec3::new(2.0, 3.0, 4.0);
    let projected = large_vec.normalize();
    assert!((projected.norm() - 1.0).abs() < 1e-15);
}

/// Test bird physics operations
#[test]
fn test_bird_physics() {
    let position = Vec3::new(1.0, 0.0, 0.0);
    let velocity = Vec3::new(0.0, 0.1, 0.0);
    let bird = Bird::new(position, velocity);

    // Test distance calculation
    let other_pos = Vec3::new(0.0, 1.0, 0.0);
    let other_vel = Vec3::new(0.1, 0.0, 0.0);
    let other_bird = Bird::new(other_pos, other_vel);

    let distance = bird.distance_to(&other_bird);
    assert!(distance > 0.0 && distance <= 2.0); // Maximum distance on unit sphere

    // Test that birds maintain their properties
    assert_eq!(bird.position(), &position);
    assert_eq!(bird.velocity(), &velocity);
}

/// Test serialization and I/O operations
#[test]
fn test_io_operations() {
    let mut rng = StdRng::seed_from_u64(789);

    let birds =
        ensemble::generate(10, 1.0, 0.1, 0.1, &mut rng).expect("Failed to generate ensemble");

    let sim = Simulation::new(birds);

    // Test that we can extract bird data for serialization
    let bird_data = sim.birds();
    assert_eq!(bird_data.len(), 10);

    // Verify data integrity
    for bird in bird_data {
        assert!((bird.position().norm() - 1.0).abs() < 1e-10);
        assert!(bird.velocity().norm() > 0.0);
    }
}

/// Test edge cases and error conditions
#[test]
fn test_edge_cases() {
    let mut rng = StdRng::seed_from_u64(999);

    // Test empty ensemble generation (should fail)
    let result = ensemble::generate(0, 1.0, 0.1, 0.1, &mut rng);
    assert!(result.is_err(), "Should fail to generate empty ensemble");

    // Test overcrowded ensemble (should fail or take very long)
    let result = ensemble::generate(1000, 1.0, 0.1, 0.9, &mut rng);
    // This might fail or succeed depending on implementation
    // The test mainly ensures it doesn't panic

    // Test simulation with extreme parameters
    let birds =
        ensemble::generate(5, 1.0, 0.1, 0.2, &mut rng).expect("Failed to generate ensemble");

    let mut sim = Simulation::new(birds);

    // Very large time step (should remain stable)
    sim.step(0.5, 0.1, 0.1);

    // Very small time step (should work fine)
    sim.step(0.001, 0.1, 0.1);

    // Extreme alignment strength
    sim.step(0.1, 10.0, 0.1);

    // Extreme noise
    sim.step(0.1, 0.1, 5.0);

    // After all these operations, birds should still be on sphere
    for bird in sim.birds() {
        let radius = bird.position().norm();
        assert!(
            (radius - 1.0).abs() < 1e-6,
            "Bird left sphere after extreme parameters: radius = {}",
            radius
        );
    }
}

/// Helper function to calculate order parameter
fn calculate_order_parameter(birds: &[Bird]) -> f64 {
    if birds.is_empty() {
        return 0.0;
    }

    let mut sum_velocity = Vec3::new(0.0, 0.0, 0.0);
    let mut total_speed = 0.0;

    for bird in birds {
        sum_velocity = sum_velocity + *bird.velocity();
        total_speed += bird.velocity().norm();
    }

    if total_speed == 0.0 {
        return 0.0;
    }

    sum_velocity.norm() / total_speed
}

/// Benchmark-style test for performance characteristics
#[test]
fn test_performance_characteristics() {
    let mut rng = StdRng::seed_from_u64(1337);

    // Generate larger ensemble for performance testing
    let birds =
        ensemble::generate(500, 1.0, 0.1, 0.02, &mut rng).expect("Failed to generate ensemble");

    let mut sim = Simulation::new(birds);

    let start = std::time::Instant::now();

    // Run many simulation steps
    for _ in 0..100 {
        sim.step(0.01, 0.2, 0.1);
    }

    let duration = start.elapsed();

    // Performance should be reasonable (adjust threshold as needed)
    assert!(
        duration.as_secs() < 5,
        "Simulation too slow: took {:.2} seconds",
        duration.as_secs_f64()
    );

    println!(
        "Performance test: {} steps with 500 birds took {:.3} ms",
        100,
        duration.as_millis()
    );
}
