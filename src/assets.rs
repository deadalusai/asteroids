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
            explosion_assets: explosion::create_explosion_assets(),
            asteroid_assets: asteroid::create_asteroid_assets(),
            rocket_assets: player::create_roket_assets(),
            bullet_assets: bullet::create_bullet_assets(),
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
    pub explosion_assets: explosion::ExplosionAssets,
    pub asteroid_assets: asteroid::AsteroidAssets,
    pub rocket_assets: player::RocketAssets,
    pub bullet_assets: bullet::BulletAssets,
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