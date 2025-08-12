//! # 3D Vector Mathematics Module
//!
//! This module provides a 3D vector implementation specifically
//! optimized for flocking simulations and particle systems. The `Vec3` struct
//! offers vector operations with both value and reference semantics
//! to minimize allocations in performance-critical code.
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
// Numerical Operations Definition
pub mod ops;
// Helper Functions for Vec3 struct
pub mod math;
// Unit tests
pub mod tests;

/// A 3D vector in Cartesian coordinates optimized for flocking simulations.
///
/// `Vec3` represents a point or direction in 3D space using double-precision
/// floating-point components. It implements `Copy` for efficient passing and
/// provides comprehensive mathematical operations essential for particle physics
/// and flocking behavior simulations.
///
/// # Fields
/// - `x`: The X-component
/// - `y`: The Y-component
/// - `z`: The Z-component
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
#[derive(Default, Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
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
    pub fn zero() -> Self {
        Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
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
    pub fn z_hat() -> Self {
        Vec3::new(0.0, 0.0, 1.0)
    }
}
