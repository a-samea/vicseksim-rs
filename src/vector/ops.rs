//! # Vector operations for Vec3
//! This module provides implementations of various mathematical operations
//! for the `Vec3` struct, including addition, subtraction, scalar multiplication,
//! scalar division, and negation. These operations are implemented using Rust's
//! operator overloading traits (`Add`, `Sub`, `Mul`, `Div`, `Neg`).

use super::Vec3;
use std::ops::{Add, Div, Mul, Neg, Sub};

/// Vector addition
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
