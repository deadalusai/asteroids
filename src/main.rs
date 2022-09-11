mod movable;
mod torus;
mod player;
mod bullet;
mod asteroid;

use bevy::prelude::*;
use movable::*;
use torus::*;
use player::*;
use asteroid::*;
use bullet::*;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.04, 0.04, 0.04)))
        .insert_resource(WindowDescriptor {
            title: "Asteroids".into(),
            width: 1600.,
            height: 1200.,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        // .add_plugin(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
        // .add_plugin(bevy::diagnostic::LogDiagnosticsPlugin::default())
        .add_plugin(PlayerPlugin)
        .add_plugin(BulletPlugin)
        .add_plugin(AsteroidPlugin)
        .add_plugin(TorusConstraintPlugin)
        .add_plugin(MovablePlugin)
        .add_startup_system(startup_system)
        .run();
}

fn startup_system(
    mut commands: Commands,
    mut player_spawn_events: EventWriter<SpawnPlayerRocketEvent>,
    mut spawn_events: EventWriter<SpawnAsteroidEvent>,
) {
    // Camera
    commands.spawn_bundle(Camera2dBundle::default());

    // Player
    player_spawn_events.send(SpawnPlayerRocketEvent);

    // Asteroids
    let asteroids = [
        (AsteroidSize::Large, 2),
        (AsteroidSize::Medium, 3),
        (AsteroidSize::Small, 5),
    ];
    for &(size, count) in &asteroids {
        for _ in 0..count {
            spawn_events.send(SpawnAsteroidEvent(size));
        }
    }
}
