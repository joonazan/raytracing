use std::f32::consts::PI;

use glam::Vec3;
use rand::{rngs::ThreadRng, thread_rng, Rng};

use crate::{
    refraction::{amount_reflected, IRON_REFRACTIVE_INDEX},
    Image, N,
};

pub struct Scene {
    spheres: Vec<Sphere>,
}

impl Image for Scene {
    fn render(&self, x: usize, y: usize) -> f32 {
        let mut rng = thread_rng();

        // The input x and y are the pixel's top left corner.
        // Generate a random position inside the pixel.
        let x = x as f32 + rng.gen::<f32>();
        let y = y as f32 + rng.gen::<f32>();
        let n = N as f32;
        let direction = Vec3::new(x / n - 0.5, 0.5 - y / n, 1.0).normalize();

        self.cast_ray(Vec3::new(0., 0., 0.), direction, &mut rng)
    }
}

impl Scene {
    pub fn new() -> Self {
        Self {
            spheres: vec![
                Sphere::new(Vec3::new(0., -8., 80.), 12.),
                Sphere::new(Vec3::new(13., -15., 65.), 5.),
                Sphere::new(Vec3::new(40., 16., 90.), 33.),
            ],
        }
    }
    fn cast_ray(&self, start: Vec3, direction: Vec3, rng: &mut ThreadRng) -> f32 {
        let mut earliest_hit_time = f32::INFINITY;
        let mut hit_object: Option<&dyn Object> = None;

        for s in self.spheres.iter() {
            if let Some(t) = s.hit_time(start, direction) {
                if t < earliest_hit_time {
                    earliest_hit_time = t;
                    hit_object = Some(s);
                }
            }
        }

        let floor = Floor;
        if let Some(t) = floor.hit_time(start, direction) {
            if t < earliest_hit_time {
                earliest_hit_time = t;
                hit_object = Some(&floor);
            }
        }
        if let Some(object) = hit_object {
            let hit_position = start + direction * earliest_hit_time;
            object.emitted_light(self, direction, hit_position, rng)
        } else {
            direction.y + 0.2
        }
    }
}

trait Object {
    fn hit_time(&self, start: Vec3, direction: Vec3) -> Option<f32>;
    fn emitted_light(
        &self,
        scene: &Scene,
        direction: Vec3,
        hit_position: Vec3,
        rng: &mut ThreadRng,
    ) -> f32;
}

struct Sphere {
    position: Vec3,
    radius_squared: f32,
}

impl Sphere {
    fn new(position: Vec3, radius: f32) -> Self {
        Self {
            position,
            radius_squared: radius * radius,
        }
    }
}

impl Object for Sphere {
    fn hit_time(&self, start: Vec3, direction: Vec3) -> Option<f32> {
        let closest_pos_time = direction.dot(self.position) - direction.dot(start);
        let closest_pos = start + closest_pos_time * direction;
        let closest_approach_squared = (closest_pos - self.position).length_squared();
        if closest_pos_time > 0. && closest_approach_squared < self.radius_squared {
            let to_actual_intersection_points =
                (self.radius_squared - closest_approach_squared).sqrt();
            let hit_position = closest_pos - to_actual_intersection_points * direction;

            Some(hit_position.dot(direction) - start.dot(direction))
        } else {
            None
        }
    }

    fn emitted_light(
        &self,
        scene: &Scene,
        direction: Vec3,
        hit_position: Vec3,
        rng: &mut ThreadRng,
    ) -> f32 {
        let normal = (hit_position - self.position).normalize();
        return amount_reflected(-direction.dot(normal), IRON_REFRACTIVE_INDEX)
            * scene.cast_ray(
                hit_position,
                direction - 2. * normal.dot(direction) * normal,
                rng,
            );
    }
}
struct Floor;

impl Object for Floor {
    fn hit_time(&self, start: Vec3, direction: Vec3) -> Option<f32> {
        let floor_height = -20.;
        let collision_time = (floor_height - start.y) / direction.y;
        if collision_time < 0. || direction.y > 0. {
            return None;
        }
        Some(collision_time)
    }

    fn emitted_light(
        &self,
        scene: &Scene,
        _direction: Vec3,
        hit_position: Vec3,
        rng: &mut ThreadRng,
    ) -> f32 {
        let scaled = hit_position * 10.;
        let color = (scaled.x as i32 ^ scaled.z as i32) as u8 as f32 / 400. + 0.2;

        color / PI * scene.cast_ray(hit_position, cosine_distributed_normal(rng), rng)
    }
}

fn cosine_distributed_normal(rng: &mut ThreadRng) -> Vec3 {
    loop {
        let x: f32 = rng.gen::<f32>() * 2. - 1.;
        let x2 = x * x;
        let z = rng.gen::<f32>() * 2. - 1.;
        let z2 = z * z;
        if x2 + z2 < 1. {
            let y2: f32 = 1. - x2 - z2;
            let y = y2.sqrt();
            return Vec3::new(x, y, z);
        }
    }
}
