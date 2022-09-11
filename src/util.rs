use bevy::prelude::{Color, Vec2};
use bevy_prototype_lyon::prelude::DrawMode;

pub fn try_update_drawmode_alpha(draw_mode: &mut DrawMode, new_alpha: f32) {

    fn update_color_alpha(color: &mut Color, a: f32) {
        let alpha_ref = match color {
            Color::Rgba { ref mut alpha, .. } => alpha,
            Color::RgbaLinear { ref mut alpha, .. } => alpha,
            Color::Hsla { ref mut alpha, .. } => alpha,
        };
        *alpha_ref = a;
    }

    match draw_mode {
        DrawMode::Stroke(stroke) => update_color_alpha(&mut stroke.color, new_alpha),
        DrawMode::Fill(fill) => update_color_alpha(&mut fill.color, new_alpha),
        DrawMode::Outlined { fill_mode, outline_mode } => {
            update_color_alpha(&mut fill_mode.color, new_alpha);
            update_color_alpha(&mut outline_mode.color, new_alpha);
        },
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
