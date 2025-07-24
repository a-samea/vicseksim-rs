use crate::bird::SphericalCoord;

/// Generates a set of `N` non-colliding particles in spherical coordinates.
pub fn generate_initial_spherical_coords(
    num_particles: usize,
    min_distance: f64,
    sphere_radius: f64,
) -> Result<Vec<SphericalCoord>, String> {
    unimplemented!()
}

/// Converts a set of spherical coordinates to a Vec of Cartesian Particles.
pub fn convert_spherical_to_cartesian(
    coords: &[SphericalCoord],
    speed: f64,
) -> Vec<crate::bird::Bird> {
    unimplemented!()
}
