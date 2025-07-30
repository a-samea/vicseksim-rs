//! Integration tests for simulation physics and behavior
//!
//! These tests verify that the simulation engine correctly implements
//! the Vicsek model physics and produces expected flocking behaviors.

use flocking_lib::{bird::Bird, ensemble, simulation::Simulation, vector::Vec3};
use rand::{rngs::StdRng, SeedableRng};

/// Test that flocking behavior emerges with appropriate parameters
#[test]
fn test_flocking_emergence() {
    let mut rng = StdRng::seed_from_u64(42);

    // Create birds with initially random velocities
    let birds =
        ensemble::generate(200, 1.0, 0.1, 0.03, &mut rng).expect("Failed to generate ensemble");

    let mut sim = Simulation::new(birds);

    // Measure initial order (should be low due to random velocities)
    let initial_order = calculate_order_parameter(sim.birds());

    // Run simulation with strong alignment and low noise
    let alignment_strength = 0.5;
    let noise_strength = 0.05;
    let dt = 0.01;

    // Allow time for flocking to emerge
    for _ in 0..500 {
        sim.step(dt, alignment_strength, noise_strength);
    }

    let final_order = calculate_order_parameter(sim.birds());

    // Order should increase significantly due to flocking
    assert!(
        final_order > initial_order + 0.1,
        "Flocking did not emerge: initial order = {:.3}, final order = {:.3}",
        initial_order,
        final_order
    );

    // Final order should be reasonably high
    assert!(final_order > 0.3, "Final order too low: {:.3}", final_order);
}

/// Test conservation laws and constraints during simulation
#[test]
fn test_conservation_laws() {
    let mut rng = StdRng::seed_from_u64(123);

    let birds =
        ensemble::generate(50, 1.0, 0.15, 0.05, &mut rng).expect("Failed to generate ensemble");

    let mut sim = Simulation::new(birds);

    // Track various quantities that should be conserved or constrained
    for step in 0..100 {
        sim.step(0.02, 0.3, 0.1);

        let birds = sim.birds();

        // All birds must remain on sphere surface
        for (i, bird) in birds.iter().enumerate() {
            let radius = bird.position().norm();
            assert!(
                (radius - 1.0).abs() < 1e-8,
                "Step {}, Bird {}: left sphere surface, radius = {:.10}",
                step,
                i,
                radius
            );
        }

        // Bird speeds should be maintained (approximately)
        for (i, bird) in birds.iter().enumerate() {
            let speed = bird.velocity().norm();
            assert!(
                speed > 0.05 && speed < 0.3,
                "Step {}, Bird {}: invalid speed = {:.6}",
                step,
                i,
                speed
            );
        }

        // Velocities should be roughly tangent to sphere
        for (i, bird) in birds.iter().enumerate() {
            let pos = bird.position().normalize();
            let vel = bird.velocity().normalize();
            let dot_product = pos.dot(&vel).abs();

            // Dot product should be small for tangent vectors
            assert!(
                dot_product < 0.1,
                "Step {}, Bird {}: velocity not tangent, dot = {:.6}",
                step,
                i,
                dot_product
            );
        }
    }
}

/// Test response to different noise levels
#[test]
fn test_noise_response() {
    let mut rng = StdRng::seed_from_u64(456);

    let birds =
        ensemble::generate(100, 1.0, 0.1, 0.04, &mut rng).expect("Failed to generate ensemble");

    let noise_levels = [0.0, 0.1, 0.3, 0.6, 1.0];
    let mut orders = Vec::new();

    for &noise in &noise_levels {
        // Reset simulation for each noise level
        let birds_copy =
            ensemble::generate(100, 1.0, 0.1, 0.04, &mut rng).expect("Failed to generate ensemble");

        let mut sim = Simulation::new(birds_copy);

        // Equilibrate
        for _ in 0..300 {
            sim.step(0.01, 0.4, noise);
        }

        // Measure order parameter
        let order = calculate_order_parameter(sim.birds());
        orders.push(order);

        println!("Noise level: {:.1}, Order parameter: {:.3}", noise, order);
    }

    // Order should generally decrease with increasing noise
    for i in 1..orders.len() {
        if orders[i] > orders[i - 1] + 0.05 {
            panic!(
                "Order increased significantly with noise: {:.3} -> {:.3} at noise {:.1}",
                orders[i - 1],
                orders[i],
                noise_levels[i]
            );
        }
    }

    // Very low noise should produce high order
    assert!(
        orders[0] > 0.5,
        "Low noise should produce high order: {:.3}",
        orders[0]
    );

    // High noise should produce low order
    assert!(
        orders[orders.len() - 1] < 0.3,
        "High noise should produce low order: {:.3}",
        orders[orders.len() - 1]
    );
}

/// Test alignment strength effects
#[test]
fn test_alignment_strength_effects() {
    let mut rng = StdRng::seed_from_u64(789);

    let alignment_strengths = [0.0, 0.1, 0.3, 0.7, 1.0];
    let mut orders = Vec::new();

    for &strength in &alignment_strengths {
        let birds =
            ensemble::generate(80, 1.0, 0.1, 0.04, &mut rng).expect("Failed to generate ensemble");

        let mut sim = Simulation::new(birds);

        // Equilibrate with fixed moderate noise
        for _ in 0..400 {
            sim.step(0.01, strength, 0.2);
        }

        let order = calculate_order_parameter(sim.birds());
        orders.push(order);

        println!(
            "Alignment strength: {:.1}, Order parameter: {:.3}",
            strength, order
        );
    }

    // Order should generally increase with alignment strength
    for i in 1..orders.len() {
        if orders[i] < orders[i - 1] - 0.05 {
            println!(
                "Warning: Order decreased with alignment strength: {:.3} -> {:.3}",
                orders[i - 1],
                orders[i]
            );
        }
    }

    // Strong alignment should overcome moderate noise
    assert!(
        orders[orders.len() - 1] > 0.2,
        "Strong alignment should produce some order: {:.3}",
        orders[orders.len() - 1]
    );
}

