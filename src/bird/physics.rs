//! # Physics Module for Bird Movement and Interactions
//!
//! This module contains physics-related methods for bird movement, distance calculations,
//! velocity transport, and noise addition for realistic flocking behavior on spherical
//! surfaces. The methods implement the mathematical foundations for flocking simulations
//! including geodesic distances, parallel transport of vectors, and stochastic dynamics.

use crate::bird::Bird;
use crate::vector::Vec3;

impl Bird {
    /// Calculates the geodesic distance between two birds on a sphere surface.
    ///
    /// This method computes the shortest path distance between two birds along the
    /// surface of a sphere using the angular separation between their position vectors.
    /// The geodesic distance is the arc length along the sphere surface.
    ///
    /// # Arguments
    ///
    /// * `other` - Reference to another bird to calculate distance to
    /// * `radius` - Radius of the sphere on which birds are constrained
    ///
    /// # Returns
    ///
    /// The geodesic distance as a positive `f64` value representing the arc length
    /// along the sphere surface between the two bird positions.
    ///
    /// # Mathematical Background
    ///
    /// For two position vectors **r₁** and **r₂** on a sphere of radius R, the
    /// geodesic distance is: `d = R × arccos(r₁ · r₂ / (|r₁| × |r₂|))`
    /// This is equivalent to: `d = R × θ` where θ is the angle between vectors.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use flocking_lib::bird::Bird;
    /// # use flocking_lib::vector::Vec3;
    /// let bird1 = Bird::new(Vec3::new(1.0, 0.0, 0.0), Vec3::zero());
    /// let bird2 = Bird::new(Vec3::new(0.0, 1.0, 0.0), Vec3::zero());
    /// let distance = bird1.distance_from(&bird2, 1.0); // π/2 ≈ 1.57
    /// ```
    pub fn distance_from(&self, other: &Bird, radius: f64) -> f64 {
        self.position.angle_between(&other.position) * radius
    }

    /// Performs parallel transport of this bird's velocity to another bird's position.
    ///
    /// Parallel transport is a fundamental concept in differential geometry that
    /// allows vectors to be "moved" along curved surfaces while preserving their
    /// intrinsic direction. This is essential for flocking simulations on spheres
    /// where velocity vectors must be compared at different positions.
    ///
    /// # Arguments
    ///
    /// * `base` - Reference to the target bird whose position defines the transport destination
    ///
    /// # Returns
    ///
    /// A new `Vec3` representing this bird's velocity transported to the base bird's
    /// position, maintaining the velocity's tangential nature to the sphere.
    ///
    /// # Mathematical Background
    ///
    /// The parallel transport rotates the velocity vector around the axis perpendicular
    /// to both position vectors by the angle between them:
    /// - **axis** = **r₁** × **r₂** / |**r₁** × **r₂**|
    /// - **angle** = arccos(**r₁** · **r₂** / (|**r₁**| × |**r₂**|))
    /// - **v'** = Rotate(**v**, **axis**, **angle**)
    ///
    /// # Special Cases
    ///
    /// When the two positions are identical or antipodal (axis ≈ 0), the original
    /// velocity is returned unchanged as no transport is needed.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use flocking_lib::bird::Bird;
    /// # use flocking_lib::vector::Vec3;
    /// let bird1 = Bird::new(Vec3::new(1.0, 0.0, 0.0), Vec3::new(0.0, 1.0, 0.0));
    /// let bird2 = Bird::new(Vec3::new(0.0, 1.0, 0.0), Vec3::zero());
    /// let transported_vel = bird1.parallel_transport_velocity(&bird2);
    /// ```
    pub fn parallel_transport_velocity(&self, base: &Bird) -> Vec3 {
        let angle = self.position.angle_between(&base.position);
        let axis = self.position.cross(&base.position).normalize();
        if axis.approx_eq(&Vec3::zero(), 1e-10) {
            // If the axis is zero, return the original velocity
            return self.velocity;
        }
        match self.velocity.rotate_around(&axis, angle) {
            Some(velocity) => velocity,
            None => {
                unreachable!(
                    "Velocity rotation failed, which should not happen with valid inputs."
                );
            }
        }
    }

