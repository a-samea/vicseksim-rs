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
            Vec3::new(0.0, 0.0, radius), // North Pole
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
    fn move_on_sphere() {
        let radius = 1.0;
        let dt = 0.1;
        let speed = 2.0;

        // Test basic movement
        let initial_pos = Vec3::new(radius, 0.0, 0.0);
        let velocity = Vec3::new(0.0, speed, 0.0); // Tangent at initial position with correct magnitude
        let bird = Bird::new(initial_pos, velocity);

        let new_bird = bird.move_on_sphere(dt, radius, speed);

        // Should remain on sphere surface
        assert!((new_bird.position.norm() - radius).abs() < 1e-10);

        // Test movement direction (should be in general direction of velocity)
        let displacement = new_bird.position - initial_pos;
        assert!(displacement.dot(&velocity) > 0.0); // Positive correlation

        // Test with different time steps
        let time_steps = vec![0.01, 0.1, 0.5, 1.0];
        for test_dt in time_steps {
            let moved_bird = bird.move_on_sphere(test_dt, radius, speed);
            assert!((moved_bird.position.norm() - radius).abs() < 1e-10);

            // Larger time steps should produce larger displacements
            let angle_moved = initial_pos.angle_between(&moved_bird.position);
            let expected_angle = speed * test_dt / radius;
            assert!((angle_moved - expected_angle).abs() < 1e-10);
        }

        // Test with different speeds
        let speeds = vec![0.1, 1.0, 5.0, 10.0];
        for test_speed in speeds {
            let test_velocity = Vec3::new(0.0, test_speed, 0.0); // Velocity with correct magnitude
            let test_bird = Bird::new(initial_pos, test_velocity);
            let moved_bird = test_bird.move_on_sphere(dt, radius, test_speed);
            assert!((moved_bird.position.norm() - radius).abs() < 1e-10);
        }
    }

    #[test]
    fn bird_geodesic_properties() {
        let radius = 1.0;
        let speed = 1.0;
        let total_time = 1.0;

        // Test great circle motion: start at the North Pole, move east
        let start_pos = Vec3::new(0.0, 0.0, radius);
        let east_velocity = Vec3::new(speed, 0.0, 0.0); // Eastward at the North Pole with correct magnitude
        let start_bird = Bird::new(start_pos, east_velocity);

        // Compare one large step vs many small steps
        let large_step_bird = start_bird.move_on_sphere(total_time, radius, speed);

        let num_small_steps = 100;
        let small_dt = total_time / num_small_steps as f64;
        let mut current_bird = start_bird;

        for _ in 0..num_small_steps {
            current_bird = current_bird.move_on_sphere(small_dt, radius, speed);
        }

        // Small steps should approximate large step (within numerical tolerance)
        assert!((large_step_bird.position - current_bird.position).norm() < 1e-6);

        // Test circular motion around a fixed axis
        let axis_pos = Vec3::new(1.0, 0.0, 0.0);
        let tangent_velocity = Vec3::new(0.0, speed, 0.0); // Tangent velocity with correct magnitude
        let mut current_bird = Bird::new(axis_pos, tangent_velocity);
        let small_dt = 0.01;
        let steps = (2.0 * PI * radius / speed / small_dt) as usize; // One full revolution

        for _ in 0..steps {
            current_bird = current_bird.move_on_sphere(small_dt, radius, speed);
        }

        // Should return close to starting position after full revolution
        assert!((current_bird.position - axis_pos).norm() < 0.1);

        // Test that velocity direction affects trajectory correctly
        let test_pos = Vec3::new(0.0, 0.0, 1.0);
        let vel_x = Vec3::new(speed, 0.0, 0.0); // Velocities with correct magnitude
        let vel_y = Vec3::new(0.0, speed, 0.0);

        let bird_x = Bird::new(test_pos, vel_x);
        let bird_y = Bird::new(test_pos, vel_y);

        let moved_x = bird_x.move_on_sphere(0.1, radius, speed);
        let moved_y = bird_y.move_on_sphere(0.1, radius, speed);

        // Movements should be in different directions
        assert!((moved_x.position - moved_y.position).norm() > 0.1);
    }
}