#[cfg(test)]
mod units {
    use super::super::Vec3;
    use std::f64::consts::PI;

    #[test]
    fn normalize_very_small_vector() {
        let v = Vec3::new(1e-20, 1e-20, 1e-20);
        let normalized = v.normalize();
        assert_eq!(normalized, Vec3::zero());
    }

    #[test]
    fn cross_product() {
        // Standard basis vectors
        let x = Vec3::new(1.0, 0.0, 0.0);
        let y = Vec3::new(0.0, 1.0, 0.0);
        let z = Vec3::new(0.0, 0.0, 1.0);

        assert_eq!(x.cross(&y), z);
        assert_eq!(y.cross(&z), x);
        assert_eq!(z.cross(&x), y);

        // Anti-commutative property
        assert_eq!(y.cross(&x), Vec3::new(0.0, 0.0, -1.0));

        // General case
        let v1 = Vec3::new(1.0, 2.0, 3.0);
        let v2 = Vec3::new(4.0, 5.0, 6.0);
        let result = v1.cross(&v2);
        assert_eq!(result, Vec3::new(-3.0, 6.0, -3.0));
    }

    #[test]
    fn angle_between() {
        let x = Vec3::new(1.0, 0.0, 0.0);
        let y = Vec3::new(0.0, 1.0, 0.0);

        // 90 degrees
        assert!((x.angle_between(&y) - PI / 2.0).abs() < f64::EPSILON);

        // 0 degrees (same direction)
        assert!(x.angle_between(&x).abs() < f64::EPSILON);

        // 180 degrees (opposite direction)
        let neg_x = Vec3::new(-1.0, 0.0, 0.0);
        assert!((x.angle_between(&neg_x) - PI).abs() < f64::EPSILON);

        // 45 degrees
        let diagonal = Vec3::new(1.0, 1.0, 0.0);
        assert!((x.angle_between(&diagonal) - PI / 4.0).abs() < f64::EPSILON);
    }

    #[test]
    fn angle_between_zero_vectors() {
        let zero = Vec3::zero();
        let v = Vec3::new(1.0, 0.0, 0.0);

        assert_eq!(zero.angle_between(&v), 0.0);
        assert_eq!(v.angle_between(&zero), 0.0);
        assert_eq!(zero.angle_between(&zero), 0.0);
    }

    #[test]
    fn project_onto() {
        let v = Vec3::new(3.0, 4.0, 0.0);
        let onto = Vec3::new(1.0, 0.0, 0.0);

        let projection = v.project_onto(&onto);
        assert_eq!(projection, Vec3::new(3.0, 0.0, 0.0));

        // Project onto diagonal
        let diagonal = Vec3::new(1.0, 1.0, 0.0);
        let proj_diag = v.project_onto(&diagonal);
        assert!((proj_diag.x - 3.5).abs() < f64::EPSILON);
        assert!((proj_diag.y - 3.5).abs() < f64::EPSILON);
        assert_eq!(proj_diag.z, 0.0);
    }

    #[test]
    fn project_onto_zero_vector() {
        let v = Vec3::new(1.0, 2.0, 3.0);
        let zero = Vec3::zero();

        let projection = v.project_onto(&zero);
        assert_eq!(projection, Vec3::zero());
    }

    #[test]
    fn approx_eq() {
        let v1 = Vec3::new(1.0, 2.0, 3.0);
        let v2 = Vec3::new(1.0000001, 2.0000001, 3.0000001);
        let v3 = Vec3::new(1.1, 2.1, 3.1);

        assert!(v1.approx_eq(&v2, 1e-6));
        assert!(!v1.approx_eq(&v2, 1e-8));
        assert!(!v1.approx_eq(&v3, 1e-6));
        assert!(v1.approx_eq(&v3, 0.2));
    }

    #[test]
    fn zero_scalar_multiplication() {
        let v = Vec3::new(1.0, 2.0, 3.0);
        let result = v * 0.0;

        assert_eq!(result, Vec3::zero());
    }

    #[test]
    fn negative_scalar_multiplication() {
        let v = Vec3::new(1.0, 2.0, 3.0);
        let result = v * -1.0;

        assert_eq!(result, Vec3::new(-1.0, -2.0, -3.0));
    }

    #[test]
    fn vector_properties() {
        let v1 = Vec3::new(1.0, 2.0, 3.0);
        let v2 = Vec3::new(4.0, 5.0, 6.0);
        let v3 = Vec3::new(7.0, 8.0, 9.0);

        // Associativity of addition
        assert_eq!((v1 + v2) + v3, v1 + (v2 + v3));

        // Commutativity of addition
        assert_eq!(v1 + v2, v2 + v1);

        // Identity element
        assert_eq!(v1 + Vec3::zero(), v1);

        // Distributivity
        let scalar = 2.5;
        assert!((scalar * (v1 + v2)).approx_eq(&(scalar * v1 + scalar * v2), f64::EPSILON));
    }

