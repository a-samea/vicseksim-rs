//! # 3D Vector Mathematics Module
//!
//! This module provides a high-performance 3D vector implementation specifically
//! optimized for flocking simulations and particle systems. The `Vec3` struct
//! offers comprehensive vector operations with both value and reference semantics
//! to minimize allocations in performance-critical code.
//!
//! ## Features
//! - Zero-allocation vector operations
//! - Complete operator overloading for ergonomic usage
//! - Reference-based operations to avoid unnecessary moves
//! - Optimized normalize and cross product implementations
//! - Robust handling of edge cases (zero vectors, near-zero magnitudes)
//! - Serde serialization support for data persistence
//!
//! ## Example Usage
//! ```
//! use flocking_lib::vector::Vec3;
//!
//! // Create vectors
//! let position = Vec3::new(1.0, 2.0, 3.0);
//! let velocity = Vec3::new(0.5, -1.0, 0.0);
//!
//! // Vector operations
//! let new_position = position + velocity * 0.1;
//! let distance = (position - new_position).norm();
//!
//! // Physics calculations
//! let normalized_vel = velocity.normalize();
//! let cross_product = position.cross(&velocity);
//! ```

use std::ops::{Add, Div, Mul, Neg, Sub};

/// A 3D vector in Cartesian coordinates optimized for flocking simulations.
///
/// `Vec3` represents a point or direction in 3D space using double-precision
/// floating-point components. It implements `Copy` for efficient passing and
/// provides comprehensive mathematical operations essential for particle physics
/// and flocking behavior simulations.
///
/// The struct is designed for high-performance scenarios where vectors are
/// frequently created, manipulated, and passed between functions. All basic
/// operations are inlined and optimized to minimize computational overhead.
///
/// # Fields
/// - `x`: The X-component (horizontal axis)
/// - `y`: The Y-component (vertical axis)
/// - `z`: The Z-component (depth axis)
///
/// # Examples
/// ```
/// # use flocking_lib::vector::Vec3;
/// // Create a vector representing a position in 3D space
/// let position = Vec3::new(10.0, 5.0, -2.0);
///
/// // Access components directly
/// assert_eq!(position.x, 10.0);
/// assert_eq!(position.y, 5.0);
/// assert_eq!(position.z, -2.0);
///
/// // Create unit vectors for coordinate axes
/// let right = Vec3::x_hat();
/// let up = Vec3::y_hat();
/// let forward = Vec3::z_hat();
/// ```
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Vec3 {
    /// X-component of the vector
    pub x: f64,
    /// Y-component of the vector
    pub y: f64,
    /// Z-component of the vector
    pub z: f64,
}

