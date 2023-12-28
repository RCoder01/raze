use crate::{
    img::Color,
    math::{Ray, Vec3},
    rand::{Reflector, ThreadLcg},
};

pub trait Material {
    fn update_color(&self, outgoing: Color) -> Color;
    fn update_ray(&self, ray: Ray) -> Ray;
}

pub struct DiffuseColorMaterial {
    pub normal: Vec3,
    pub color: Color,
}

impl DiffuseColorMaterial {
    pub const fn new(normal: Vec3, color: Color) -> Self {
        Self { normal, color }
    }
}

impl Material for DiffuseColorMaterial {
    fn update_color(&self, outgoing: Color) -> Color {
        outgoing.reflect_on(self.color)
    }

    fn update_ray(&self, mut ray: Ray) -> Ray {
        ray.dir = Reflector::new(ThreadLcg).random_diffuse(self.normal);
        ray
    }
}
