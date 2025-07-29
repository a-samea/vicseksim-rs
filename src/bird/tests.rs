#[cfg(test)]
mod units {
    use crate::bird::Bird;
    use crate::vector::Vec3;
    use std::f64::consts::PI;

    #[test]
    fn distance_from() {
        let radius = 1.0;
        let speed = 2.0;
        let b1 = Bird::from_spherical(radius, 0.0, 0.0, speed, 0.0);
        let b2 = Bird::from_spherical(radius, PI / 2.0, 0.0, speed, 0.0);
        let b3 = Bird::from_spherical(radius, PI, 0.0, speed, 0.0);

        assert!(b1.distance_from(&b1, radius).abs() < f64::EPSILON);
        assert!((b1.distance_from(&b2, radius) - (PI / 2.0 * radius)).abs() < f64::EPSILON);
        assert!((b1.distance_from(&b3, radius) - (PI * radius)).abs() < f64::EPSILON);
        assert!((b2.distance_from(&b3, radius) - (PI / 2.0 * radius)).abs() < f64::EPSILON);
    }

    #[test]
    fn parallel_transport_velocity() {
        let radius = 1.0;
        let speed = 1.0;

        // Test case 1: Orthogonal positions
        let b1 = Bird::from_spherical(radius, PI / 2.0, 0.0, speed, 0.0); // (1,0,0) moving in phi direction
        let b2 = Bird::from_spherical(radius, PI / 2.0, PI / 2.0, speed, 0.0); // (0,1,0)

        let transported = b1.parallel_transport_velocity(&b2);

        // Verify norm preservation
        assert!((transported.norm() - b1.velocity.norm()).abs() < 1e-10);

        // Verify tangency to sphere at target position
        assert!(transported.dot(&b2.position).abs() < 1e-10);

        // Test case 2: Nearby positions (small angle transport)
        let b3 = Bird::from_spherical(radius, PI / 2.0, 0.1, speed, 0.0);
        let b4 = Bird::from_spherical(radius, PI / 2.0, 0.15, speed, 0.0);

        let transported_small = b3.parallel_transport_velocity(&b4);
        assert!((transported_small.norm() - b3.velocity.norm()).abs() < 1e-10);
        assert!(transported_small.dot(&b4.position).abs() < 1e-10);

        // Test case 3: Round-trip consistency
        let b5 = Bird::from_spherical(radius, PI / 3.0, PI / 4.0, speed, PI / 6.0);
        let b6 = Bird::from_spherical(radius, 2.0 * PI / 3.0, 3.0 * PI / 4.0, speed, 0.0);

        let forward = b5.parallel_transport_velocity(&b6);
        let backward = Bird::new(b6.position, forward).parallel_transport_velocity(&b5);

        // Should be close to original velocity (within numerical precision)
        assert!((backward - b5.velocity).norm() < 1e-12);
    }

    #[test]
    fn parallel_transport_special_cases() {
        // Test parallel transport edge cases: identical positions (no transport needed),
        // antipodal positions (ambiguous axis), and transport with zero velocity.

        let radius = 1.0;
        let speed = 1.0;

        // Case 1: Identical positions
        let b1 = Bird::from_spherical(radius, PI / 4.0, PI / 3.0, speed, PI / 2.0);
        let transported_identity = b1.parallel_transport_velocity(&b1);

        assert!((transported_identity - b1.velocity).norm() < 1e-15);

        // Case 2: Zero velocity
        let b2 = Bird::new(Vec3::new(1.0, 0.0, 0.0), Vec3::zero());
        let b3 = Bird::new(Vec3::new(0.0, 1.0, 0.0), Vec3::zero());
        let transported_zero = b2.parallel_transport_velocity(&b3);

        assert!(transported_zero.norm() < 1e-15);

        // Case 3: Antipodal positions (should handle gracefully)
        let b4 = Bird::new(Vec3::new(1.0, 0.0, 0.0), Vec3::new(0.0, 1.0, 0.0));
        let b5 = Bird::new(Vec3::new(-1.0, 0.0, 0.0), Vec3::zero());
        let transported_antipodal = b4.parallel_transport_velocity(&b5);

        // Should preserve norm even in degenerate case
        assert!((transported_antipodal.norm() - b4.velocity.norm()).abs() < 1e-10);
    }

