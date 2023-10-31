use std::ops::Deref;

use crate::math::Vec3;

#[derive(Debug, PartialEq)]
pub(crate) struct Collision {
    pub(crate) position: Vec3,
    pub(crate) normal: Vec3,
    // color, scattering, ...
}

impl Collision {
    pub(crate) fn new(position: Vec3, normal: Vec3) -> Self {
        Self { position, normal }
    }

    // fn cmp(&self, other: &Self) -> Ordering {
    //     self.position
    //         .squared_magnitude()
    //         .total_cmp(&other.position.squared_magnitude())
    // }
}

pub(crate) trait Shape {
    fn ray_intersection(&self, ray_start: Vec3, ray_dir: Vec3) -> Option<Collision>;
}

impl<T> Shape for T
where
    T: Deref,
    T::Target: Shape,
{
    fn ray_intersection(&self, ray_start: Vec3, ray_dir: Vec3) -> Option<Collision> {
        (**self).ray_intersection(ray_start, ray_dir)
    }
}

impl<T> Shape for [T]
where
    T: Shape,
{
    fn ray_intersection(&self, ray_start: Vec3, ray_dir: Vec3) -> Option<Collision> {
        self.iter()
            .filter_map(|shape| shape.ray_intersection(ray_start, ray_dir))
            .min_by(|c1, c2| {
                (c1.position - ray_start)
                    .squared_magnitude()
                    .total_cmp(&(c2.position - ray_start).squared_magnitude())
            })
    }
}

#[derive(Debug, Clone)]
pub(crate) struct TriangleMesh {
    pub(crate) vertices: Vec<Vec3>,
    pub(crate) triangles: Vec<[u16; 3]>,
    pub(crate) normals: Vec<Vec3>,
}

impl TriangleMesh {
    pub(crate) fn new(vertices: Vec<Vec3>, triangles: Vec<[u16; 3]>) -> Self {
        let normals: Vec<_> = triangles
            .iter()
            .copied()
            .map(|[a, b, c]| {
                (vertices[b as usize] - vertices[a as usize])
                    .cross(vertices[c as usize] - vertices[b as usize])
                    .normalize()
            })
            .collect();
        Self {
            vertices,
            triangles,
            normals,
        }
    }
}

impl Shape for TriangleMesh {
    fn ray_intersection(&self, ray_start: Vec3, ray_dir: Vec3) -> Option<Collision> {
        let nearest_collision = self
            .triangles
            .iter()
            .copied()
            .zip(self.normals.iter().copied())
            .enumerate()
            .filter_map(|(i, ([a, b, c], normal))| {
                if ray_dir.dot(normal) > -1e-5 {
                    // ray and normal are roughly the same direction
                    return None;
                }

                let vertices = [
                    self.vertices[a as usize] - ray_start,
                    self.vertices[b as usize] - ray_start,
                    self.vertices[c as usize] - ray_start,
                ];
                if vertices.iter().all(|vertex| vertex.dot(ray_dir) < -1e-5) {
                    return None;
                }
                // println!("{}, {:?}", i, ray_dir);

                let normals = [
                    vertices[1].cross(vertices[0]),
                    vertices[2].cross(vertices[1]),
                    vertices[0].cross(vertices[2]),
                ];

                let intersects = normals.iter().all(|normal| normal.dot(ray_dir) > -1e-5);
                if !intersects {
                    return None;
                }
                let distance = (vertices[0]).project_onto(normal);
                let intersect_point =
                    (ray_dir.dot(distance) / distance.squared_magnitude()).recip() * ray_dir;
                Some((i, intersect_point))
            })
            .min_by(|(_, v1), (_, v2)| v1.squared_magnitude().total_cmp(&v2.squared_magnitude()));
        nearest_collision
            .map(|(i, intersect)| Collision::new(ray_start + intersect, self.normals[i]))
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Sphere {
    pub(crate) center: Vec3,
    pub(crate) radius: f32,
}

impl Sphere {
    pub(crate) fn new(center: Vec3, radius: f32) -> Self {
        Self { center, radius }
    }
}

impl Shape for Sphere {
    fn ray_intersection(&self, ray_start: Vec3, ray_dir: Vec3) -> Option<Collision> {
        let relative_center = self.center - ray_start;
        let cx = relative_center.dot(ray_dir);
        let cc = relative_center.dot(relative_center);
        let xx = ray_dir.dot(ray_dir);
        let rr = self.radius * self.radius;
        let l = cx - (cx * cx - xx * (cc - rr)).sqrt();
        if l.is_nan() {
            None
        } else {
            let ray = ray_dir * l;
            let point = ray + ray_start;
            let normal = (relative_center - ray).normalize();
            Some(Collision::new(point, normal))
        }
    }
}
