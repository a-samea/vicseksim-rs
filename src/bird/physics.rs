use crate::bird::Bird;
use crate::vector::Vec3;

impl Bird {
    pub fn distance_from(&self, other: Bird) -> f64 {
        unimplemented!()
    }

    pub fn parallel_transport_velocity(&self, base: Bird) -> Vec3 {
        unimplemented!()
    }

    pub fn add_noise(averaged: Vec3, order_parameter: f64) -> Vec3 {
        unimplemented!()
    }

    pub fn move_bird(current: Vec3, speed: f64, dt: f64, radius: f64) -> Vec3 {
        unimplemented!()
    }
}
