pub mod svg;
pub mod util;
pub mod movable;
pub mod collidable;
pub mod hit;
pub mod player;
pub mod alien;
pub mod invulnerable;
pub mod bullet;
pub mod asteroid;
pub mod explosion;
pub mod hud;
pub mod manager;
pub mod assets;

use bevy::prelude::*;
use bevy::app::PluginGroupBuilder;

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FrameStage {
    Start,
    Input,
    Movement,
    Collision,
    CollisionEffect
}

// Plugins

pub struct GamePluginGroup;

impl PluginGroup for GamePluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(collidable::CollidablePlugin)
            .add(movable::MovablePlugin)
            .add(hit::HitPlugin)
            .add(invulnerable::InvulnerablePlugin)
            .add(assets::AssetsPlugin)
            .add(player::PlayerPlugin)
            .add(alien::AlienPlugin)
            .add(bullet::BulletPlugin)
            .add(asteroid::AsteroidPlugin)
            .add(explosion::ExplosionPlugin)
            .add(hud::HeadsUpDisplayPlugin)
            .add(manager::GameManagerPlugin)
    }
}