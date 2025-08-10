//! # Bird Module - Flocking Simulation Particles
//!
//! This module defines the core `Bird` struct and its associated functionality for
//! simulating flocking behavior on a spherical surface. Each `Bird` represents a
//! single particle in the flock with position and velocity vectors in 3D Cartesian
//! coordinates.
//!
//! ## Submodules
//!
//! - [`physics`]: Contains physics-related methods for bird movement, distance calculations,
//!   velocity transport, and noise addition for realistic flocking behavior
//! - [`tests`]: Unit tests ensuring correctness of bird operations and physics
//!
//! ## Usage Example
//!
//! ```rust
//! use flocking_lib::bird::Bird;
//! use flocking_lib::vector::Vec3;
//!
//! // Create a bird at a specific position with initial velocity
//! let position = Vec3::new(1.0, 0.0, 0.0);  // On unit sphere
//! let velocity = Vec3::new(0.0, 1.0, 0.0);  // Tangent velocity
//! let bird = Bird::new(position, velocity);
//!
//! // Birds can be created from spherical coordinates
//! let bird_spherical = Bird::from_spherical(
//!     1.0,    // radius
//!     0.5,    // theta (polar angle)
//!     1.2,    // phi (azimuthal angle)
//!     2.0,    // speed
//!     0.3     // velocity angle alpha
//! );
//!
//! // Calculate distance between birds
//! let distance = bird.distance_from(bird_spherical);
//!
//! // Display bird information
//! println!("{}", bird);
//! ```

use crate::vector::Vec3;
use std::fmt::Display;

// Unit Tests
pub mod tests;
// Physics-related methods for bird movement and flocking behavior
pub mod physics;

// Represents a single particle on the surface of the sphere.
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct Bird {
    /// Position vector from sphere center to particle location
    pub position: Vec3,
    /// Velocity vector tangent to sphere surface at particle position
    pub velocity: Vec3,
}
impl Bird {
    /// Creates a new bird from Cartesian position and velocity vectors.
    ///
    /// This constructor creates a bird with the given position and velocity vectors.
    /// It's the responsibility of the caller to ensure that the velocity vector
    /// is tangent to the sphere surface at the given position for physically
    /// correct simulation behavior.
    ///
    /// # Arguments
    ///
    /// * `position` - 3D Cartesian position vector from sphere center
    /// * `velocity` - 3D velocity vector (**should be tangent to sphere surface**)
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use flocking_lib::bird::Bird;
    /// # use flocking_lib::vector::Vec3;
    /// // Create a bird on unit sphere at (1,0,0) moving in y-direction
    /// let position = Vec3::new(1.0, 0.0, 0.0);
    /// let velocity = Vec3::new(0.0, 1.0, 0.0);  // Tangent at this position
    /// let bird = Bird::new(position, velocity);
    /// ```
    fn new(position: Vec3, velocity: Vec3) -> Self {
        Bird { position, velocity }
    }

    /// Creates a new bird from spherical coordinates with velocity parameterization.
    ///
    /// This constructor converts spherical coordinates to Cartesian representation
    /// and generates a tangent velocity vector from speed and direction parameters.
    /// This is particularly useful for initial condition generation and analytical
    /// positioning of particles on the sphere surface.
    ///
    /// # Arguments
    ///
    /// * `radius` - Distance from sphere center (positive value)
    /// * `theta` - Polar angle from positive z-axis in radians [0, π]
    /// * `phi` - Azimuthal angle from positive x-axis in radians [0, 2π]
    /// * `speed` - Magnitude of velocity vector (non-negative)
    /// * `alpha` - Direction angle of velocity in local tangent plane (radians)
    ///
    /// # Mathematical Conversion
    ///
    /// The conversion from spherical to Cartesian coordinates follows:
    /// ```text
    /// x = r * sin(θ) * cos(φ)
    /// y = r * sin(θ) * sin(φ)
    /// z = r * cos(θ)
    /// ```
    ///
    /// The tangent velocity vector is constructed using the local basis vectors:
    /// ```text
    /// e_θ = (cos(θ)cos(φ), cos(θ)sin(φ), -sin(θ))  // θ-direction
    /// e_φ = (-sin(φ), cos(φ), 0)                     // φ-direction
    /// velocity = speed * (cos(α) * e_φ + sin(α) * e_θ)
    /// ```
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use flocking_lib::bird::Bird;
    /// use std::f64::consts::PI;
    ///
    /// // Bird at north pole moving eastward
    /// let bird_north = Bird::from_spherical(1.0, 0.0, 0.0, 2.0, 0.0);
    ///
    /// // Bird at equator (0° longitude) moving southward
    /// let bird_equator = Bird::from_spherical(1.0, PI/2.0, 0.0, 1.5, PI/2.0);
    ///
    /// // Bird at arbitrary position with random velocity direction
    /// let bird_random = Bird::from_spherical(2.0, PI/3.0, PI/4.0, 1.0, PI/6.0);
    /// ```
    pub fn from_spherical(radius: f64, theta: f64, phi: f64, speed: f64, alpha: f64) -> Self {
        // Ensure radius is positive
        assert!(radius > 0.0, "Radius must be positive");

        // Convert spherical coordinates to Cartesian position
        let x = radius * theta.sin() * phi.cos();
        let y = radius * theta.sin() * phi.sin();
        let z = radius * theta.cos();
        let position = Vec3::new(x, y, z);

        // Calculate tangent basis vectors at this position
        let theta_hat = Vec3::new(
            theta.cos() * phi.cos(),
            theta.cos() * phi.sin(),
            -theta.sin(),
        );
        let phi_hat = Vec3::new(-phi.sin(), phi.cos(), 0.0);

        // Construct velocity vector in local tangent plane
        let velocity = speed * (alpha.cos() * phi_hat + alpha.sin() * theta_hat);

        Bird { position, velocity }
    }
}
impl Display for Bird {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Calculate derived properties for display
        let speed = self.velocity.norm();
        let pos_norm = self.position.norm();

        // Format position with 3 decimal places
        let pos_str = format!(
            "({:.3}, {:.3}, {:.3})",
            self.position.x, self.position.y, self.position.z
        );

        // Format velocity with 3 decimal places
        let vel_str = format!(
            "({:.3}, {:.3}, {:.3})",
            self.velocity.x, self.velocity.y, self.velocity.z
        );

        // Calculate spherical coordinates for additional context
        let theta = pos_norm.atan2((self.position.x.powi(2) + self.position.y.powi(2)).sqrt());
        let phi = self.position.y.atan2(self.position.x);

        write!(
            f,
            "Bird {{ pos: {}, vel: {}, |v|: {:.3}, |r|: {:.3}, θ: {:.2}°, φ: {:.2}° }}",
            pos_str,
            vel_str,
            speed,
            pos_norm,
            theta.to_degrees(),
            phi.to_degrees()
        )
    }
}

impl Default for Bird {
    /// Creates a default bird at the origin with zero velocity.
    /// This is useful for initializing vectors of birds without specific
    /// initial conditions.
    fn default() -> Self {
        // Default bird at origin with zero velocity
        Bird {
            position: Vec3::zero(),
            velocity: Vec3::zero(),
        }
    }
}
