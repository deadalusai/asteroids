use bevy::prelude::*;

use super::explosion;
use super::asteroid;
use super::player;
use super::bullet;
use super::alien;

pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        // Assets
        app.insert_resource(GameAssets {
            explosion: explosion::create_explosion_assets(),
            asteroid: asteroid::create_asteroid_assets(),
            rocket: player::create_roket_assets(),
            alien: alien::create_alien_assets(),
            bullet: bullet::create_bullet_assets(),
        });
    }
}

#[derive(Resource)]
pub struct GameAssets {
    pub explosion: explosion::ExplosionAssets,
    pub asteroid: asteroid::AsteroidAssets,
    pub rocket: player::RocketAssets,
    pub alien: alien::AlienAssets,
    pub bullet: bullet::BulletAssets,
}