    #[test]
    fn cross_product_properties() {
        let v1 = Vec3::new(1.0, 2.0, 3.0);
        let v2 = Vec3::new(4.0, 5.0, 6.0);

        // Anti-commutativity
        assert_eq!(v1.cross(&v2), Vec3::zero() - v2.cross(&v1));

        // Cross product with itself is zero
        assert_eq!(v1.cross(&v1), Vec3::zero());

        // Cross product is perpendicular to both vectors
        let cross = v1.cross(&v2);
        assert!((cross.dot(&v1)).abs() < f64::EPSILON);
        assert!((cross.dot(&v2)).abs() < f64::EPSILON);
    }

    #[test]
    fn normalization_properties() {
        let v = Vec3::new(3.0, 4.0, 5.0);
        let normalized = v.normalize();

        // Normalized vector has unit length
        assert!((normalized.norm() - 1.0).abs() < f64::EPSILON);

        // Direction is preserved
        assert!(v.dot(&normalized) > 0.0);

        // Normalizing a normalized vector gives the same result
        let double_normalized = normalized.normalize();
        assert!(normalized.approx_eq(&double_normalized, f64::EPSILON));
    }

    #[test]
    fn serialization_deserialization() {
        let v = Vec3::new(1.23, 4.56, 7.89);

        // Test that the vector can be serialized and deserialized
        let serialized = serde_json::to_string(&v).unwrap();
        let deserialized: Vec3 = serde_json::from_str(&serialized).unwrap();

        assert_eq!(v, deserialized);
    }

    #[test]
    fn debug_and_clone() {
        let v = Vec3::new(1.0, 2.0, 3.0);

        // Test Debug trait
        let debug_string = format!("{:?}", v);
        assert!(debug_string.contains("1.0"));
        assert!(debug_string.contains("2.0"));
        assert!(debug_string.contains("3.0"));

        // Test Clone trait
        let cloned = v.clone();
        assert_eq!(v, cloned);

        // Test Copy trait (implicit through assignment)
        let copied = v;
        assert_eq!(v, copied);
    }

    #[test]
    fn negation_properties() {
        let v = Vec3::new(5.0, -3.0, 1.5);

        // Double negation returns original
        assert_eq!(-(-v), v);

        // Negation preserves magnitude
        assert!((v.norm() - (-v).norm()).abs() < f64::EPSILON);

        // Negation reverses direction (dot product is negative of magnitude squared)
        assert!((v.dot(&(-v)) + v.norm_squared()).abs() < f64::EPSILON);

        // Negation is equivalent to multiplication by -1
        assert_eq!(-v, v * -1.0);
    }

    #[test]
    fn rotate_around_z_axis_90_degrees() {
        let v = Vec3::new(1.0, 0.0, 0.0);
        let axis = Vec3::z_hat();
        let angle = PI / 2.0;
        let rotated = v.rotate_around(&axis, angle).unwrap();
        let expected = Vec3::new(0.0, 1.0, 0.0);
        assert!(rotated.approx_eq(&expected, f64::EPSILON));
    }

    #[test]
    fn rotate_by_zero_angle() {
        let v = Vec3::new(1.0, 2.0, 3.0);
        let axis = Vec3::y_hat();
        let rotated = v.rotate_around(&axis, 0.0).unwrap();
        assert!(rotated.approx_eq(&v, f64::EPSILON));
    }

    #[test]
    fn rotate_by_very_small_angle() {
        let v = Vec3::new(1.0, 2.0, 3.0);
        let axis = Vec3::y_hat();
        let small_angle = f64::EPSILON / 2.0;
        let rotated = v.rotate_around(&axis, small_angle).unwrap();
        assert!(rotated.approx_eq(&v, f64::EPSILON));
    }

    #[test]
    fn rotate_by_full_circle() {
        let v = Vec3::new(1.0, 2.0, 3.0);
        let axis = Vec3::new(1.0, 1.0, 1.0).normalize();
        let rotated = v.rotate_around(&axis, 2.0 * PI).unwrap();
        assert!(rotated.approx_eq(&v, 1e-10));
    }

    #[test]
    fn rotate_by_multiple_full_circles() {
        let v = Vec3::new(1.0, 2.0, 3.0);
        let axis = Vec3::z_hat();
        let rotated = v.rotate_around(&axis, 6.0 * PI).unwrap(); // 3 full circles
        assert!(rotated.approx_eq(&v, 1e-10));
    }

    #[test]
    fn rotate_vector_parallel_to_axis() {
        let axis = Vec3::new(1.0, 1.0, 1.0).normalize();
        let v = axis * 5.0; // Vector parallel to the axis
        let rotated = v.rotate_around(&axis, PI / 4.0).unwrap();
        assert!(rotated.approx_eq(&v, f64::EPSILON));
    }

    #[test]
    fn rotate_vector_anti_parallel_to_axis() {
        let axis = Vec3::new(1.0, 1.0, 1.0).normalize();
        let v = axis * -3.0; // Vector anti-parallel to the axis
        let rotated = v.rotate_around(&axis, PI / 3.0).unwrap();
        assert!(rotated.approx_eq(&v, 1e-10));
    }

