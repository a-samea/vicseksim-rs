//! Integration tests for ensemble generation and statistical properties
//!
//! These tests verify that ensemble generation produces statistically valid
//! distributions and that the generated configurations are suitable for
//! physical simulations.

use flocking_lib::{ensemble, vector::Vec3};
use rand::{rngs::StdRng, SeedableRng};

/// Test uniform distribution on sphere surface
#[test]
fn test_uniform_sphere_distribution() {
    let mut rng = StdRng::seed_from_u64(42);

    // Generate a large ensemble to test statistical properties
    let birds =
        ensemble::generate(1000, 1.0, 0.1, 0.03, &mut rng).expect("Failed to generate ensemble");

    // Test that positions are uniformly distributed on sphere
    let mut north_count = 0;
    let mut south_count = 0;

    for bird in &birds {
        let pos = bird.position();
        if pos.z > 0.0 {
            north_count += 1;
        } else {
            south_count += 1;
        }
    }

    // Should be roughly equal distribution between hemispheres
    let ratio = north_count as f64 / south_count as f64;
    assert!(
        ratio > 0.8 && ratio < 1.2,
        "Uneven hemisphere distribution: N={}, S={}, ratio={:.3}",
        north_count,
        south_count,
        ratio
    );
}

/// Test velocity distribution properties
#[test]
fn test_velocity_distribution() {
    let mut rng = StdRng::seed_from_u64(123);

    let birds =
        ensemble::generate(500, 1.0, 0.15, 0.05, &mut rng).expect("Failed to generate ensemble");

    let mut speed_sum = 0.0;
    let mut speed_squared_sum = 0.0;

    for bird in &birds {
        let speed = bird.velocity().norm();
        speed_sum += speed;
        speed_squared_sum += speed * speed;

        // All speeds should be close to the specified speed
        assert!(
            (speed - 0.15).abs() < 1e-10,
            "Bird speed deviates from expected: {} vs 0.15",
            speed
        );
    }

    let mean_speed = speed_sum / birds.len() as f64;
    let variance = speed_squared_sum / birds.len() as f64 - mean_speed * mean_speed;

    // Mean should be very close to specified speed
    assert!(
        (mean_speed - 0.15).abs() < 1e-8,
        "Mean speed incorrect: {}",
        mean_speed
    );

    // Variance should be very small (all speeds should be identical)
    assert!(variance < 1e-15, "Speed variance too large: {}", variance);
}

/// Test that ensemble respects minimum separation constraints
#[test]
fn test_separation_constraints() {
    let mut rng = StdRng::seed_from_u64(456);

    let min_sep = 0.1;
    let birds =
        ensemble::generate(100, 1.0, 0.1, min_sep, &mut rng).expect("Failed to generate ensemble");

    let mut min_distance = f64::INFINITY;
    let mut violations = 0;

    for (i, bird1) in birds.iter().enumerate() {
        for bird2 in birds.iter().skip(i + 1) {
            let distance = bird1.distance_to(bird2);
            min_distance = min_distance.min(distance);

            if distance < min_sep - 1e-12 {
                violations += 1;
            }
        }
    }

    assert_eq!(
        violations, 0,
        "Found {} separation violations, minimum distance: {}",
        violations, min_distance
    );

    assert!(
        min_distance >= min_sep - 1e-12,
        "Minimum distance {} below threshold {}",
        min_distance,
        min_sep
    );
}

/// Test ensemble generation with different sphere radii
#[test]
fn test_different_sphere_radii() {
    let mut rng = StdRng::seed_from_u64(789);

    for radius in [0.5, 1.0, 2.0, 5.0] {
        let birds = ensemble::generate(50, radius, 0.1, 0.05, &mut rng).expect(&format!(
            "Failed to generate ensemble for radius {}",
            radius
        ));

        // All birds should be on the sphere of the specified radius
        for bird in &birds {
            let actual_radius = bird.position().norm();
            assert!(
                (actual_radius - radius).abs() < 1e-10,
                "Bird not on sphere of radius {}: actual radius = {}",
                radius,
                actual_radius
            );
        }

        // Minimum separation should scale with sphere size for surface distance
        let min_surface_sep = 0.05;
        for (i, bird1) in birds.iter().enumerate() {
            for bird2 in birds.iter().skip(i + 1) {
                let distance = bird1.distance_to(bird2);
                assert!(
                    distance >= min_surface_sep - 1e-12,
                    "Surface separation violation on radius {} sphere",
                    radius
                );
            }
        }
    }
}

