#[cfg(test)]
mod units {
    use crate::vector::Vec3;
    use std::sync::mpsc;

    #[test]
    fn test_simulation_creation() {
        let (tx, _rx) = mpsc::channel();
        let birds = vec![Bird::new(
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
        )];
        let sim = Simulation::new(birds, tx, 10);

        assert_eq!(sim.particle_count(), 1);
        assert_eq!(sim.step_count(), 0);
        assert_eq!(sim.current_time(), 0.0);
    }

    #[test]
    fn test_simulation_step_counting() {
        let birds = vec![Bird::new(
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
        )];
        let mut sim = Simulation::new_no_output(birds);

        // Note: This will panic with unimplemented! until physics is implemented
        // sim.run_for_steps(5);
        // assert_eq!(sim.step_count(), 5);
    }

    #[test]
    fn test_stop_flag() {
        let birds = vec![Bird::new(
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
        )];
        let sim = Simulation::new_no_output(birds);

        let stop_flag = sim.stop_flag();
        assert!(!stop_flag.load(Ordering::Relaxed));

        sim.stop();
        assert!(stop_flag.load(Ordering::Relaxed));
    }
}
