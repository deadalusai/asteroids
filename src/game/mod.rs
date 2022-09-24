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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemLabel)]
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
    fn build(&mut self, group: &mut bevy::app::PluginGroupBuilder) {
        group.add(collidable::CollidablePlugin);
        group.add(movable::MovablePlugin);
        group.add(hit::HitPlugin);
        group.add(invulnerable::InvulnerablePlugin);
        group.add(assets::AssetsPlugin);
        group.add(player::PlayerPlugin);
        group.add(alien::AlienPlugin);
        group.add(bullet::BulletPlugin);
        group.add(asteroid::AsteroidPlugin);
        group.add(explosion::ExplosionPlugin);
        group.add(hud::HeadsUpDisplayPlugin);
        group.add(manager::GameManagerPlugin);
    }
}