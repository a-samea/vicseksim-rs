use crate::bird::Bird;
use rayon::prelude::*;
use std::sync::mpsc::Sender;

#[derive(Debug)]
pub struct Simulation {
    // The double-buffering strategy: we read from one and write to the other.
    pub particles_a: Vec<Bird>,
    pub particles_b: Vec<Bird>,
    pub params: SimulationParams,
    // The "sender" end of a channel to send data to the I/O thread.
    pub io_sender: Sender<Vec<Bird>>,
}

/// Parameters that define the physics of a simulation run.
#[derive(Debug, Clone, Copy)]
pub struct SimulationParams {
    pub number_of_particles: usize,
    pub radius: f64,
    pub speed: f64,
    pub dt: f64,
    pub interaction_distance: f64,
    pub eta: f64,
}

impl Simulation {
    pub fn new(
        initial_value: Vec<Bird>,
        interaction_distance: f64,
        dt: f64,
        noise_parameter: f64,
        io_sender: Sender<Vec<Bird>>,
    ) -> Self {
        let radius = initial_value[0].position.norm();
        let speed = initial_value[0].velocity.norm();
        let particles_a = initial_value;
        let num_particles = particles_a.len();
        if num_particles == 0 {
            panic!("Simulation must be initialized with at least one particle.");
        }
        let particles_b = vec![Bird::default(); num_particles];
        let params = SimulationParams {
            number_of_particles: num_particles,
            radius,
            speed,
            dt,
            interaction_distance,
            eta: noise_parameter,
        };

        Simulation {
            particles_a,
            particles_b,
            params,
            io_sender,
        }
    }

    /// Runs the entire simulation for a given number of steps.
    pub fn run(&mut self, total_steps: u32, save_interval: u32) {
        for i in 0..total_steps {
            // Optional: Save the current state to the file via the I/O thread.
            // We clone `particles_a` so the simulation can continue immediately
            // while the I/O thread writes the data.
            let time = i * self.params.dt;
            if i % save_interval == 0 {
                println!("Step {}: Sending data to I/O thread.", time);
                if self.io_sender.send(self.particles_a.clone()).is_err() {
                    // This error means the I/O thread has terminated.
                    eprintln!("Error: Could not send data to I/O thread. It might have panicked.");
                    break;
                }
            }
            self.step();
        }
    }

    /// Performs a single step of the simulation.
    fn step(&mut self) {
        // Determine which buffer is current and which is next.
        // We will read from `current` and write the new state into `next`.
        let (current_state, next_state) = (&self.particles_a, &mut self.particles_b);

        // This is the core of the parallel computation using `rayon`.
        // `par_iter_mut()` gives us a parallel iterator over mutable items.
        next_state
            .par_iter_mut()
            .enumerate()
            .for_each(|(i, particle_next)| {
                // The `current_state` vector is borrowed immutably and is
                // accessible safely from all threads. This is the "lock" we wanted.
                *particle_next = calculate_new_particle_state(i, current_state);
            });

        // Swap the buffers. The new state in `particles_b` now becomes the
        // current state in `particles_a` for the next iteration.
        // This is a very cheap operation, it only swaps pointers.
        std::mem::swap(&mut self.particles_a, &mut self.particles_b);
        self.step_count += 1;
    }
}

/// The pure calculation logic for a single particle's next state.
/// This function is where the physics of your simulation would live.
/// It depends only on the particle's index and the *entire* previous state.
fn calculate_new_particle_state(index: usize, all_particles: &[Particle]) -> Particle {
    let current_particle = all_particles[index];
    let mut force = 0.0;
    let time_step = 0.1;

    // A simple N-body simulation: every particle attracts every other particle.
    // This is computationally intensive and benefits greatly from parallelism.
    for (other_idx, other_particle) in all_particles.iter().enumerate() {
        if index == other_idx {
            continue;
        }
        let distance = other_particle.position - current_particle.position;
        // Simple gravitational-like force
        if distance.abs() > 0.1 {
            force += 1.0 / distance;
        }
    }

    let mut new_particle = current_particle;
    new_particle.acceleration = force; // Assuming mass = 1
    new_particle.velocity += new_particle.acceleration * time_step;
    new_particle.position += new_particle.velocity * time_step;

    new_particle
}
