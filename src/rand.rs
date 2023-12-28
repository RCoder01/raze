#![allow(unused)]
use std::{
    cell::RefCell,
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

pub trait RandSource {
    fn next(&mut self) -> u32;

    fn rand<R: Rand>(&mut self) -> R
    where
        Self: Sized,
    {
        R::get(self)
    }
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
        lcg.next();
        lcg.next();
        lcg.next();
        lcg
    }

    fn advance_state(&mut self) {
        // using LCG params as used in java
        self.state = (self.state * Wrapping(0x5DEECE66Du64) + Wrapping(11)) % Wrapping(2u64 << 48);
    }

    // pub fn pseudo_rand_f32(&mut self) -> f32 {
    //     (self.pseudo_rand_u32() % (2 << 23)) as f32 / (2 << 23) as f32
    // }

    // pub fn pseudo_rand_f64(&mut self) -> f64 {
    //     (self.pseudo_rand_u64() % (2u64 << 52)) as f64 / (2u64 << 52) as f64
    // }
}

impl RandSource for Lcg {
    fn next(&mut self) -> u32 {
        self.advance_state();
        (self.state.0 >> 16) as u32
    }
}

#[derive(Debug, Clone)]
pub struct Reflector<R: RandSource> {
    pub random: R,
}

impl<R: RandSource> Reflector<R> {
    pub const fn new(random: R) -> Self {
        Self { random }
    }

    fn random_unit(&mut self) -> Vec3 {
        let dir = self.random.rand::<f64>() * std::f64::consts::TAU;
        let height = self.random.rand::<f64>() * 2. - 1.;
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

pub trait Rand {
    fn get(rng: &mut impl RandSource) -> Self;
}

impl Rand for bool {
    fn get(rng: &mut impl RandSource) -> Self {
        (rng.next() & 1) == 1
    }
}

impl Rand for u8 {
    fn get(rng: &mut impl RandSource) -> Self {
        (rng.next() & 0xFF) as u8
    }
}

impl Rand for u16 {
    fn get(rng: &mut impl RandSource) -> Self {
        (rng.next() & 0xFFFF) as u16
    }
}

impl Rand for u32 {
    fn get(rng: &mut impl RandSource) -> Self {
        rng.next()
    }
}

impl Rand for u64 {
    fn get(rng: &mut impl RandSource) -> Self {
        u64_from_u32s(rng.next(), rng.next())
    }
}

impl Rand for i8 {
    fn get(rng: &mut impl RandSource) -> Self {
        (rng.next() & 0xFF) as i8
    }
}

impl Rand for i16 {
    fn get(rng: &mut impl RandSource) -> Self {
        (rng.next() & 0xFFFF) as i16
    }
}

impl Rand for i32 {
    fn get(rng: &mut impl RandSource) -> Self {
        rng.next() as i32
    }
}

impl Rand for i64 {
    fn get(rng: &mut impl RandSource) -> Self {
        u64_from_u32s(rng.next(), rng.next()) as i64
    }
}

impl Rand for f32 {
    fn get(rng: &mut impl RandSource) -> Self {
        (rng.next() % (2 << 23)) as f32 / (2 << 23) as f32
    }
}

impl Rand for f64 {
    fn get(rng: &mut impl RandSource) -> Self {
        (rng.rand::<u64>() % (2u64 << 52)) as f64 / (2u64 << 52) as f64
    }
}

thread_local! {
    static THREAD_LCG: RefCell<Lcg> = RefCell::new(Lcg::from_time());
}

pub fn thread_lcg<R: Rand>() -> R {
    THREAD_LCG.with(|lcg| R::get(&mut *lcg.borrow_mut()))
}

pub struct ThreadLcg;

impl RandSource for ThreadLcg {
    fn next(&mut self) -> u32 {
        thread_lcg()
    }
}
