
use std::ops::{Add, Sub, Mul, Div, Neg};

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Vec3 { pub x: f64, pub y: f64, pub z: f64 }

impl Vec3 {
    pub fn new(x: f64, y: f64, z: f64) -> Self { Self { x, y, z } }
    pub fn dot(self, o: Self) -> f64 { self.x*o.x + self.y*o.y + self.z*o.z }
    pub fn cross(self, o: Self) -> Self {
        Self::new(
            self.y*o.z - self.z*o.y,
            self.z*o.x - self.x*o.z,
            self.x*o.y - self.y*o.x
        )
    }
    pub fn length(self) -> f64 { self.dot(self).sqrt() }
    pub fn normalized(self) -> Self { let l = self.length(); if l > 0.0 { self / l } else { self } }
}

impl Add for Vec3 {
    type Output = Self;
    fn add(self, o: Self) -> Self { Self::new(self.x + o.x, self.y + o.y, self.z + o.z) }
}
impl Sub for Vec3 {
    type Output = Self;
    fn sub(self, o: Self) -> Self { Self::new(self.x - o.x, self.y - o.y, self.z - o.z) }
}
impl Mul<f64> for Vec3 {
    type Output = Self;
    fn mul(self, s: f64) -> Self { Self::new(self.x * s, self.y * s, self.z * s) }
}
impl Div<f64> for Vec3 {
    type Output = Self;
    fn div(self, s: f64) -> Self { Self::new(self.x / s, self.y / s, self.z / s) }
}
impl Neg for Vec3 {
    type Output = Self;
    fn neg(self) -> Self { Self::new(-self.x, -self.y, -self.z) }
}

// Útil para permitir 2.0 * vec en addition a vec * 2.0
impl Mul<Vec3> for f64 {
    type Output = Vec3;
    fn mul(self, v: Vec3) -> Vec3 { v * self }
}

pub type Color = Vec3;

pub fn clamp01(x: f64) -> f64 { if x < 0.0 { 0.0 } else if x > 1.0 { 1.0 } else { x } }
pub fn to_u8(x: f64) -> u8 { (clamp01(x).powf(1.0/2.2) * 255.0 + 0.5) as u8 }
