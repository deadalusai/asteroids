use bevy::prelude::*;

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
    }
}

pub struct GameAssets {
    pub explosion: explosion::ExplosionAssets,
    pub asteroid: asteroid::AsteroidAssets,
    pub rocket: player::RocketAssets,
    pub bullet: bullet::BulletAssets,
}