    #[test]
    fn random_angle_noise() {
        let order_param = 0.5;
        let sample_size = 10000;
        let mut angles = Vec::new();

        for _ in 0..sample_size {
            angles.push(Bird::random_angle_noise(order_param));
        }

        // Test mean (should be close to 0)
        let mean: f64 = angles.iter().sum::<f64>() / sample_size as f64;
        assert!(mean.abs() < 0.05); // Within 5% tolerance

        // Test standard deviation (should be close to order_param)
        let variance: f64 =
            angles.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / sample_size as f64;
        let std_dev = variance.sqrt();

        assert!((std_dev - order_param).abs() < 0.05); // Within 5% tolerance

        // Test with different order parameters
        let small_order = 0.1;
        let large_order = 2.0;

        let small_noise = Bird::random_angle_noise(small_order);
        let large_noise = Bird::random_angle_noise(large_order);

        // These are probabilistic tests, but with high probability:
        // Small order parameter should produce smaller angles
        // We just verify the function doesn't panic and returns finite values
        assert!(small_noise.is_finite());
        assert!(large_noise.is_finite());
    }

    #[test]
    fn random_angle_noise_parameter_validation() {
        // Test very small positive value (should work)
        let tiny_param = 1e-10;
        let result = Bird::random_angle_noise(tiny_param);
        assert!(result.is_finite());

        // Test normal values
        let normal_param = 1.0;
        let result2 = Bird::random_angle_noise(normal_param);
        assert!(result2.is_finite());

        // Test large values (should still work)
        let large_param = 100.0;
        let result3 = Bird::random_angle_noise(large_param);
        assert!(result3.is_finite());
    }

    #[test]
    fn add_noise() {
        let radius = 1.0;
        let base_velocity = Vec3::new(1.0, 0.0, 0.0);

        // Test at equator
        let base_bird = Bird::new(Vec3::new(0.0, 0.0, radius), Vec3::zero());
        let noisy_velocity = Bird::add_noise(base_velocity, &base_bird, 0.1);

        // norm should be preserved
        assert!((noisy_velocity.norm() - base_velocity.norm()).abs() < 1e-10);

        // Should remain tangent to sphere (perpendicular to position)
        assert!(noisy_velocity.dot(&base_bird.position).abs() < 1e-10);

        // Test at different positions
        let positions = vec![
            Vec3::new(radius, 0.0, 0.0), // On x-axis
            Vec3::new(0.0, radius, 0.0), // On y-axis
            Vec3::new(0.0, 0.0, radius), // North pole
            Vec3::new(radius / 2.0_f64.sqrt(), radius / 2.0_f64.sqrt(), 0.0), // 45° on equator
        ];

        for pos in positions {
            let bird = Bird::new(pos, Vec3::zero());
            let test_velocity = Vec3::new(0.0, 1.0, 0.0);
            let noisy = Bird::add_noise(test_velocity, &bird, 0.2);

            // Basic invariants
            assert!((noisy.norm() - test_velocity.norm()).abs() < 1e-10);
        }

        // Test with different noise strengths
        let noise_levels = vec![0.01, 0.1, 0.5, 1.0, 2.0];
        let test_bird = Bird::new(Vec3::new(1.0, 0.0, 0.0), Vec3::zero());

        for noise in noise_levels {
            let result = Bird::add_noise(Vec3::new(0.0, 1.0, 0.0), &test_bird, noise);
            assert!((result.norm() - 1.0).abs() < 1e-10);
        }
    }

