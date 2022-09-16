use bevy::prelude::*;

use crate::StartupSystemLabel;
use crate::explosion;
use crate::asteroid;
use crate::player;
use crate::bullet;

pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        // Assets
        app.insert_resource(GameAssets {
            explosion: explosion::create_explosion_assets(),
            asteroid: asteroid::create_asteroid_assets(),
            rocket: player::create_roket_assets(),
            bullet: bullet::create_bullet_assets(),
        });

        // Viewport
        app.insert_resource(Viewport { width: 0., height: 0. });
        app.add_startup_system(
            viewport_update_system
                .label(StartupSystemLabel::LoadGameAssets)
        );
        app.add_system_to_stage(CoreStage::PreUpdate, viewport_update_system);
    }
}

pub struct GameAssets {
    pub explosion: explosion::ExplosionAssets,
    pub asteroid: asteroid::AsteroidAssets,
    pub rocket: player::RocketAssets,
    pub bullet: bullet::BulletAssets,
}

// Viewport resource

pub struct Viewport {
    pub width: f32,
    pub height: f32,
}

fn viewport_update_system(
    mut resize_events: EventReader<bevy::window::WindowResized>,
    mut viewport: ResMut<Viewport>
) {
    for ev in resize_events.iter() {
        viewport.width = ev.width;
        viewport.height = ev.height;
    }
}