/// Test time step stability
#[test]
fn test_time_step_stability() {
    let mut rng = StdRng::seed_from_u64(999);

    let birds =
        ensemble::generate(30, 1.0, 0.1, 0.06, &mut rng).expect("Failed to generate ensemble");

    // Test various time steps
    let time_steps = [0.001, 0.01, 0.05, 0.1, 0.2];

    for &dt in &time_steps {
        let mut sim = Simulation::new(birds.clone());

        // Run simulation and check stability
        for step in 0..50 {
            sim.step(dt, 0.3, 0.1);

            // Check for any instabilities
            for (i, bird) in sim.birds().iter().enumerate() {
                let radius = bird.position().norm();
                let speed = bird.velocity().norm();

                assert!(
                    !radius.is_nan() && !radius.is_infinite(),
                    "NaN/Inf radius at step {}, bird {}, dt = {}",
                    step,
                    i,
                    dt
                );

                assert!(
                    !speed.is_nan() && !speed.is_infinite(),
                    "NaN/Inf speed at step {}, bird {}, dt = {}",
                    step,
                    i,
                    dt
                );

                assert!(
                    (radius - 1.0).abs() < 0.01,
                    "Large radius deviation at step {}, bird {}, dt = {}: radius = {}",
                    step,
                    i,
                    dt,
                    radius
                );
            }
        }

        println!("Time step dt = {:.3} completed successfully", dt);
    }
}

/// Test simulation with extreme parameters
#[test]
fn test_extreme_parameters() {
    let mut rng = StdRng::seed_from_u64(1337);

    let birds =
        ensemble::generate(20, 1.0, 0.1, 0.1, &mut rng).expect("Failed to generate ensemble");

    let mut sim = Simulation::new(birds);

    // Test with extreme alignment (should not cause instability)
    for _ in 0..20 {
        sim.step(0.01, 10.0, 0.1);

        // Check birds still on sphere
        for bird in sim.birds() {
            let radius = bird.position().norm();
            assert!(
                (radius - 1.0).abs() < 1e-6,
                "Extreme alignment caused instability"
            );
        }
    }

    // Test with extreme noise (should not cause crashes)
    for _ in 0..20 {
        sim.step(0.01, 0.1, 5.0);

        for bird in sim.birds() {
            let radius = bird.position().norm();
            assert!(!radius.is_nan(), "Extreme noise caused NaN");
            assert!(
                (radius - 1.0).abs() < 1e-6,
                "Extreme noise caused instability"
            );
        }
    }
}

/// Test cluster formation and persistence
#[test]
fn test_cluster_formation() {
    let mut rng = StdRng::seed_from_u64(2024);

    // Start with a configuration that should form clusters
    let birds =
        ensemble::generate(150, 1.0, 0.1, 0.02, &mut rng).expect("Failed to generate ensemble");

    let mut sim = Simulation::new(birds);

    // Run with parameters that promote clustering
    for _ in 0..800 {
        sim.step(0.01, 0.6, 0.15);
    }

    // Analyze cluster formation
    let birds = sim.birds();
    let clusters = find_clusters(birds, 0.1); // Birds within 0.1 distance are in same cluster

    // Should form multiple clusters (not all birds in one cluster, not all isolated)
    assert!(
        clusters.len() > 1,
        "No clustering occurred: only {} clusters",
        clusters.len()
    );
    assert!(
        clusters.len() < birds.len() / 2,
        "Too many small clusters: {} clusters for {} birds",
        clusters.len(),
        birds.len()
    );

    // Largest cluster should contain a reasonable fraction of birds
    let largest_cluster_size = clusters.iter().map(|c| c.len()).max().unwrap_or(0);
    let fraction_in_largest = largest_cluster_size as f64 / birds.len() as f64;

    assert!(
        fraction_in_largest > 0.1 && fraction_in_largest < 0.9,
        "Unrealistic cluster size distribution: largest cluster has {:.1}% of birds",
        fraction_in_largest * 100.0
    );
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

/// Helper function to find clusters of nearby birds
fn find_clusters(birds: &[Bird], max_distance: f64) -> Vec<Vec<usize>> {
    let mut visited = vec![false; birds.len()];
    let mut clusters = Vec::new();

    for i in 0..birds.len() {
        if visited[i] {
            continue;
        }

        let mut cluster = Vec::new();
        let mut stack = vec![i];

        while let Some(idx) = stack.pop() {
            if visited[idx] {
                continue;
            }

            visited[idx] = true;
            cluster.push(idx);

            // Find neighbors
            for j in 0..birds.len() {
                if !visited[j] && birds[idx].distance_to(&birds[j]) <= max_distance {
                    stack.push(j);
                }
            }
        }

        clusters.push(cluster);
    }

    clusters
}