    #[test]
    fn move_bird() {
        // Test bird movement along sphere surface using geodesic motion.
        // Verifies that moved positions remain on sphere surface (constant radius),
        // movement direction follows velocity vector, and displacement norm
        // scales correctly with speed and time step. Tests various time steps
        // and speeds including edge cases.

        let radius = 1.0;
        let dt = 0.1;
        let speed = 2.0;

        // Test basic movement
        let initial_pos = Vec3::new(1.0, 0.0, 0.0);
        let velocity = Vec3::new(0.0, 1.0, 0.0); // Tangent at initial position

        let new_pos = Bird::move_bird(&initial_pos, &velocity, dt, radius, speed);

        // Should remain on sphere surface
        assert!((new_pos.norm() - radius).abs() < 1e-10);

        // Test movement direction (should be in general direction of velocity)
        let displacement = new_pos - initial_pos;
        assert!(displacement.dot(&velocity) > 0.0); // Positive correlation

        // Test with different time steps
        let time_steps = vec![0.01, 0.1, 0.5, 1.0];
        for test_dt in time_steps {
            let moved = Bird::move_bird(&initial_pos, &velocity, test_dt, radius, speed);
            assert!((moved.norm() - radius).abs() < 1e-10);

            // Larger time steps should produce larger displacements
            let angle_moved = initial_pos.angle_between(&moved);
            let expected_angle = speed * test_dt / radius;
            assert!((angle_moved - expected_angle).abs() < 1e-10);
        }

        // Test with different speeds
        let speeds = vec![0.1, 1.0, 5.0, 10.0];
        for test_speed in speeds {
            let moved = Bird::move_bird(&initial_pos, &velocity, dt, radius, test_speed);
            assert!((moved.norm() - radius).abs() < 1e-10);
        }
    }

    #[test]
    fn bird_geodesic_properties() {
        let radius = 1.0;
        let speed = 1.0;
        let total_time = 1.0;

        // Test great circle motion: start at north pole, move east
        let start_pos = Vec3::new(0.0, 0.0, radius);
        let east_velocity = Vec3::new(1.0, 0.0, 0.0); // Eastward at north pole

        // Compare one large step vs many small steps
        let large_step = Bird::move_bird(&start_pos, &east_velocity, total_time, radius, speed);

        let num_small_steps = 100;
        let small_dt = total_time / num_small_steps as f64;
        let mut current_pos = start_pos;

        for _ in 0..num_small_steps {
            current_pos = Bird::move_bird(&current_pos, &east_velocity, small_dt, radius, speed);
            println!("Current position: {:?} {}", current_pos, current_pos.norm());
        }

        // Small steps should approximate large step (within numerical tolerance)
        assert!((large_step - current_pos).norm() < 1e-6);

        // Test circular motion around a fixed axis
        let axis_pos = Vec3::new(1.0, 0.0, 0.0);
        let tangent_velocity = Vec3::new(0.0, 1.0, 0.0);

        let mut pos = axis_pos;
        let small_dt = 0.01;
        let steps = (2.0 * PI * radius / speed / small_dt) as usize; // One full revolution

        for _ in 0..steps {
            pos = Bird::move_bird(&pos, &tangent_velocity, small_dt, radius, speed);
        }

        // Should return close to starting position after full revolution
        assert!((pos - axis_pos).norm() < 0.1);

        // Test that velocity direction affects trajectory correctly
        let test_pos = Vec3::new(0.0, 0.0, 1.0);
        let vel_x = Vec3::new(1.0, 0.0, 0.0);
        let vel_y = Vec3::new(0.0, 1.0, 0.0);

        let moved_x = Bird::move_bird(&test_pos, &vel_x, 0.1, radius, speed);
        let moved_y = Bird::move_bird(&test_pos, &vel_y, 0.1, radius, speed);

        // Movements should be in different directions
        assert!((moved_x - moved_y).norm() > 0.1);
    }