    /// Generates random angular noise for stochastic flocking dynamics.
    ///
    /// This function produces normally distributed random angles used to introduce
    /// noise into bird velocity directions, simulating environmental perturbations
    /// and individual behavioral variations in flocking systems.
    ///
    /// # Arguments
    ///
    /// * `order_parameter` - Standard deviation of the normal distribution controlling
    ///   noise strength. Higher values produce more chaotic behavior, lower values
    ///   result in more ordered flocking.
    ///
    /// # Returns
    ///
    /// A random angle in radians sampled from N(0, σ²) where σ is the order parameter.
    ///
    /// # Panics
    ///
    /// Panics if `order_parameter` is zero or negative, as this would result in
    /// invalid noise distribution parameters.
    ///
    /// # Mathematical Background
    ///
    /// The noise follows: θ ~ N(0, η²) where η is the order parameter.
    /// This implements the stochastic component of the Vicsek model and similar
    /// flocking algorithms where noise strength controls the order-disorder transition.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use flocking_lib::bird::Bird;
    /// let noise = Bird::random_angle_noise(0.1); // Low noise for ordered flocking
    /// let chaos = Bird::random_angle_noise(1.0); // High noise for disordered motion
    /// ```
    #[inline]
    pub fn random_angle_noise(order_parameter: f64) -> f64 {
        use rand::prelude::*;
        use rand_distr::Normal;
        if order_parameter < f64::EPSILON {
            unreachable!("Order parameter must be greater than zero for random angle generation.");
        }

        let mut rng = rand::rng();
        let normal = Normal::new(0.0, order_parameter).unwrap();
        normal.sample(&mut rng)
    }

    /// Adds angular noise to an averaged velocity vector around a reference position.
    ///
    /// This method applies random rotational noise to a velocity vector, typically
    /// used after averaging neighbor velocities in flocking algorithms. The noise
    /// is applied as a rotation around the local normal (position) vector.
    ///
    /// # Arguments
    ///
    /// * `averaged` - The base velocity vector (often averaged from neighbors)
    /// * `base` - Reference bird providing the rotation axis (position normal)
    /// * `order_parameter` - Noise strength parameter passed to `random_angle_noise`
    ///
    /// # Returns
    ///
    /// A new `Vec3` representing the input velocity with added angular noise,
    /// rotated around the base bird's position vector (sphere normal).
    ///
    /// # Mathematical Implementation
    ///
    /// 1. Generate random angle: θ ~ N(0, η²)
    /// 2. Rotation axis: **n** = **r_base** / |**r_base**|
    /// 3. Apply rotation: **v'** = Rotate(**v_avg**, **n**, θ)
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use flocking_lib::bird::Bird;
    /// # use flocking_lib::vector::Vec3;
    /// let base_bird = Bird::new(Vec3::new(0.0, 0.0, 1.0), Vec3::zero());
    /// let avg_velocity = Vec3::new(1.0, 0.0, 0.0);
    /// let noisy_vel = Bird::add_noise(avg_velocity, &base_bird, 0.2);
    /// ```
    pub fn add_noise(averaged: Vec3, base: &Bird, order_parameter: f64) -> Vec3 {
        let noise = Self::random_angle_noise(order_parameter);
        averaged
            .rotate_around(&base.position.normalize(), noise)
            .unwrap()
    }

    /// Moves a bird position along the sphere surface given velocity and time step.
    ///
    /// This function implements geodesic motion on a sphere surface, updating a bird's
    /// position based on its tangent velocity. The movement preserves the constraint
    /// that the bird remains on the sphere surface of the specified radius.
    ///
    /// # Arguments
    ///
    /// * `current` - Current position vector of the bird
    /// * `velocity` - Tangent velocity vector (should be perpendicular to position)
    /// * `dt` - Time step duration
    /// * `radius` - Sphere radius for constraint maintenance
    /// * `speed` - Magnitude of velocity for the movement calculation
    ///
    /// # Returns
    ///
    /// New position `Vec3` after moving along the sphere surface for time `dt`.
    ///
    /// # Mathematical Background
    ///
    /// The geodesic motion on a sphere is implemented using Rodrigues' rotation formula:
    /// - Angular displacement: α = (speed × dt) / radius
    /// - **r'** = **r** × cos(α) + (radius × sin(α)) × **v̂**
    ///
    /// where **v̂** is the normalized velocity direction.
    ///
    /// # Sphere Constraint
    ///
    /// The resulting position automatically maintains |**r'**| = radius, ensuring
    /// the bird remains on the sphere surface throughout the simulation.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use flocking_lib::bird::Bird;
    /// # use flocking_lib::vector::Vec3;
    /// let current_pos = Vec3::new(1.0, 0.0, 0.0);
    /// let velocity = Vec3::new(0.0, 1.0, 0.0);
    /// let new_pos = Bird::move_bird(&current_pos, &velocity, 0.1, 1.0, 2.0);
    /// ```
    pub fn move_bird(current: &Vec3, velocity: &Vec3, dt: f64, radius: f64, speed: f64) -> Vec3 {
        let angle = speed * dt / radius;
        *current * angle.cos() + (radius * angle.sin()) * velocity.normalize()
    }
}
