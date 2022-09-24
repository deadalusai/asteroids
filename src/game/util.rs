use bevy::prelude::Vec2;
use bevy::utils::HashSet;
use bevy_prototype_lyon::prelude::DrawMode;

pub fn update_drawmode_alpha(draw_mode: &mut DrawMode, new_alpha: f32) {
    match draw_mode {
        DrawMode::Stroke(stroke) => {
            stroke.color.set_a(new_alpha);
        },
        DrawMode::Fill(fill) => {
            fill.color.set_a(new_alpha);
        },
        DrawMode::Outlined { fill_mode, outline_mode } => {
            fill_mode.color.set_a(new_alpha);
            outline_mode.color.set_a(new_alpha);
        },
    }
}

// Rng

pub trait RngUtil {
    fn random_unit_vec2(&mut self) -> Vec2;
    fn random_f32(&mut self) -> f32;
    fn random_choice<'a, T>(&mut self, slice: &'a [T]) -> Option<&'a T>;
}

impl RngUtil for rand::rngs::ThreadRng {
    fn random_unit_vec2(&mut self) -> Vec2 {
        let x = self.random_f32() * 2.0 - 1.0;
        let y = self.random_f32() * 2.0 - 1.0;
        Vec2::new(x, y).normalize()
    }
    
    fn random_f32(&mut self) -> f32 {
        use rand::Rng;
        self.gen()
    }
    
    fn random_choice<'a, T>(&mut self, slice: &'a [T]) -> Option<&'a T> {
        use rand::seq::SliceRandom;
        slice.choose(self)
    }
}

// Iterable

pub fn distinct_by<T, F, V>(iterator: impl Iterator<Item=T>, selector: F) -> impl Iterator<Item=T>
    where V: Eq + std::hash::Hash,
          F: Fn(&T) -> V
{
    let mut seen = HashSet::new();
    iterator.filter(move |v| seen.insert(selector(v)))
}

// Lines and ray utilities

pub struct Line {
    origin: Vec2,
    normal: Vec2,
}

impl Line {
    pub fn from_origin_and_normal(origin: Vec2, normal: Vec2) -> Self {
        Self { origin, normal }
    }

    /// Tests for an intersection between the given ray and this line.
    /// Returns the distance along the ray at which the intersection occurs.
    pub fn try_intersect_line(&self, ray: &Ray2) -> Option<f32> {
        // intersection of ray with a line (or plane, with 3d vectors) at point `t`
        //  t = ((line_origin - ray_origin) . line_normal) / (ray_direction . line_normal)
        let denominator = ray.direction.dot(self.normal);
        // When the line and ray are nearing parallel the denominator approaches zero.
        if denominator.abs() < 1.0e-6 {
            return None;
        }
        let numerator = (self.origin - ray.origin).dot(self.normal);
        let t = numerator / denominator;
        // A negative `t` indicates the line is behind the ray origin
        if t < 0.0 {
            return None;
        }
        Some(t)
    }
}

pub struct Ray2 {
    origin: Vec2,
    direction: Vec2,
}

impl Ray2 {
    pub fn from_origin_and_direction(origin: Vec2, direction: Vec2) -> Self {
        Self { origin, direction: direction.normalize() }
    }

    pub fn point_at_t(&self, t: f32) -> Vec2 {
        self.origin + (self.direction * t)
    }
}