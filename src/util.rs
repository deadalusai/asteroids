use bevy::prelude::{Color, Vec2};
use bevy_prototype_lyon::prelude::{DrawMode, StrokeMode};
use bevy_prototype_lyon::prelude::tess::geom::euclid::approxeq::ApproxEq;

pub fn try_update_stroke_alpha(draw_mode: &mut DrawMode, new_alpha: f32) {
    let stroke = match draw_mode {
        DrawMode::Stroke(stroke) => *stroke,
        _ => panic!("Called read_stroke_mode on non-stroke draw mode"),
    };
    if !stroke.color.a().approx_eq(&new_alpha) {
        // Update the opacity of the stroke
        let [r, g, b, _] = stroke.color.as_rgba_f32();
        let color = Color::rgba(r, g, b, new_alpha);
        *draw_mode = DrawMode::Stroke(StrokeMode { color, ..stroke })
    }
}

pub trait RngUtil {
    fn random_unit_vec2(&mut self) -> Vec2;
    fn random_f32(&mut self) -> f32;
    fn random_choice<'a, T>(&mut self, slice: &'a [T]) -> Option<&'a T>;
}

impl RngUtil for rand::rngs::ThreadRng {
    fn random_unit_vec2(&mut self) -> Vec2 {
        use rand::Rng;
        let x = self.gen::<f32>() * 2.0 - 1.0;
        let y = self.gen::<f32>() * 2.0 - 1.0;
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
