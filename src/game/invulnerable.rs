use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;

use crate::AppState;

// Plugins

pub struct InvulnerablePlugin;

impl Plugin for InvulnerablePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(
            update_invulnerability_system
                .in_set(OnUpdate(AppState::Game))
        );
    }
}

// Components

#[derive(Component)]
pub struct Invulnerable {
    timer: Timer,
}

impl Invulnerable {
    pub fn new(timer: Timer) -> Self {
        Self { timer }
    }
}

pub trait TestInvulnerable {
    fn is_invulnerable(&self) -> bool;
}

impl TestInvulnerable for Invulnerable {
    fn is_invulnerable(&self) -> bool {
        !self.timer.finished()
    }
}

impl TestInvulnerable for Option<&Invulnerable> {
    fn is_invulnerable(&self) -> bool {
        self.map(Invulnerable::is_invulnerable).unwrap_or(false)
    }
}

// Systems

pub fn update_invulnerability_system(
    time: Res<Time>,
    mut query: Query<(&mut Invulnerable, Option<&mut Stroke>)>
) {
    for (mut invulnerable, stroke) in query.iter_mut() {
        // Tick all potentially-invulnerable entities
        invulnerable.timer.tick(time.delta());

        // Optional animations
        if let Some(mut stroke) = stroke {
            let new_alpha = if invulnerable.is_invulnerable() {
                invulnerability_opacity_over_t(invulnerable.timer.elapsed_secs())
            }
            else {
                1.0
            };    
            stroke.color.set_a(new_alpha);
        }
    }
}

fn invulnerability_opacity_over_t(t_secs: f32) -> f32 {
    // animate the opacity between (0.4, 1.0) every 0.5 seconds, clamped rather than smooth
    let (min, max) = (0.1, 0.8);
    let frequency = 1.0 / 0.5;
    let scale = ((t_secs * std::f32::consts::TAU * frequency).cos() + 1.0) / 2.0;
    return scale.clamp(min, max);
}