mod svg;
mod util;
mod movable;
mod hit;
mod player;
mod bullet;
mod asteroid;
mod explosion;
mod viewport;
mod hud;

use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use viewport::*;
use movable::*;
use hit::*;
use player::*;
use bullet::*;
use asteroid::*;
use explosion::*;

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
        // bevy_sepax2d (collision detection)
        .add_system_to_stage(CoreStage::PostUpdate, bevy_sepax2d::plugin::update_movable_system)
        .add_system_to_stage(CoreStage::PostUpdate, bevy_sepax2d::plugin::clear_correction_system)
        // Game
        .add_plugin(ViewportPlugin)
        .add_plugin(PlayerPlugin)
        .add_plugin(BulletPlugin)
        .add_plugin(AsteroidPlugin)
        .add_plugin(ExplosionPlugin)
        .add_plugin(MovablePlugin)
        .add_plugin(HitEventsPlugin)
        .add_startup_system(startup_system)
        .add_system(global_keyboard_event_system)
        .run();
}

fn startup_system(
    mut commands: Commands,
    mut player_spawn_events: EventWriter<SpawnPlayerRocketEvent>,
    mut asteroid_spawn_events: EventWriter<SpawnAsteroidEvent>
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
            asteroid_spawn_events.send(SpawnAsteroidEvent(size));
        }
    }
}

fn global_keyboard_event_system(
    kb: Res<Input<KeyCode>>,
    mut player_spawn_events: EventWriter<SpawnPlayerRocketEvent>,
    mut asteroid_spawn_events: EventWriter<SpawnAsteroidEvent>,
) {
    // DEBUG: Spawn another player rocket
    if kb.just_released(KeyCode::Numpad0) {
        player_spawn_events.send(SpawnPlayerRocketEvent);
    }
    // DEBUG: Spawn another asteroid
    if kb.just_released(KeyCode::Numpad1) {
        asteroid_spawn_events.send(SpawnAsteroidEvent(AsteroidSize::Small));
    }
    if kb.just_released(KeyCode::Numpad2) {
        asteroid_spawn_events.send(SpawnAsteroidEvent(AsteroidSize::Medium));
    }
    if kb.just_released(KeyCode::Numpad3) {
        asteroid_spawn_events.send(SpawnAsteroidEvent(AsteroidSize::Large));
    }
}
