use std::ops::{Add, Div, Mul, Sub};

/// A 3D vector in Cartesian coordinates. The workhorse of our simulation.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

// --- Implementation of Vector Math ---
impl Vec3 {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        unimplemented!()
    }
    pub fn zero() -> Self {
        unimplemented!()
    }
    pub fn norm(&self) -> f64 {
        unimplemented!()
    }
    pub fn norm_squared(&self) -> f64 {
        unimplemented!()
    }
    pub fn normalize(&self) -> Self {
        unimplemented!()
    }
    pub fn dot(&self, other: &Self) -> f64 {
        unimplemented!()
    }
    pub fn cross(&self, other: &Self) -> Self {
        unimplemented!()
    }
}

// --- Operator Overloads for Ergonomics ---
impl Add for Vec3 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        unimplemented!()
    }
}
impl Sub for Vec3 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        unimplemented!()
    }
}
impl Mul<f64> for Vec3 {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self::Output {
        unimplemented!()
    }
}
impl Div<f64> for Vec3 {
    type Output = Self;
    fn div(self, rhs: f64) -> Self::Output {
        unimplemented!()
    }
}
