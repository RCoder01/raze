use crate::{
    img::Color,
    math::{Ray, Vec3},
    rand::{self, RandSource, ThreadLcg},
};

fn uniform_relfection(random: &mut impl RandSource, normal: Vec3) -> Vec3 {
    let unit = rand::random_unit(random);
    if unit.dot(normal).is_sign_negative() {
        -unit
    } else {
        unit
    }
}

pub trait Reflector {
    fn reflect(&self, dir: Vec3, normal: Vec3) -> Vec3;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct UniformDiffuse;

impl Reflector for UniformDiffuse {
    fn reflect(&self, _dir: Vec3, normal: Vec3) -> Vec3 {
        uniform_relfection(&mut ThreadLcg, normal)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Lambertian;

impl Reflector for Lambertian {
    fn reflect(&self, _dir: Vec3, normal: Vec3) -> Vec3 {
        (uniform_relfection(&mut ThreadLcg, normal) + normal).normalize()
    }
}

pub trait Material {
    fn update_color(&self, outgoing: Color) -> Color;
    fn update_ray(&self, ray: Ray) -> Ray;
}

pub struct ColorMaterial<R: Reflector> {
    pub normal: Vec3,
    pub color: Color,
    pub reflector: R,
}

impl<R: Reflector> ColorMaterial<R> {
    pub const fn new(normal: Vec3, color: Color, reflector: R) -> Self {
        Self {
            normal,
            color,
            reflector,
        }
    }
}

impl<R: Reflector> Material for ColorMaterial<R> {
    fn update_color(&self, outgoing: Color) -> Color {
        outgoing.reflect_on(self.color)
    }

    fn update_ray(&self, mut ray: Ray) -> Ray {
        ray.dir = self.reflector.reflect(ray.dir, self.normal);
        ray
    }
}
