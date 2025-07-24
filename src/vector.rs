use std::ops::{Add, Div, Mul, Sub};

/// A 3D vector in Cartesian coordinates. The workhorse of our simulation.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

// --- Implementation of Vector Math ---
impl Vec3 {
    #[inline]
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Vec3 { x, y, z }
    }

    #[inline]
    pub fn zero() -> Self {
        Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }

    #[inline]
    pub fn norm_squared(&self) -> f64 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    #[inline]
    pub fn norm(&self) -> f64 {
        self.norm_squared().sqrt()
    }

    pub fn normalize(&self) -> Self {
        let norm_sq = self.norm_squared();
        if norm_sq > f64::EPSILON * f64::EPSILON {
            let inv_norm = norm_sq.sqrt().recip();
            Vec3 {
                x: self.x * inv_norm,
                y: self.y * inv_norm,
                z: self.z * inv_norm,
            }
        } else {
            Vec3::zero()
        }
    }

    #[inline]
    pub fn dot(&self, other: &Self) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    #[inline]
    pub fn cross(&self, other: &Self) -> Self {
        Vec3 {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    /// Calculate the angle between two vectors in radians
    pub fn angle_between(&self, other: &Self) -> f64 {
        let dot_product = self.dot(other);
        let norm_product_sq = self.norm_squared() * other.norm_squared();
        if norm_product_sq > f64::EPSILON * f64::EPSILON {
            (dot_product / norm_product_sq.sqrt()).acos()
        } else {
            0.0
        }
    }

    /// Project this vector onto another vector
    pub fn project_onto(&self, other: &Self) -> Self {
        let norm_sq = other.norm_squared();
        if norm_sq > f64::EPSILON * f64::EPSILON {
            let scalar_projection = self.dot(other) / norm_sq;
            Vec3 {
                x: other.x * scalar_projection,
                y: other.y * scalar_projection,
                z: other.z * scalar_projection,
            }
        } else {
            Vec3::zero()
        }
    }

    /// Check if this vector is approximately equal to another within epsilon
    #[inline]
    pub fn approx_eq(&self, other: &Self, epsilon: f64) -> bool {
        (self.x - other.x).abs() < epsilon
            && (self.y - other.y).abs() < epsilon
            && (self.z - other.z).abs() < epsilon
    }

    /// Common unit vectors
    #[inline]
    pub fn x_hat() -> Self {
        Vec3::new(1.0, 0.0, 0.0)
    }

    #[inline]
    pub fn y_hat() -> Self {
        Vec3::new(0.0, 1.0, 0.0)
    }

    #[inline]
    pub fn z_hat() -> Self {
        Vec3::new(0.0, 0.0, 1.0)
    }
}

// --- Operator Overloads for Ergonomics ---
// Value + Value
impl Add for Vec3 {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Vec3 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

// Reference + Reference (most common case in simulations)
impl Add for &Vec3 {
    type Output = Vec3;
    #[inline]
    fn add(self, rhs: &Vec3) -> Self::Output {
        Vec3 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

// Value - Value
impl Sub for Vec3 {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Vec3 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

// Reference - Reference
impl Sub for &Vec3 {
    type Output = Vec3;
    #[inline]
    fn sub(self, rhs: &Vec3) -> Self::Output {
        Vec3 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

// Value * scalar
impl Mul<f64> for Vec3 {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: f64) -> Self::Output {
        Vec3 {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

// Reference * scalar
impl Mul<f64> for &Vec3 {
    type Output = Vec3;
    #[inline]
    fn mul(self, rhs: f64) -> Self::Output {
        Vec3 {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

// scalar * Value
impl Mul<Vec3> for f64 {
    type Output = Vec3;
    #[inline]
    fn mul(self, rhs: Vec3) -> Self::Output {
        Vec3 {
            x: self * rhs.x,
            y: self * rhs.y,
            z: self * rhs.z,
        }
    }
}

// scalar * Reference
impl Mul<&Vec3> for f64 {
    type Output = Vec3;
    #[inline]
    fn mul(self, rhs: &Vec3) -> Self::Output {
        Vec3 {
            x: self * rhs.x,
            y: self * rhs.y,
            z: self * rhs.z,
        }
    }
}

// Value / scalar
impl Div<f64> for Vec3 {
    type Output = Self;
    #[inline]
    fn div(self, rhs: f64) -> Self::Output {
        let inv = rhs.recip();
        Vec3 {
            x: self.x * inv,
            y: self.y * inv,
            z: self.z * inv,
        }
    }
}

// Reference / scalar
impl Div<f64> for &Vec3 {
    type Output = Vec3;
    #[inline]
    fn div(self, rhs: f64) -> Self::Output {
        let inv = rhs.recip();
        Vec3 {
            x: self.x * inv,
            y: self.y * inv,
            z: self.z * inv,
        }
    }
}

#[cfg(test)]
mod vector_tests {
    use super::Vec3;
    use std::f64::consts::PI;

    const EPSILON: f64 = 1e-10;

    #[test]
    fn test_new() {
        let v = Vec3::new(1.0, 2.0, 3.0);
        assert_eq!(v.x, 1.0);
        assert_eq!(v.y, 2.0);
        assert_eq!(v.z, 3.0);
    }

    #[test]
    fn test_zero() {
        let v = Vec3::zero();
        assert_eq!(v.x, 0.0);
        assert_eq!(v.y, 0.0);
        assert_eq!(v.z, 0.0);
    }

    #[test]
    fn test_norm_squared() {
        let v = Vec3::new(3.0, 4.0, 0.0);
        assert_eq!(v.norm_squared(), 25.0);

        let v2 = Vec3::new(1.0, 2.0, 3.0);
        assert_eq!(v2.norm_squared(), 14.0);
    }

    #[test]
    fn test_norm() {
        let v = Vec3::new(3.0, 4.0, 0.0);
        assert_eq!(v.norm(), 5.0);

        let v2 = Vec3::new(1.0, 1.0, 1.0);
        assert!((v2.norm() - 3.0_f64.sqrt()).abs() < EPSILON);
    }

    #[test]
    fn test_normalize() {
        let v = Vec3::new(3.0, 4.0, 0.0);
        let normalized = v.normalize();
        assert!((normalized.norm() - 1.0).abs() < EPSILON);
        assert!((normalized.x - 0.6).abs() < EPSILON);
        assert!((normalized.y - 0.8).abs() < EPSILON);
        assert_eq!(normalized.z, 0.0);
    }

    #[test]
    fn test_normalize_zero_vector() {
        let v = Vec3::zero();
        let normalized = v.normalize();
        assert_eq!(normalized, Vec3::zero());
    }

    #[test]
    fn test_normalize_very_small_vector() {
        let v = Vec3::new(1e-20, 1e-20, 1e-20);
        let normalized = v.normalize();
        assert_eq!(normalized, Vec3::zero());
    }

    #[test]
    fn test_dot_product() {
        let v1 = Vec3::new(1.0, 2.0, 3.0);
        let v2 = Vec3::new(4.0, 5.0, 6.0);
        assert_eq!(v1.dot(&v2), 32.0); // 1*4 + 2*5 + 3*6 = 4 + 10 + 18 = 32

        // Test orthogonal vectors
        let v3 = Vec3::new(1.0, 0.0, 0.0);
        let v4 = Vec3::new(0.0, 1.0, 0.0);
        assert_eq!(v3.dot(&v4), 0.0);
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
        assert!((x.angle_between(&y) - PI / 2.0).abs() < EPSILON);

        // 0 degrees (same direction)
        assert!(x.angle_between(&x).abs() < EPSILON);

        // 180 degrees (opposite direction)
        let neg_x = Vec3::new(-1.0, 0.0, 0.0);
        assert!((x.angle_between(&neg_x) - PI).abs() < EPSILON);

        // 45 degrees
        let diagonal = Vec3::new(1.0, 1.0, 0.0);
        assert!((x.angle_between(&diagonal) - PI / 4.0).abs() < EPSILON);
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
        assert!((proj_diag.x - 3.5).abs() < EPSILON);
        assert!((proj_diag.y - 3.5).abs() < EPSILON);
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
    fn test_unit_vectors() {
        let x = Vec3::x_hat();
        let y = Vec3::y_hat();
        let z = Vec3::z_hat();

        assert_eq!(x, Vec3::new(1.0, 0.0, 0.0));
        assert_eq!(y, Vec3::new(0.0, 1.0, 0.0));
        assert_eq!(z, Vec3::new(0.0, 0.0, 1.0));

        assert!((x.norm() - 1.0).abs() < EPSILON);
        assert!((y.norm() - 1.0).abs() < EPSILON);
        assert!((z.norm() - 1.0).abs() < EPSILON);
    }

    #[test]
    fn test_addition_value() {
        let v1 = Vec3::new(1.0, 2.0, 3.0);
        let v2 = Vec3::new(4.0, 5.0, 6.0);
        let result = v1 + v2;

        assert_eq!(result, Vec3::new(5.0, 7.0, 9.0));
    }

    #[test]
    fn test_addition_reference() {
        let v1 = Vec3::new(1.0, 2.0, 3.0);
        let v2 = Vec3::new(4.0, 5.0, 6.0);
        let result = &v1 + &v2;

        assert_eq!(result, Vec3::new(5.0, 7.0, 9.0));
        // Ensure original vectors are unchanged
        assert_eq!(v1, Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(v2, Vec3::new(4.0, 5.0, 6.0));
    }

    #[test]
    fn test_subtraction_value() {
        let v1 = Vec3::new(5.0, 7.0, 9.0);
        let v2 = Vec3::new(1.0, 2.0, 3.0);
        let result = v1 - v2;

        assert_eq!(result, Vec3::new(4.0, 5.0, 6.0));
    }

    #[test]
    fn test_subtraction_reference() {
        let v1 = Vec3::new(5.0, 7.0, 9.0);
        let v2 = Vec3::new(1.0, 2.0, 3.0);
        let result = &v1 - &v2;

        assert_eq!(result, Vec3::new(4.0, 5.0, 6.0));
        // Ensure original vectors are unchanged
        assert_eq!(v1, Vec3::new(5.0, 7.0, 9.0));
        assert_eq!(v2, Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn test_scalar_multiplication_value() {
        let v = Vec3::new(1.0, 2.0, 3.0);
        let result = v * 2.5;

        assert_eq!(result, Vec3::new(2.5, 5.0, 7.5));
    }

    #[test]
    fn test_scalar_multiplication_reference() {
        let v = Vec3::new(1.0, 2.0, 3.0);
        let result = &v * 2.5;

        assert_eq!(result, Vec3::new(2.5, 5.0, 7.5));
        // Ensure original vector is unchanged
        assert_eq!(v, Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn test_scalar_multiplication_commutative() {
        let v = Vec3::new(1.0, 2.0, 3.0);
        let result1 = v * 2.5;
        let result2 = 2.5 * v;
        let result3 = 2.5 * &v;

        assert_eq!(result1, result2);
        assert_eq!(result1, result3);
    }

    #[test]
    fn test_scalar_division_value() {
        let v = Vec3::new(2.0, 4.0, 6.0);
        let result = v / 2.0;

        assert_eq!(result, Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn test_scalar_division_reference() {
        let v = Vec3::new(2.0, 4.0, 6.0);
        let result = &v / 2.0;

        assert_eq!(result, Vec3::new(1.0, 2.0, 3.0));
        // Ensure original vector is unchanged
        assert_eq!(v, Vec3::new(2.0, 4.0, 6.0));
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
        assert!((scalar * (v1 + v2)).approx_eq(&(scalar * v1 + scalar * v2), EPSILON));
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
        assert!((cross.dot(&v1)).abs() < EPSILON);
        assert!((cross.dot(&v2)).abs() < EPSILON);
    }

    #[test]
    fn test_normalization_properties() {
        let v = Vec3::new(3.0, 4.0, 5.0);
        let normalized = v.normalize();

        // Normalized vector has unit length
        assert!((normalized.norm() - 1.0).abs() < EPSILON);

        // Direction is preserved
        assert!(v.dot(&normalized) > 0.0);

        // Normalizing a normalized vector gives the same result
        let double_normalized = normalized.normalize();
        assert!(normalized.approx_eq(&double_normalized, EPSILON));
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
}