impl Vec3 {
    /// Creates a new 3D vector with the given components.
    ///
    /// This is the primary constructor for `Vec3`. All components are stored
    /// as `f64` for maximum precision in mathematical operations.
    ///
    /// # Arguments
    /// * `x` - The X-component
    /// * `y` - The Y-component
    /// * `z` - The Z-component
    ///
    /// # Examples
    /// ```
    /// # use flocking_lib::vector::Vec3;
    /// let velocity = Vec3::new(1.5, -2.3, 0.0);
    /// let position = Vec3::new(-10.0, 15.0, 5.5);
    ///
    /// // Vectors can represent any 3D quantity
    /// let force = Vec3::new(0.0, -9.81, 0.0); // Gravity
    /// ```
    #[inline]
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Vec3 { x, y, z }
    }

    /// Creates a zero vector (0, 0, 0).
    ///
    /// The zero vector is the additive identity and represents no displacement,
    /// velocity, or force. It's commonly used as a default value or starting
    /// point for accumulative operations.
    ///
    /// # Examples
    /// ```
    /// # use flocking_lib::vector::Vec3;
    /// let origin = Vec3::zero();
    /// assert_eq!(origin.x, 0.0);
    /// assert_eq!(origin.y, 0.0);
    /// assert_eq!(origin.z, 0.0);
    ///
    /// // Zero vector properties
    /// let v = Vec3::new(1.0, 2.0, 3.0);
    /// assert_eq!(v + Vec3::zero(), v);
    /// assert_eq!(v * 0.0, Vec3::zero());
    /// ```
    #[inline]
    pub fn zero() -> Self {
        Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }

    /// Calculates the squared magnitude (length) of the vector.
    ///
    /// This is more efficient than `norm()` as it avoids the square root operation.
    /// Use this when you only need to compare magnitudes or when the actual
    /// magnitude value isn't required.
    ///
    /// The squared norm is calculated as: x² + y² + z²
    ///
    /// # Examples
    /// ```
    /// # use flocking_lib::vector::Vec3;
    /// let v = Vec3::new(3.0, 4.0, 0.0);
    /// assert_eq!(v.norm_squared(), 25.0); // 3² + 4² + 0² = 9 + 16 + 0 = 25
    ///
    /// // Useful for distance comparisons without sqrt
    /// let distance_sq = v.norm_squared();
    /// if distance_sq < 100.0 { // Instead of norm() < 10.0
    ///     println!("Vector is close to origin");
    /// }
    /// ```
    #[inline]
    pub fn norm_squared(&self) -> f64 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    /// Calculates the magnitude (length) of the vector.
    ///
    /// Returns the Euclidean norm: √(x² + y² + z²)
    ///
    /// For performance-critical code where only magnitude comparison is needed,
    /// consider using `norm_squared()` instead to avoid the square root operation.
    ///
    /// # Examples
    /// ```
    /// # use flocking_lib::vector::Vec3;
    /// let v = Vec3::new(3.0, 4.0, 0.0);
    /// assert_eq!(v.norm(), 5.0); // √(3² + 4²) = √25 = 5
    ///
    /// // Unit vectors have magnitude 1
    /// let unit = Vec3::x_hat();
    /// assert!((unit.norm() - 1.0).abs() < f64::EPSILON);
    /// ```
    #[inline]
    pub fn norm(&self) -> f64 {
        self.norm_squared().sqrt()
    }

    /// Returns a unit vector in the same direction as this vector.
    ///
    /// If the vector has zero or near-zero magnitude (within floating-point epsilon),
    /// returns the zero vector to avoid division by zero and numerical instability.
    ///
    /// The normalization process preserves direction while setting magnitude to 1.
    /// This is essential for direction vectors in physics calculations.
    ///
    /// # Examples
    /// ```
    /// # use flocking_lib::vector::Vec3;
    /// let v = Vec3::new(3.0, 4.0, 0.0);
    /// let normalized = v.normalize();
    ///
    /// // Magnitude is now 1 (within floating-point precision)
    /// assert!((normalized.norm() - 1.0).abs() < 1e-10);
    ///
    /// // Zero vector normalization
    /// assert_eq!(Vec3::zero().normalize(), Vec3::zero());
    /// ```
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

    /// Calculates the dot product (scalar product) with another vector.
    ///
    /// The dot product measures how much two vectors point in the same direction.
    /// It returns a scalar value calculated as: a·b = ax*bx + ay*by + az*bz
    ///
    /// # Properties
    /// - Positive when vectors point in similar directions
    /// - Zero when vectors are perpendicular
    /// - Negative when vectors point in opposite directions
    /// - Equals |a||b|cos(θ) where θ is the angle between vectors
    ///
    /// # Arguments
    /// * `other` - The vector to compute the dot product with
    ///
    /// # Examples
    /// ```
    /// # use flocking_lib::vector::Vec3;
    /// let a = Vec3::new(1.0, 2.0, 3.0);
    /// let b = Vec3::new(4.0, 5.0, 6.0);
    /// assert_eq!(a.dot(&b), 32.0); // 1*4 + 2*5 + 3*6 = 32
    ///
    /// // Perpendicular vectors have dot product of 0
    /// let x_axis = Vec3::x_hat();
    /// let y_axis = Vec3::y_hat();
    /// assert_eq!(x_axis.dot(&y_axis), 0.0);
    ///
    /// // Parallel vectors
    /// let parallel = Vec3::new(2.0, 4.0, 6.0); // 2 * a
    /// assert!(a.dot(&parallel) > 0.0); // Positive dot product
    /// ```
    #[inline]
    pub fn dot(&self, other: &Self) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    /// Calculates the cross product with another vector.
    ///
    /// The cross product produces a vector perpendicular to both input vectors.
    /// The magnitude equals the area of the parallelogram formed by the vectors.
    /// Direction follows the right-hand rule.
    ///
    /// # Properties
    /// - Anti-commutative: a × b = -(b × a)
    /// - Result is perpendicular to both input vectors
    /// - Magnitude = |a||b|sin(θ) where θ is the angle between vectors
    /// - Zero when vectors are parallel or anti-parallel
    ///
    /// # Arguments
    /// * `other` - The vector to compute the cross product with
    ///
    /// # Examples
    /// ```
    /// # use flocking_lib::vector::Vec3;
    /// // Standard basis vectors follow right-hand rule
    /// let x = Vec3::x_hat();
    /// let y = Vec3::y_hat();
    /// let z = Vec3::z_hat();
    ///
    /// assert_eq!(x.cross(&y), z);
    /// assert_eq!(y.cross(&z), x);
    /// assert_eq!(z.cross(&x), y);
    ///
    /// // Anti-commutative property
    /// assert_eq!(y.cross(&x), -z);
    ///
    /// // Parallel vectors yield zero
    /// let parallel = Vec3::new(2.0, 4.0, 6.0);
    /// let base = Vec3::new(1.0, 2.0, 3.0);
    /// assert_eq!(base.cross(&parallel), Vec3::zero());
    /// ```
    #[inline]
    pub fn cross(&self, other: &Self) -> Self {
        Vec3 {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    /// Calculates the angle between two vectors in radians.
    ///
    /// Uses the dot product formula: θ = arccos((a·b)/(|a||b|))
    /// Returns 0 for zero vectors to avoid numerical issues.
    /// The result is always in the range [0, π].
    ///
    /// # Arguments
    /// * `other` - The vector to measure the angle to
    ///
    /// # Returns
    /// The angle in radians between the two vectors
    ///
    /// # Examples
    /// ```
    /// # use flocking_lib::vector::Vec3;
    /// use std::f64::consts::PI;
    ///
    /// let x = Vec3::x_hat();
    /// let y = Vec3::y_hat();
    ///
    /// // 90 degrees between perpendicular vectors
    /// assert!((x.angle_between(&y) - PI/2.0).abs() < 1e-10);
    ///
    /// // 0 degrees for same direction
    /// assert!(x.angle_between(&x).abs() < 1e-10);
    ///
    /// // 180 degrees for opposite directions
    /// let neg_x = Vec3::new(-1.0, 0.0, 0.0);
    /// assert!((x.angle_between(&neg_x) - PI).abs() < 1e-10);
    /// ```
    pub fn angle_between(&self, other: &Self) -> f64 {
        let dot_product = self.dot(other);
        let norm_product_sq = self.norm_squared() * other.norm_squared();
        if norm_product_sq > f64::EPSILON * f64::EPSILON {
            (dot_product / norm_product_sq.sqrt()).acos()
        } else {
            0.0
        }
    }

    /// Projects this vector onto another vector.
    ///
    /// Vector projection finds the component of this vector that lies along
    /// the direction of the target vector. The result is a vector parallel
    /// to the target with magnitude equal to the scalar projection.
    ///
    /// Formula: proj_b(a) = ((a·b)/(b·b)) * b
    ///
    /// # Arguments
    /// * `other` - The vector to project onto
    ///
    /// # Returns
    /// The projection of this vector onto the target vector
    ///
    /// # Examples
    /// ```
    /// # use flocking_lib::vector::Vec3;
    /// let v = Vec3::new(3.0, 4.0, 0.0);
    /// let x_axis = Vec3::x_hat();
    ///
    /// // Project onto X-axis extracts X-component
    /// let projection = v.project_onto(&x_axis);
    /// assert_eq!(projection, Vec3::new(3.0, 0.0, 0.0));
    ///
    /// // Projection onto zero vector returns zero
    /// assert_eq!(v.project_onto(&Vec3::zero()), Vec3::zero());
    /// ```
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

    /// Checks if this vector is approximately equal to another within epsilon tolerance.
    ///
    /// Due to floating-point precision limitations, exact equality is rarely
    /// appropriate for vector comparisons. This method compares each component
    /// individually within the specified tolerance.
    ///
    /// # Arguments
    /// * `other` - The vector to compare with
    /// * `epsilon` - The maximum allowed difference per component
    ///
    /// # Examples
    /// ```
    /// # use flocking_lib::vector::Vec3;
    /// let v1 = Vec3::new(1.0, 2.0, 3.0);
    /// let v2 = Vec3::new(1.0000001, 2.0000001, 3.0000001);
    ///
    /// assert!(v1.approx_eq(&v2, 1e-6));
    /// assert!(!v1.approx_eq(&v2, 1e-8));
    /// ```
    #[inline]
    pub fn approx_eq(&self, other: &Self, epsilon: f64) -> bool {
        (self.x - other.x).abs() < epsilon
            && (self.y - other.y).abs() < epsilon
            && (self.z - other.z).abs() < epsilon
    }

    /// Returns the unit vector along the positive X-axis (1, 0, 0).
    ///
    /// This represents the standard "right" or "east" direction in most
    /// coordinate systems. Commonly used as a reference direction.
    ///
    /// # Examples
    /// ```
    /// # use flocking_lib::vector::Vec3;
    /// let right = Vec3::x_hat();
    /// assert_eq!(right, Vec3::new(1.0, 0.0, 0.0));
    /// assert!((right.norm() - 1.0).abs() < f64::EPSILON);
    /// ```
    #[inline]
    pub fn x_hat() -> Self {
        Vec3::new(1.0, 0.0, 0.0)
    }

    /// Returns the unit vector along the positive Y-axis (0, 1, 0).
    ///
    /// This represents the standard "up" or "north" direction in most
    /// coordinate systems. Essential for defining vertical orientation.
    ///
    /// # Examples
    /// ```
    /// # use flocking_lib::vector::Vec3;
    /// let up = Vec3::y_hat();
    /// assert_eq!(up, Vec3::new(0.0, 1.0, 0.0));
    /// assert!((up.norm() - 1.0).abs() < f64::EPSILON);
    /// ```
    #[inline]
    pub fn y_hat() -> Self {
        Vec3::new(0.0, 1.0, 0.0)
    }

    /// Returns the unit vector along the positive Z-axis (0, 0, 1).
    ///
    /// This represents the standard "forward" direction in most coordinate
    /// systems. Used to define depth or the viewing direction.
    ///
    /// # Examples
    /// ```
    /// # use flocking_lib::vector::Vec3;
    /// let forward = Vec3::z_hat();
    /// assert_eq!(forward, Vec3::new(0.0, 0.0, 1.0));
    /// assert!((forward.norm() - 1.0).abs() < f64::EPSILON);
    /// ```
    #[inline]
    pub fn z_hat() -> Self {
        Vec3::new(0.0, 0.0, 1.0)
    }
}

// --- Operator Overloads for Ergonomic Vector Mathematics ---

/// Vector addition by value (Vec3 + Vec3).
///
/// Adds corresponding components of two vectors. This consumes both input
/// vectors and returns a new vector containing the sum.
///
/// # Examples
/// ```
/// # use flocking_lib::vector::Vec3;
/// let a = Vec3::new(1.0, 2.0, 3.0);
/// let b = Vec3::new(4.0, 5.0, 6.0);
/// let sum = a + b;
/// assert_eq!(sum, Vec3::new(5.0, 7.0, 9.0));
/// ```
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

/// Vector addition by reference (&Vec3 + &Vec3).
///
/// More efficient for cases where you want to preserve the original vectors.
/// This is the most common pattern in simulation loops where vectors are
/// reused across multiple operations.
///
/// # Examples
/// ```
/// # use flocking_lib::vector::Vec3;
/// let position = Vec3::new(10.0, 5.0, 0.0);
/// let velocity = Vec3::new(1.0, -0.5, 0.0);
/// let new_position = &position + &velocity; // Originals preserved
/// ```
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

/// Vector subtraction by value (Vec3 - Vec3).
///
/// Subtracts corresponding components. The result represents the displacement
/// vector from the second vector to the first.
///
/// # Examples
/// ```
/// # use flocking_lib::vector::Vec3;
/// let a = Vec3::new(5.0, 7.0, 9.0);
/// let b = Vec3::new(1.0, 2.0, 3.0);
/// let difference = a - b;
/// assert_eq!(difference, Vec3::new(4.0, 5.0, 6.0));
/// ```
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

/// Vector subtraction by reference (&Vec3 - &Vec3).
///
/// Efficient subtraction that preserves original vectors. Commonly used
/// for calculating displacement vectors between positions.
///
/// # Examples
/// ```
/// # use flocking_lib::vector::Vec3;
/// let target = Vec3::new(10.0, 5.0, 0.0);
/// let current = Vec3::new(8.0, 3.0, 0.0);
/// let direction = &target - &current; // Points from current to target
/// ```
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

/// Scalar multiplication by value (Vec3 * f64).
///
/// Scales all vector components by the same factor. Commonly used for
/// applying time steps, scaling forces, or changing magnitudes.
///
/// # Examples
/// ```
/// # use flocking_lib::vector::Vec3;
/// let velocity = Vec3::new(2.0, -1.0, 0.5);
/// let dt = 0.1;
/// let displacement = velocity * dt;
/// assert_eq!(displacement, Vec3::new(0.2, -0.1, 0.05));
/// ```
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

/// Scalar multiplication by reference (&Vec3 * f64).
///
/// Efficient scaling that preserves the original vector. Useful when the
/// same vector needs to be scaled multiple times or used elsewhere.
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

/// Commutative scalar multiplication by value (f64 * Vec3).
///
/// Allows writing scalar multiplication in natural mathematical notation.
/// Equivalent to Vec3 * f64 but reads more naturally in many contexts.
///
/// # Examples
/// ```
/// # use flocking_lib::vector::Vec3;
/// let force = Vec3::new(0.0, -9.81, 0.0);
/// let mass = 2.5;
/// let acceleration = mass * force; // Natural physics notation
/// ```
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

/// Commutative scalar multiplication by reference (f64 * &Vec3).
///
/// Efficient commutative multiplication that preserves the original vector.
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

/// Scalar division by value (Vec3 / f64).
///
/// Divides all components by the scalar. More efficient than multiplication
/// by reciprocal due to optimized implementation using `recip()`.
///
/// # Examples
/// ```
/// # use flocking_lib::vector::Vec3;
/// let position = Vec3::new(10.0, 20.0, 30.0);
/// let center = position / 2.0;
/// assert_eq!(center, Vec3::new(5.0, 10.0, 15.0));
/// ```
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

/// Scalar division by reference (&Vec3 / f64).
///
/// Efficient division that preserves the original vector. The implementation
/// uses multiplication by reciprocal for better performance.
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

/// Vector negation by value (-Vec3).
///
/// Returns a vector pointing in the opposite direction with the same magnitude.
/// This is equivalent to multiplying by -1 but more expressive and efficient.
/// Commonly used for reversing forces, velocities, or directions.
///
/// # Examples
/// ```
/// # use flocking_lib::vector::Vec3;
/// let velocity = Vec3::new(2.0, -1.0, 3.0);
/// let opposite = -velocity;
/// assert_eq!(opposite, Vec3::new(-2.0, 1.0, -3.0));
///
/// // Useful for physics calculations
/// let force = Vec3::new(10.0, 0.0, 0.0);
/// let reaction_force = -force; // Newton's 3rd law
/// ```
impl Neg for Vec3 {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self::Output {
        Vec3 {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

/// Vector negation by reference (-&Vec3).
///
/// Efficient negation that preserves the original vector. Useful when you
/// need both the original and negated vectors or in performance-critical loops.
impl Neg for &Vec3 {
    type Output = Vec3;
    #[inline]
    fn neg(self) -> Self::Output {
        Vec3 {
            x: -self.x,
            y: -self.y,
            z: -self.z,
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

    #[test]
    fn test_negation_value() {
        let v = Vec3::new(1.0, -2.0, 3.0);
        let negated = -v;

        assert_eq!(negated, Vec3::new(-1.0, 2.0, -3.0));

        // Test with zero vector
        let zero = Vec3::zero();
        assert_eq!(-zero, Vec3::zero());
    }

    #[test]
    fn test_negation_reference() {
        let v = Vec3::new(1.0, -2.0, 3.0);
        let negated = -&v;

        assert_eq!(negated, Vec3::new(-1.0, 2.0, -3.0));
        // Ensure original vector is unchanged
        assert_eq!(v, Vec3::new(1.0, -2.0, 3.0));
    }

    #[test]
    fn test_negation_properties() {
        let v = Vec3::new(5.0, -3.0, 1.5);

        // Double negation returns original
        assert_eq!(-(-v), v);

        // Negation preserves magnitude
        assert!((v.norm() - (-v).norm()).abs() < EPSILON);

        // Negation reverses direction (dot product is negative of magnitude squared)
        assert!((v.dot(&(-v)) + v.norm_squared()).abs() < EPSILON);

        // Negation is equivalent to multiplication by -1
        assert_eq!(-v, v * -1.0);
    }
}
