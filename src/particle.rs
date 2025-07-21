use crate::vector::Vec3;

/// Represents a single particle on the surface of the sphere.
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct Particle {
    pub position: Vec3,
    pub velocity: Vec3,
}

impl Particle {
    /// Creates a new particle from Cartesian components, ensuring velocity is tangent.
    pub fn new(position: Vec3, velocity: Vec3) -> Self {
        unimplemented!()
    }
}

/// Represents the initial spherical coordinates for generating a particle.
/// This struct is for user input and initial state generation ONLY.
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct SphericalCoord {
    pub theta: f64, // Colatitude
    pub phi: f64,   // Azimuth
    pub alpha: f64, // Tangent velocity angle
}

impl SphericalCoord {
    /// Converts these spherical coordinates into a valid Cartesian Particle.
    pub fn to_particle(&self, speed: f64) -> Particle {
        unimplemented!()
    }
}
