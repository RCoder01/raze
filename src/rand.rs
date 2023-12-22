#![allow(unused)]
use std::{
    num::Wrapping,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    math::{Mat3x3, Vec3},
    EPSILON,
};

pub fn sysnanos() -> u32 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Current system time is before unix time")
        .subsec_nanos()
}

#[derive(Debug, Clone)]
pub struct Lcg {
    state: Wrapping<u64>,
}

fn u64_from_u32s(a: u32, b: u32) -> u64 {
    a as u64 | ((b as u64) << 32)
}

impl Lcg {
    pub fn from_time() -> Self {
        Self::from_seed(u64_from_u32s(sysnanos(), sysnanos()))
    }

    pub fn from_seed(seed: u64) -> Self {
        let mut lcg = Self {
            state: Wrapping(seed),
        };
        lcg.pseudo_rand_u32();
        lcg.pseudo_rand_u32();
        lcg.pseudo_rand_u32();
        lcg
    }

    fn advance_state(&mut self) {
        // using LCG params as used in java
        self.state = (self.state * Wrapping(0x5DEECE66Du64) + Wrapping(11)) % Wrapping(2u64 << 48);
    }

    pub fn pseudo_rand_bool(&mut self) -> bool {
        self.advance_state();
        self.state.0 >> 16 & 0b1 == 0
    }

    pub fn pseudo_rand_u32(&mut self) -> u32 {
        self.advance_state();
        (self.state.0 >> 16) as u32
    }

    pub fn pseudo_rand_f32(&mut self) -> f32 {
        (self.pseudo_rand_u32() % (2 << 23)) as f32 / (2 << 23) as f32
    }

    pub fn pseudo_rand_u64(&mut self) -> u64 {
        u64_from_u32s(self.pseudo_rand_u32(), self.pseudo_rand_u32())
    }

    pub fn pseudo_rand_f64(&mut self) -> f64 {
        (self.pseudo_rand_u64() % (2u64 << 52)) as f64 / (2u64 << 52) as f64
    }
}

#[derive(Debug, Clone)]
pub struct Reflector {
    pub random: Lcg,
}

impl Reflector {
    pub fn new() -> Self {
        Self {
            random: Lcg::from_time(),
        }
    }

    fn random_unit(&mut self) -> Vec3 {
        let dir = self.random.pseudo_rand_f64() * std::f64::consts::TAU;
        let height = self.random.pseudo_rand_f64() * 2. - 1.;
        let (sin, cos) = dir.sin_cos();
        let xz = Vec3::new(cos, 0., sin);
        (1. - height.powi(2)).sqrt() * xz + height * Vec3::Y
    }

    pub fn random_diffuse(&mut self, normal: Vec3) -> Vec3 {
        let unit = self.random_unit();
        if unit.dot(normal).is_sign_negative() {
            -unit
        } else {
            unit
        }
    }
}
