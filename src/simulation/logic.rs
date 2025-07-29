use crate::bird::Bird;
use crate::simulation::{FrameData, Simulation};
use rayon::iter::IntoParallelRefMutIterator;
use std::sync::atomic::Ordering;

impl Simulation {
    /// Runs the simulation for a specified number of steps
    ///
    /// This method will execute exactly the requested number of simulation steps,
    /// regardless of the stop flag state.
    ///
    /// # Arguments
    ///
    /// * `num_steps` - Number of simulation steps to execute
    pub fn run_for_steps(&mut self, num_steps: u64) {
        for _ in 0..num_steps {
            self.step();

            // Send frame data if interval reached
            if self.step_count % self.frame_interval == 0 {
                self.send_frame_data();
            }
        }
    }

    /// Runs the simulation until the stop flag is set
    ///
    /// This method runs indefinitely until `should_stop` is set to true
    /// from another thread. Useful for interactive simulations or when
    /// the stopping condition is external.
    pub fn run_until_stopped(&mut self) {
        while !self.should_stop.load(Ordering::Relaxed) {
            self.step();

            // Send frame data if interval reached
            if self.step_count % self.frame_interval == 0 {
                self.send_frame_data();
            }
        }
    }

    /// Runs the simulation with a maximum step limit and stop condition
    ///
    /// Combines both approaches: runs until either the step limit is reached
    /// or the stop flag is set, whichever comes first.
    ///
    /// # Arguments
    ///
    /// * `max_steps` - Maximum number of steps to run
    pub fn run_with_limit(&mut self, max_steps: u64) {
        for _ in 0..max_steps {
            if self.should_stop.load(Ordering::Relaxed) {
                break;
            }

            self.step();

            // Send frame data if interval reached
            if self.step_count % self.frame_interval == 0 {
                self.send_frame_data();
            }
        }
    }

    /// Performs a single simulation step using double buffering and parallel processing
    ///
    /// This is the core simulation method that:
    /// 1. Reads the current state from one buffer
    /// 2. Calculates new states in parallel using rayon
    /// 3. Writes results to the other buffer
    /// 4. Swaps buffers for the next iteration
    fn step(&mut self) {
        // Determine which buffer is current and which is next
        // We read from particles_a and write to particles_b
        let current_state = &self.particles_a;
        let next_state = &mut self.particles_b;

        // Parallel computation using rayon for maximum CPU utilization
        // Each thread processes a subset of particles independently
        next_state
            .par_iter_mut()
            .enumerate()
            .for_each(|(i, particle_next)| {
                // Calculate the new state for particle i based on current state
                *particle_next = self.calculate_new_particle_state(i, current_state);
            });

        // Swap buffers - this is extremely cheap (just pointer swaps)
        std::mem::swap(&mut self.particles_a, &mut self.particles_b);

        // Update simulation state
        self.step_count += 1;
        self.current_time += self.params.dt;
    }

    /// Calculates the new state for a single particle based on flocking rules
    ///
    /// This function contains the core physics and flocking behavior logic.
    /// It computes forces from neighboring particles and updates position/velocity.
    ///
    /// # Arguments
    ///
    /// * `particle_index` - Index of the particle to update
    /// * `current_state` - Read-only reference to all current particle states
    ///
    /// # Returns
    ///
    /// New `Bird` state for the specified particle
    fn calculate_new_particle_state(&self, particle_index: usize, current_state: &[Bird]) -> Bird {
        // TODO: Implement the core flocking physics here
        // This should include:
        // 1. Find neighboring particles within interaction radius
        // 2. Calculate alignment, cohesion, and separation forces
        // 3. Apply noise and speed limits
        // 4. Integrate motion equations
        // 5. Constrain to sphere surface

        unimplemented!("Flocking physics calculation - implement based on bird::physics methods")
    }

    /// Finds all neighbors within the interaction radius of a given particle
    ///
    /// # Arguments
    ///
    /// * `particle_index` - Index of the particle to find neighbors for
    /// * `current_state` - Current state of all particles
    ///
    /// # Returns
    ///
    /// Vector of indices of neighboring particles
    fn find_neighbors(&self, particle_index: usize, current_state: &[Bird]) -> Vec<usize> {
        // TODO: Implement efficient neighbor finding
        // Consider spatial data structures for large particle counts

        unimplemented!("Neighbor finding algorithm")
    }

