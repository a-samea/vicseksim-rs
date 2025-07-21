use crate::particle::SphericalCoord;
use crate::simulation::Particle;
use std::path::Path;

/// Saves a list of SphericalCoords to a file (e.g., for manual inspection).
pub fn save_spherical_coords_to_json(coords: &[SphericalCoord], path: &Path) -> Result<(), String> {
    unimplemented!()
}

/// Loads a list of SphericalCoords from a JSON file.
pub fn load_spherical_coords_from_json(path: &Path) -> Result<Vec<SphericalCoord>, String> {
    unimplemented!()
}

/// Saves a full simulation state (Vec<Particle>) to a binary file.
pub fn save_snapshot_to_binary(particles: &[Particle], path: &Path) -> Result<(), String> {
    unimplemented!()
}

/// Writes the header for a CSV file meant for Python visualization.
pub fn write_visualization_csv_header(
    writer: &mut csv::Writer<std::fs::File>,
) -> Result<(), csv::Error> {
    unimplemented!()
}

/// Appends the state of all particles for a single frame to a CSV writer.
pub fn append_frame_to_visualization_csv(
    writer: &mut csv::Writer<std::fs::File>,
    frame: u64,
    particles: &[Particle],
) -> Result<(), csv::Error> {
    unimplemented!()
}
