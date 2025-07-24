pub(crate) use crate::bird::Bird;

/// Holds the global parameters and state for a single simulation run.
#[derive(Debug)]
pub struct FlockSimulation {
    pub particles: Vec<Bird>,
    pub params: SimulationParams,
}

/// Parameters that define the physics of a simulation run.
#[derive(Debug, Clone, Copy)]
pub struct SimulationParams {
    pub speed: f64,
    pub sphere_radius: f64,
    pub interaction_radius: f64,
    pub repulsion_radius: f64,
    pub noise: f64,
}

impl FlockSimulation {
    /// Creates a new simulation from a set of particles and parameters.
    pub fn new(particles: Vec<Bird>, params: SimulationParams) -> Self {
        unimplemented!()
    }

    /// Performs a single time step of the simulation.
    pub fn step(&mut self, dt: f64) {
        // This is the main engine. It will call all the helper functions.
        unimplemented!()
    }
}

// --- Helper Functions for the Physics Engine ---

/// Calculates the great-circle distance between two particles.
pub fn great_circle_distance(p1: &Bird, p2: &Bird, radius: f64) -> f64 {
    unimplemented!()
}

/// Parallel transports a velocity vector from a start point to an end point.
pub fn parallel_transport(
    velocity: &crate::vector::Vec3,
    start: &Bird,
    end: &Bird,
) -> crate::vector::Vec3 {
    unimplemented!()
}

/// Generates a random noise angle based on the chosen distribution and noise parameter.
pub fn generate_noise_angle(noise_param: f64) -> f64 {
    unimplemented!()
}