    #[test]
    fn rotate_preserves_magnitude() {
        let v = Vec3::new(3.0, 4.0, 5.0);
        let axis = Vec3::new(-1.0, 2.0, -3.0).normalize();
        let angle = 2.5; // Arbitrary angle
        let rotated = v.rotate_around(&axis, angle).unwrap();

        assert!((rotated.norm() - v.norm()).abs() < 1e-10);
    }

    #[test]
    fn rotate_negative_angle() {
        let v = Vec3::new(1.0, 0.0, 0.0);
        let axis = Vec3::z_hat();
        let rotated_pos = v.rotate_around(&axis, PI / 2.0).unwrap();
        let rotated_neg = v.rotate_around(&axis, -PI / 2.0).unwrap();

        // Rotating by -90° should be the opposite of +90°
        let expected = Vec3::new(0.0, -1.0, 0.0);
        assert!(rotated_neg.approx_eq(&expected, f64::EPSILON));

        // They should be negatives of each other in y-component
        assert!((rotated_pos.y + rotated_neg.y).abs() < f64::EPSILON);
    }

    #[test]
    fn rotate_around_zero_vector_returns_none() {
        let v = Vec3::new(1.0, 2.0, 3.0);
        let zero_axis = Vec3::zero();
        let result = v.rotate_around(&zero_axis, PI / 2.0);
        assert!(result.is_none());
    }

    #[test]
    fn rotate_around_very_small_axis_returns_none() {
        let v = Vec3::new(1.0, 2.0, 3.0);
        let tiny_axis = Vec3::new(1e-20, 1e-20, 1e-20);
        let result = v.rotate_around(&tiny_axis, PI / 2.0);
        assert!(result.is_none());
    }

    #[test]
    fn rotate_around_non_normalized_axis_returns_none() {
        let v = Vec3::new(1.0, 2.0, 3.0);
        let non_normalized = Vec3::new(2.0, 0.0, 0.0); // Length = 2, not 1
        let result = v.rotate_around(&non_normalized, PI / 2.0);
        assert!(result.is_none());
    }

    #[test]
    fn rotate_around_slightly_non_normalized_axis_returns_none() {
        let v = Vec3::new(1.0, 2.0, 3.0);
        let slightly_off = Vec3::new(1.001, 0.0, 0.0); // Just slightly longer than 1
        let result = v.rotate_around(&slightly_off, PI / 2.0);
        assert!(result.is_none());
    }

    #[test]
    fn rotate_around_axis_within_tolerance() {
        let v = Vec3::new(1.0, 2.0, 3.0);
        // Create an axis that's very close to normalized (within numerical tolerance)
        let almost_normalized = Vec3::new(1.0, 0.0, 0.0) * (1.0 + f64::EPSILON * 5.0);
        let result = v.rotate_around(&almost_normalized, PI / 2.0);
        assert!(result.is_some());
    }

    #[test]
    fn rodrigues_formula_consistency() {
        // Test that our implementation matches the mathematical formula
        let v = Vec3::new(1.0, 2.0, 3.0);
        let k = Vec3::new(0.0, 0.0, 1.0); // z-axis
        let theta = PI / 6.0; // 30 degrees

        let rotated = v.rotate_around(&k, theta).unwrap();

        // Manual calculation using Rodrigues' formula
        let cos_theta = theta.cos();
        let sin_theta = theta.sin();
        let cross_product = k.cross(&v);
        let dot_product = k.dot(&v);

        let expected =
            v * cos_theta + cross_product * sin_theta + k * dot_product * (1.0 - cos_theta);

        assert!(rotated.approx_eq(&expected, f64::EPSILON));
    }

    #[test]
    fn rotation_composition() {
        // Test that rotating by angle A then angle B equals rotating by angle A+B
        let v = Vec3::new(1.0, 2.0, 3.0);
        let axis = Vec3::new(1.0, 1.0, 1.0).normalize();
        let angle1 = PI / 6.0;
        let angle2 = PI / 4.0;

        let rotated_separate = v
            .rotate_around(&axis, angle1)
            .unwrap()
            .rotate_around(&axis, angle2)
            .unwrap();
        let rotated_combined = v.rotate_around(&axis, angle1 + angle2).unwrap();

        assert!(rotated_separate.approx_eq(&rotated_combined, 1e-10));
    }

    #[test]
    fn rotation_inverse() {
        // Test that rotating by angle then by -angle returns to original
        let v = Vec3::new(1.0, 2.0, 3.0);
        let axis = Vec3::new(-2.0, 3.0, 1.0).normalize();
        let angle = 1.5; // Arbitrary angle

        let rotated_and_back = v
            .rotate_around(&axis, angle)
            .unwrap()
            .rotate_around(&axis, -angle)
            .unwrap();

        assert!(rotated_and_back.approx_eq(&v, 1e-10));
    }

    #[test]
    fn zero_vector_rotation() {
        let zero = Vec3::zero();
        let axis = Vec3::x_hat();
        let rotated = zero.rotate_around(&axis, PI).unwrap();
        assert_eq!(rotated, Vec3::zero());
    }
}
