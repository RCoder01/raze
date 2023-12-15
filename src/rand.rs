#![allow(unused)]
use std::{
    num::Wrapping,
    time::{SystemTime, UNIX_EPOCH},
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

    pub fn pseudo_rand_u32(&mut self) -> u32 {
        // using LCG params as used in java
        self.state = (self.state * Wrapping(0x5DEECE66Du64) + Wrapping(11)) % Wrapping(2u64 << 48);
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
