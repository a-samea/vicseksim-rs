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

use rand::Rng;
use rayon::prelude::*;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::sync::mpsc::{self, Sender};
use std::thread;
use std::time::Instant;

/// Represents a single particle in our simulation.
/// It must be `Copy` so it's cheap to pass around and `Clone`.
/// `Send` and `Sync` are required by `rayon` to safely move data across threads.
#[derive(Debug, Copy, Clone)]
struct Particle {
    position: f64,
    velocity: f64,
    acceleration: f64,
}

// We implement Default to easily create a vector of default particles.
impl Default for Particle {
    fn default() -> Self {
        Particle {
            position: 0.0,
            velocity: 0.0,
            acceleration: 0.0,
        }
    }
}

/// The main simulation struct. It holds the state and manages the simulation process.
struct Simulation {
    // The double-buffering strategy: we read from one and write to the other.
    particles_a: Vec<Particle>,
    particles_b: Vec<Particle>,
    step_count: u32,
    // The "sender" end of a channel to send data to the I/O thread.
    io_sender: Sender<Vec<Particle>>,
}

impl Simulation {
    /// Creates a new simulation with a given number of particles.
    pub fn new(num_particles: usize, io_sender: Sender<Vec<Particle>>) -> Self {
        let mut rng = rand::thread_rng();
        let particles_a = (0..num_particles)
            .map(|_| Particle {
                position: rng.gen_range(-100.0..100.0),
                velocity: rng.gen_range(-10.0..10.0),
                acceleration: 0.0,
            })
            .collect();

        // Pre-allocate the second buffer to be the same size.
        let particles_b = vec![Particle::default(); num_particles];

        Simulation {
            particles_a,
            particles_b,
            step_count: 0,
            io_sender,
        }
    }

    /// Runs the entire simulation for a given number of steps.
    pub fn run(&mut self, total_steps: u32, save_interval: u32) {
        for i in 0..total_steps {
            // Optional: Save the current state to the file via the I/O thread.
            // We clone `particles_a` so the simulation can continue immediately
            // while the I/O thread writes the data.
            if i % save_interval == 0 {
                println!("Step {}: Sending data to I/O thread.", self.step_count);
                if self.io_sender.send(self.particles_a.clone()).is_err() {
                    // This error means the I/O thread has terminated.
                    eprintln!("Error: Could not send data to I/O thread. It might have panicked.");
                    break;
                }
            }

            self.step();
        }
        println!("Simulation finished after {} steps.", self.step_count);
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

fn main() {
    // --- Simulation Parameters ---
    let num_particles = 1_000;
    let total_steps = 200;
    let save_interval = 20; // Save data every 20 steps
    let output_filename = "simulation_output.txt";

    // --- I/O Thread Setup ---
    // 1. Create a multi-producer, single-consumer (mpsc) channel.
    let (tx, rx) = mpsc::channel::<Vec<Particle>>();

    // 2. Spawn a dedicated I/O thread.
    // The `move` keyword gives the thread ownership of `rx` and the filename.
    let io_thread_handle = thread::spawn(move || {
        println!("I/O thread started. Writing to '{}'.", output_filename);
        let file = File::create(output_filename).expect("Could not create output file.");
        let mut writer = BufWriter::new(file);

        // This loop will block until a message is received.
        // It will automatically exit when the `tx` (sender) is dropped.
        for (i, received_particles) in rx.iter().enumerate() {
            writeln!(writer, "--- Frame {} ---", i * save_interval).unwrap();
            for p in received_particles {
                writeln!(writer, "{:?}", p).unwrap();
            }
        }
        println!("I/O thread finished.");
    });

    // --- Simulation Setup and Execution ---
    let mut simulation = Simulation::new(num_particles, tx);

    println!(
        "Starting simulation with {} particles for {} steps.",
        num_particles, total_steps
    );
    let start_time = Instant::now();

    simulation.run(total_steps, save_interval);

    let duration = start_time.elapsed();
    println!("Total simulation time: {:?}", duration);

    // --- Graceful Shutdown ---
    // The sender (`tx`) is part of the `simulation` struct. When `simulation` goes
    // out of scope here at the end of `main`, `tx` is dropped. This closes the
    // channel, causing the `for` loop in the I/O thread to terminate.

    // We explicitly wait for the I/O thread to finish its work. This ensures
    // that all data is written to the file before the program exits.
    io_thread_handle
        .join()
        .expect("I/O thread panicked during execution.");

    println!("Program finished successfully.");
}
