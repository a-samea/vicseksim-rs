//! Vector math operations for Vec3
//! Provides methods for vector normalization, dot and cross products, angle calculations,
//! and projections.

use super::Vec3;

impl Vec3 {
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
    /// assert!((normalized.norm() - 1.0).abs() < f64::EPSILON);
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
        let epsilon = epsilon.max(f64::EPSILON); // Ensure non-zero epsilon
        (self.x - other.x).abs() < epsilon
            && (self.y - other.y).abs() < epsilon
            && (self.z - other.z).abs() < epsilon
    }

    /// Rotates this vector around a normalized axis by the given angle.
    /// Uses Rodrigues' rotation formula: v' = v*cos(θ) + (k×v)*sin(θ) + k*(k·v)*(1-cos(θ))
    ///
    /// # Arguments
    /// * `axis` - The **normalized** unit vector representing the axis of rotation
    /// * `angle` - The angle in radians to rotate by
    ///
    /// # Returns
    /// * `Some(Vec3)` - The rotated vector if the axis is properly normalized
    /// * `None` - If the axis is zero, not normalized, or invalid
    ///
    /// # Examples
    /// ```
    /// # use flocking_lib::vector::Vec3;
    /// let v = Vec3::new(1.0, 0.0, 0.0);
    /// let axis = Vec3::new(0.0, 0.0, 1.0); // Z-axis (normalized)
    /// let rotated = v.rotate_around(&axis, std::f64::consts::PI / 2.0).unwrap();
    /// // Should rotate 90 degrees around Z-axis: (1,0,0) -> (0,1,0)
    /// ```
    pub fn rotate_around(&self, axis: &Self, angle: f64) -> Option<Self> {
        let axis_norm_sq = axis.norm_squared();

        // Check if axis is zero vector
        if axis_norm_sq < f64::EPSILON * f64::EPSILON {
            return None; // Cannot rotate around zero vector
        }

        // Check if axis is normalized (within tolerance)
        let tolerance = f64::EPSILON * 10.0; // Allow small numerical errors
        if (axis_norm_sq - 1.0).abs() > tolerance {
            return None; // Axis must be normalized
        }

        // Handle zero rotation angle
        if angle.abs() < f64::EPSILON {
            return Some(*self);
        }

        // Apply Rodrigues' rotation formula
        let cos_angle = angle.cos();
        let sin_angle = angle.sin();
        let cross_product = axis.cross(self);
        let dot_product = axis.dot(self);

        let rotated =
            *self * cos_angle + cross_product * sin_angle + *axis * dot_product * (1.0 - cos_angle);

        Some(rotated)
    }
}
