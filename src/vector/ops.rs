//! # Vector operations for Vec3
//! This module provides implementations of various mathematical operations
//! for the `Vec3` struct, including addition, subtraction, scalar multiplication,
//! scalar division, and negation. These operations are implemented using Rust's
//! operator overloading traits (`Add`, `Sub`, `Mul`, `Div`, `Neg`).
//!
//! Since Vec3 implements Copy, only value-based operations are provided for
//! simplicity and clarity. The Copy trait ensures these operations are efficient.

use super::Vec3;
use std::ops::{Add, Div, Mul, Neg, Sub};

/// Vector addition (Vec3 + Vec3).
///
/// Adds corresponding components of two vectors. Since Vec3 implements Copy,
/// this operation is efficient whether used with owned values or references.
/// When using references, they will be automatically dereferenced due to Copy.
///
/// # Examples
/// ```
/// # use flocking_lib::vector::Vec3;
/// let a = Vec3::new(1.0, 2.0, 3.0);
/// let b = Vec3::new(4.0, 5.0, 6.0);
/// let sum = a + b;
/// assert_eq!(sum, Vec3::new(5.0, 7.0, 9.0));
/// 
/// // With references, use explicit dereferencing or let Copy handle it
/// let sum_ref = *&a + *&b;  // or simply: a + b
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

/// Vector subtraction (Vec3 - Vec3).
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

/// Scalar multiplication (Vec3 * f64).
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

/// Commutative scalar multiplication (f64 * Vec3).
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

/// Scalar division (Vec3 / f64).
///
/// Divides all components by the scalar. 
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

/// Vector negation (-Vec3).
///
/// Negates all components, effectively reversing the vector's direction.
/// Useful for representing opposite forces or directions.
///
/// # Examples
/// ```
/// # use flocking_lib::vector::Vec3;
/// let velocity = Vec3::new(1.0, -2.0, 3.0);
/// let opposite = -velocity;
/// assert_eq!(opposite, Vec3::new(-1.0, 2.0, -3.0));
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