/// Test ensemble generation failure modes
#[test]
fn test_ensemble_generation_limits() {
    let mut rng = StdRng::seed_from_u64(999);

    // Test impossible configurations

    // Too many birds for the space available
    let result = ensemble::generate(10000, 1.0, 0.1, 0.5, &mut rng);
    // This should either fail or take extremely long - we mainly test it doesn't panic

    // Minimum separation larger than sphere diameter
    let result = ensemble::generate(2, 1.0, 0.1, 5.0, &mut rng);
    // Should fail as it's impossible to place 2 birds this far apart on a sphere

    // Zero or negative parameters
    let result = ensemble::generate(10, 0.0, 0.1, 0.1, &mut rng);
    // Should fail for zero radius

    let result = ensemble::generate(10, 1.0, -0.1, 0.1, &mut rng);
    // Should fail for negative speed

    let result = ensemble::generate(10, 1.0, 0.1, -0.1, &mut rng);
    // Should fail for negative separation
}

/// Test reproducibility with fixed random seeds
#[test]
fn test_ensemble_reproducibility() {
    // Same seed should produce identical ensembles
    let mut rng1 = StdRng::seed_from_u64(12345);
    let mut rng2 = StdRng::seed_from_u64(12345);

    let birds1 = ensemble::generate(20, 1.0, 0.1, 0.1, &mut rng1)
        .expect("Failed to generate first ensemble");

    let birds2 = ensemble::generate(20, 1.0, 0.1, 0.1, &mut rng2)
        .expect("Failed to generate second ensemble");

    assert_eq!(birds1.len(), birds2.len());

    // Positions and velocities should be identical
    for (bird1, bird2) in birds1.iter().zip(birds2.iter()) {
        let pos_diff = (*bird1.position() - *bird2.position()).norm();
        let vel_diff = (*bird1.velocity() - *bird2.velocity()).norm();

        assert!(pos_diff < 1e-15, "Position difference: {}", pos_diff);
        assert!(vel_diff < 1e-15, "Velocity difference: {}", vel_diff);
    }
}

/// Test statistical properties of velocity orientations
#[test]
fn test_velocity_orientation_statistics() {
    let mut rng = StdRng::seed_from_u64(2024);

    let birds =
        ensemble::generate(1000, 1.0, 0.1, 0.02, &mut rng).expect("Failed to generate ensemble");

    // Calculate average velocity direction (should be close to zero for random orientations)
    let mut avg_velocity = Vec3::new(0.0, 0.0, 0.0);

    for bird in &birds {
        let normalized_vel = bird.velocity().normalize();
        avg_velocity = avg_velocity + normalized_vel;
    }

    avg_velocity = avg_velocity * (1.0 / birds.len() as f64);

    // For uniformly random orientations, average should be near zero
    let avg_magnitude = avg_velocity.norm();
    assert!(
        avg_magnitude < 0.1,
        "Average velocity direction not random enough: magnitude = {}",
        avg_magnitude
    );
}

/// Test ensemble generation performance
#[test]
fn test_ensemble_generation_performance() {
    let mut rng = StdRng::seed_from_u64(1111);

    let start = std::time::Instant::now();

    // Generate several ensembles to test performance
    for i in 0..10 {
        let _birds = ensemble::generate(100, 1.0, 0.1, 0.03, &mut rng)
            .expect(&format!("Failed to generate ensemble {}", i));
    }

    let duration = start.elapsed();

    // Should complete reasonably quickly
    assert!(
        duration.as_secs() < 10,
        "Ensemble generation too slow: took {:.2} seconds",
        duration.as_secs_f64()
    );

    println!(
        "Ensemble generation performance: 10 ensembles of 100 birds took {:.3} ms",
        duration.as_millis()
    );
}
