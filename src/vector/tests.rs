#[cfg(test)]
mod vector_tests {
    use super::super::Vec3;
    use std::f64::consts::PI;

    #[test]
    fn test_normalize_very_small_vector() {
        let v = Vec3::new(1e-20, 1e-20, 1e-20);
        let normalized = v.normalize();
        assert_eq!(normalized, Vec3::zero());
    }

    #[test]
    fn test_cross_product() {
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
    fn test_angle_between() {
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
    fn test_angle_between_zero_vectors() {
        let zero = Vec3::zero();
        let v = Vec3::new(1.0, 0.0, 0.0);

        assert_eq!(zero.angle_between(&v), 0.0);
        assert_eq!(v.angle_between(&zero), 0.0);
        assert_eq!(zero.angle_between(&zero), 0.0);
    }

    #[test]
    fn test_project_onto() {
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
    fn test_project_onto_zero_vector() {
        let v = Vec3::new(1.0, 2.0, 3.0);
        let zero = Vec3::zero();

        let projection = v.project_onto(&zero);
        assert_eq!(projection, Vec3::zero());
    }

    #[test]
    fn test_approx_eq() {
        let v1 = Vec3::new(1.0, 2.0, 3.0);
        let v2 = Vec3::new(1.0000001, 2.0000001, 3.0000001);
        let v3 = Vec3::new(1.1, 2.1, 3.1);

        assert!(v1.approx_eq(&v2, 1e-6));
        assert!(!v1.approx_eq(&v2, 1e-8));
        assert!(!v1.approx_eq(&v3, 1e-6));
        assert!(v1.approx_eq(&v3, 0.2));
    }

    #[test]
    fn test_zero_scalar_multiplication() {
        let v = Vec3::new(1.0, 2.0, 3.0);
        let result = v * 0.0;

        assert_eq!(result, Vec3::zero());
    }

    #[test]
    fn test_negative_scalar_multiplication() {
        let v = Vec3::new(1.0, 2.0, 3.0);
        let result = v * -1.0;

        assert_eq!(result, Vec3::new(-1.0, -2.0, -3.0));
    }

    #[test]
    fn test_vector_properties() {
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
    fn test_cross_product_properties() {
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
    fn test_normalization_properties() {
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
    fn test_serialization_deserialization() {
        let v = Vec3::new(1.23, 4.56, 7.89);

        // Test that the vector can be serialized and deserialized
        let serialized = serde_json::to_string(&v).unwrap();
        let deserialized: Vec3 = serde_json::from_str(&serialized).unwrap();

        assert_eq!(v, deserialized);
    }

    #[test]
    fn test_debug_and_clone() {
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
    fn test_negation_properties() {
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
}
