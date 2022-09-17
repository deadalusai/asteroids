#![feature(drain_filter)]

mod svg;
mod util;
mod movable;
mod collidable;
mod hit;
mod player;
mod bullet;
mod asteroid;
mod explosion;
mod hud;
mod game;
mod assets;

use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use collidable::*;
use movable::*;
use assets::*;
use hud::*;
use hit::*;
use player::*;
use bullet::*;
use asteroid::*;
use explosion::*;
use game::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemLabel)]
pub enum StartupSystemLabel {
    LoadGameAssets
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemLabel)]
pub enum SystemLabel {
    Input,
    Movement,
    Collision,
}

fn main() {
    let title = "Asteroids".into();
    let (width, height) = (1600., 1200.);
    App::new()
        // bevy
        .insert_resource(ClearColor(Color::rgb(0.04, 0.04, 0.04)))
        .insert_resource(WindowDescriptor {
            title,
            width,
            height,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        // bevy_prototype_lyon
        .insert_resource(Msaa { samples: 4 })
        .add_plugin(ShapePlugin)
        // Game
        .add_plugin(CollidablePlugin)
        .add_plugin(MovablePlugin)
        .add_plugin(HitEventsPlugin)
        .add_plugin(AssetsPlugin)
        .add_plugin(PlayerPlugin)
        .add_plugin(BulletPlugin)
        .add_plugin(AsteroidPlugin)
        .add_plugin(ExplosionPlugin)
        .add_plugin(HeadsUpDisplayPlugin)
        .add_plugin(GamePlugin)
        .add_startup_system(
            startup_system
                .after(StartupSystemLabel::LoadGameAssets))
        .add_system(global_keyboard_event_system)
        .run();
}

// Startup

const FIXED_VIEWPORT_WIDTH: f32 = 256.0;

fn startup_system(mut commands: Commands) {
    // Spawn a camera
    // NOTE: Our graphics are small! Tune the projection to keep the size of the "world" known
    // despite window scale
    commands.spawn_bundle(Camera2dBundle {
        transform: Transform::from_xyz(0.0, 0.0, 999.9),
        projection: bevy::render::camera::OrthographicProjection {
            far: 1000.0,
            depth_calculation: bevy::render::camera::DepthCalculation::ZDifference,
            scaling_mode: bevy::render::camera::ScalingMode::FixedHorizontal(FIXED_VIEWPORT_WIDTH),
            ..default()
        },
        ..default()
    });
}

// Keyboard handlers

fn global_keyboard_event_system(
    kb: Res<Input<KeyCode>>,
    mut game: ResMut<Game>
) {
    // DEBUG: Reset the game state
    if kb.just_released(KeyCode::R) {
        game.reset();
    }
}