    /// Calculates alignment force based on neighboring bird velocities
    ///
    /// # Arguments
    ///
    /// * `current_bird` - The bird to calculate alignment for
    /// * `neighbors` - Slice of neighboring birds
    ///
    /// # Returns
    ///
    /// Alignment force vector
    fn calculate_alignment_force(
        &self,
        current_bird: &Bird,
        neighbors: &[Bird],
    ) -> crate::vector::Vec3 {
        // TODO: Implement alignment force calculation
        // Average neighbor velocities and create force toward that direction

        unimplemented!("Alignment force calculation")
    }

    /// Calculates cohesion force toward the center of neighboring birds
    ///
    /// # Arguments
    ///
    /// * `current_bird` - The bird to calculate cohesion for
    /// * `neighbors` - Slice of neighboring birds
    ///
    /// # Returns
    ///
    /// Cohesion force vector
    fn calculate_cohesion_force(
        &self,
        current_bird: &Bird,
        neighbors: &[Bird],
    ) -> crate::vector::Vec3 {
        // TODO: Implement cohesion force calculation
        // Calculate center of mass of neighbors and create force toward it

        unimplemented!("Cohesion force calculation")
    }

    /// Calculates separation force away from neighboring birds
    ///
    /// # Arguments
    ///
    /// * `current_bird` - The bird to calculate separation for
    /// * `neighbors` - Slice of neighboring birds
    ///
    /// # Returns
    ///
    /// Separation force vector
    fn calculate_separation_force(
        &self,
        current_bird: &Bird,
        neighbors: &[Bird],
    ) -> crate::vector::Vec3 {
        // TODO: Implement separation force calculation
        // Create repulsive forces from nearby neighbors

        unimplemented!("Separation force calculation")
    }

    /// Applies random noise to bird motion for realistic behavior
    ///
    /// # Arguments
    ///
    /// * `bird` - The bird to apply noise to
    ///
    /// # Returns
    ///
    /// Noise force vector
    fn apply_noise(&self, bird: &Bird) -> crate::vector::Vec3 {
        // TODO: Implement noise generation
        // Use random number generator to create small perturbations

        unimplemented!("Noise generation")
    }

    /// Constrains bird motion to the sphere surface
    ///
    /// # Arguments
    ///
    /// * `bird` - Bird to constrain
    ///
    /// # Returns
    ///
    /// Constrained bird state
    fn constrain_to_sphere(&self, mut bird: Bird) -> Bird {
        // TODO: Implement sphere constraint
        // Normalize position to sphere radius and make velocity tangent

        unimplemented!("Sphere surface constraint")
    }

    /// Sends current frame data through the I/O channel
    ///
    /// This method creates a snapshot of the current simulation state and
    /// sends it asynchronously for disk I/O processing.
    fn send_frame_data(&self) {
        if let Some(ref sender) = self.frame_sender {
            let frame = FrameData {
                step: self.step_count,
                timestamp: self.current_time,
                birds: self.particles_a.clone(),
            };

            // Non-blocking send - if receiver is gone, just continue
            let _ = sender.try_send(frame);
        }
    }

    /// Gracefully stops the simulation by setting the stop flag
    pub fn stop(&self) {
        self.should_stop.store(true, Ordering::Relaxed);
    }

    /// Resets the simulation to initial conditions
    ///
    /// # Arguments
    ///
    /// * `initial_birds` - New initial bird configuration
    pub fn reset(&mut self, initial_birds: Vec<Bird>) {
        let num_particles = initial_birds.len();

        self.particles_a = initial_birds;
        self.particles_b = vec![
            Bird::new(
                crate::vector::Vec3::new(0.0, 0.0, 0.0),
                crate::vector::Vec3::new(0.0, 0.0, 0.0)
            );
            num_particles
        ];
        self.step_count = 0;
        self.current_time = 0.0;
        self.should_stop.store(false, Ordering::Relaxed);
    }
}