    #[test]
    fn test_physics_integration() {
        // Integration test combining multiple physics functions. Tests complete
        // simulation step: distance calculation between neighbors, parallel
        // transport for velocity averaging, noise addition, and position update.
        // Verifies that combined operations maintain physical constraints.

        let radius = 1.0;
        let dt = 0.1;
        let speed = 1.0;
        let noise_level = 0.1;

        // Create a small flock
        let mut birds = vec![
            Bird::from_spherical(radius, PI / 2.0, 0.0, speed, 0.0),
            Bird::from_spherical(radius, PI / 2.0, 0.2, speed, PI / 4.0),
            Bird::from_spherical(radius, PI / 2.0, -0.2, speed, -PI / 4.0),
        ];

        // Simulate one complete step
        for i in 0..birds.len() {
            let mut neighbor_velocities = Vec::new();

            // Collect neighbor velocities via parallel transport
            for j in 0..birds.len() {
                if i != j {
                    let distance = birds[i].distance_from(&birds[j], radius);
                    if distance < 1.0 {
                        // Within interaction range
                        let transported = birds[j].parallel_transport_velocity(&birds[i]);
                        neighbor_velocities.push(transported);
                    }
                }
            }

            // Average neighbor velocities
            let mut avg_velocity = birds[i].velocity;
            if !neighbor_velocities.is_empty() {
                avg_velocity = neighbor_velocities
                    .iter()
                    .fold(Vec3::zero(), |acc, &v| acc + v)
                    * (1.0 / neighbor_velocities.len() as f64);
            }

            // Add noise
            let noisy_velocity = Bird::add_noise(avg_velocity, &birds[i], noise_level);

            // Update position
            let new_position =
                Bird::move_bird(&birds[i].position, &noisy_velocity, dt, radius, speed);

            // Verify physical constraints
            assert!((new_position.norm() - radius).abs() < 1e-10);
            assert!((noisy_velocity.norm() - avg_velocity.norm()).abs() < 1e-10);
            assert!(noisy_velocity.dot(&birds[i].position).abs() < 1e-10);

            // Update bird
            birds[i].position = new_position;
            birds[i].velocity = noisy_velocity;
        }

        // Verify all birds still on sphere
        for bird in &birds {
            assert!((bird.position.norm() - radius).abs() < 1e-10);
        }
    }

    #[test]
    fn test_physics_numerical_stability() {
        // Test numerical stability of physics functions under extreme conditions.
        // Tests behavior with very small time steps, high speeds, large noise,
        // and positions near coordinate singularities (poles). Validates
        // graceful degradation and error bounds.

        let radius = 1.0;

        // Test extreme time steps
        let tiny_dt = 1e-12;
        let huge_dt = 1000.0;
        let normal_pos = Vec3::new(1.0, 0.0, 0.0);
        let normal_vel = Vec3::new(0.0, 1.0, 0.0);

        let tiny_move = Bird::move_bird(&normal_pos, &normal_vel, tiny_dt, radius, 1.0);
        let huge_move = Bird::move_bird(&normal_pos, &normal_vel, huge_dt, radius, 1.0);

        assert!((tiny_move.norm() - radius).abs() < 1e-10);
        assert!((huge_move.norm() - radius).abs() < 1e-10);

        // Test extreme speeds
        let tiny_speed = 1e-10;
        let huge_speed = 1e10;

        let slow_move = Bird::move_bird(&normal_pos, &normal_vel, 0.1, radius, tiny_speed);
        let fast_move = Bird::move_bird(&normal_pos, &normal_vel, 0.1, radius, huge_speed);

        assert!((slow_move.norm() - radius).abs() < 1e-10);
        assert!((fast_move.norm() - radius).abs() < 1e-10);

        // Test positions near poles (coordinate singularities)
        let near_north = Vec3::new(1e-10, 1e-10, radius);
        let near_south = Vec3::new(1e-10, 1e-10, -radius);

        let north_bird = Bird::new(near_north, Vec3::new(1.0, 0.0, 0.0));
        let south_bird = Bird::new(near_south, Vec3::new(1.0, 0.0, 0.0));

        // Distance calculation should be stable
        let pole_distance = north_bird.distance_from(&south_bird, radius);
        assert!(pole_distance.is_finite());
        assert!(pole_distance > 0.0);

        // Parallel transport should handle poles gracefully
        let transported_at_pole = north_bird.parallel_transport_velocity(&south_bird);
        assert!(transported_at_pole.norm().is_finite());

        // Movement near poles should be stable
        let moved_near_pole =
            Bird::move_bird(&near_north, &Vec3::new(1.0, 0.0, 0.0), 0.1, radius, 1.0);
        assert!((moved_near_pole.norm() - radius).abs() < 1e-10);

        // Test extreme noise levels
        let extreme_noise = 100.0;
        let noisy_extreme = Bird::add_noise(
            normal_vel,
            &Bird::new(normal_pos, Vec3::zero()),
            extreme_noise,
        );
        assert!(noisy_extreme.norm().is_finite());
        assert!((noisy_extreme.norm() - normal_vel.norm()).abs() < 1e-10);

        // Test with very small vectors
        let tiny_vel = Vec3::new(1e-15, 1e-15, 0.0);
        let tiny_result = Bird::add_noise(tiny_vel, &Bird::new(normal_pos, Vec3::zero()), 0.1);
        assert!(tiny_result.norm().is_finite());
    }
}
