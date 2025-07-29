use crate::bird::Bird;
use crate::vector::Vec3;

/// Calculates the new state for a single particle based on flocking rules
///
/// This function contains the core physics and flocking behavior logic.
/// It computes forces from neighboring particles and updates position/velocity.
///
/// # Arguments
///
/// * `particle_index` - Index of the particle to update
/// * `current_state` - Read-only reference to all current particle states
/// * `dt` - Time step for integration
/// * `interaction_radius` - Radius for neighbor detection
/// * `eta` - Noise parameter
/// * `speed` - Constant speed constraint
/// * `radius` - Sphere radius
///
/// # Returns
///
/// New `Bird` state for the specified particle
pub fn calculate_new_particle_state(
    particle_index: usize,
    current_state: &[Bird],
    dt: f64,
    interaction_radius: f64,
    eta: f64,
    speed: f64,
    radius: f64,
) -> Bird {
    let current_bird = &current_state[particle_index];

    // 1. Find neighboring particles within interaction radius and parallel transport their velocities
    let transported_velocities: Vec<Vec3> = current_state
        .iter()
        .enumerate()
        .filter_map(|(i, neighbor)| {
            // Skip self
            if i == particle_index {
                return None;
            }

            // Check if neighbor is within interaction radius
            let distance = current_bird.distance_from(neighbor, radius);
            if distance < interaction_radius {
                // Parallel transport neighbor's velocity to current bird's position
                Some(neighbor.parallel_transport_velocity(current_bird))
            } else {
                None
            }
        })
        .collect();

    // 2. Calculate average velocity from neighbors
    let new_velocity = if transported_velocities.is_empty() {
        // No neighbors found, keep current velocity
        current_bird.velocity
    } else {
        // Sum all transported velocities and divide by number of neighbors
        let velocity_sum = transported_velocities
            .iter()
            .fold(Vec3::zero(), |acc, &vel| acc + vel);
        let averaged_velocity = velocity_sum / transported_velocities.len() as f64;

        // 3. Check if the norm of resulting vector is not small
        let velocity_norm = averaged_velocity.norm();
        if velocity_norm < 1e-6 {
            // Norm is too small, use previous velocity without averaging
            Bird::add_noise(current_bird.velocity, current_bird, eta)
        } else {
            // 4. Normalize and multiply by speed, then apply noise
            let normalized_velocity = averaged_velocity.normalize() * speed;
            Bird::add_noise(normalized_velocity, current_bird, eta)
        }
    };

    // 5. Move bird with the new velocity
    let updated_bird = Bird::new(current_bird.position, new_velocity);
    updated_bird.move_on_sphere(dt, radius, speed)
}
