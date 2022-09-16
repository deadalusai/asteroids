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
use util::*;

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
        .add_startup_system(
            startup_system
                .after(StartupSystemLabel::LoadGameAssets)
        )
        .run();
}

fn startup_system(
    mut commands: Commands,
    viewport: Res<Viewport>,
    assets: Res<GameAssets>
) {
    // Camera
    commands.spawn_bundle(Camera2dBundle::default());

    // Player
    spawn_player_rocket(&mut commands, &assets.rocket_assets, &RocketSpawn::default());

    // Asteroids
    let mut rng = rand::thread_rng();
    let asteroids = [
        (AsteroidSize::Large, 2),
        (AsteroidSize::Medium, 3),
        (AsteroidSize::Small, 5),
    ];
    for &(size, count) in &asteroids {
        for _ in 0..count {
            let (position, velocity) = random_asteroid_position_and_speed(&mut rng, &viewport);
            let spawn = AsteroidSpawn { size, position, velocity };
            spawn_asteroids(&mut commands, &assets.asteroid_assets, &[spawn]);
        }
    }
}

static ASTEROID_MAX_SPEED: f32 = 350.0;
static ASTEROID_MIN_SPEED: f32 = 80.0;

fn random_asteroid_position_and_speed(rng: &mut rand::rngs::ThreadRng, viewport: &Viewport) -> (Vec2, Vec2) {
    // Generate a random asteroid
    let position = rng.random_unit_vec2() * Vec2::new(viewport.width, viewport.height) / 2.0;
    let velocity = ASTEROID_MIN_SPEED + rng.random_unit_vec2() * (ASTEROID_MAX_SPEED - ASTEROID_MIN_SPEED);
    (position, velocity)
}

// fn global_keyboard_event_system(
//     kb: Res<Input<KeyCode>>,
//     mut player_spawn_events: EventWriter<PlayerRocketSpawnEvent>,
//     mut asteroid_spawn_events: EventWriter<SpawnAsteroidEvent>,
// ) {
//     // DEBUG: Spawn another player rocket
//     if kb.just_released(KeyCode::Numpad0) {
//         player_spawn_events.send(PlayerRocketSpawnEvent);
//     }
//     // DEBUG: Spawn another asteroid
//     if kb.just_released(KeyCode::Numpad1) {
//         asteroid_spawn_events.send(SpawnAsteroidEvent(AsteroidSize::Small, None));
//     }
//     if kb.just_released(KeyCode::Numpad2) {
//         asteroid_spawn_events.send(SpawnAsteroidEvent(AsteroidSize::Medium, None));
//     }
//     if kb.just_released(KeyCode::Numpad3) {
//         asteroid_spawn_events.send(SpawnAsteroidEvent(AsteroidSize::Large, None));
//     }
// }
