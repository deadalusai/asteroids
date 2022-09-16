use std::time::Duration;
use bevy::prelude::*;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(game_system);
    }
}

fn game_system(
    mut res: ResMut<GameController>,

) {
    
}

// Game Controller

static GAME_ASTEROID_RESPAWN_TIME_SECS: f32 = 3.0;
static GAME_PLAYER_RESPAWN_TIME_SECS: f32 = 1.5;
static GAME_PLAYER_INVULNERABLE_TIME_SECS: f32 = 1.5;

pub struct GameInit {
    /// The number of asteroids the game will try to maintain on screen
    pub target_asteroids: u32,
    pub player_lives: u32,
}

pub struct GameController {
    init: GameInit,
    player_lives_remaining: u32,
    player_points: u32,
    asteroid_spawn_timer: Timer,
    player_spawn_timer: Timer,
    player_invulnerable_timer: Timer,
}

impl GameController {
    pub fn new(init: GameInit) -> Self {
        Self {
            player_lives_remaining: init.player_lives,
            player_points: 0,
            init,
            asteroid_spawn_timer: Timer::new(Duration::from_secs_f32(GAME_ASTEROID_RESPAWN_TIME_SECS), false),
            player_spawn_timer: Timer::new(Duration::from_secs_f32(GAME_PLAYER_RESPAWN_TIME_SECS), false),
            player_invulnerable_timer: Timer::new(Duration::from_secs_f32(GAME_PLAYER_INVULNERABLE_TIME_SECS), false),
        }
    }
}

// // Player
// spawn_player_rocket(&mut commands, &assets.rocket_assets, &RocketSpawn::default());

// // Asteroids
// let mut rng = rand::thread_rng();
// let asteroids = [
//     (AsteroidSize::Large, 2),
//     (AsteroidSize::Medium, 3),
//     (AsteroidSize::Small, 5),
// ];
// for &(size, count) in &asteroids {
//     for _ in 0..count {
//         let (position, velocity) = random_asteroid_position_and_speed(&mut rng, &viewport);
//         let spawn = AsteroidSpawn { size, position, velocity };
//         spawn_asteroids(&mut commands, &assets.asteroid_assets, &[spawn]);
//     }
// }

// static ASTEROID_MAX_SPEED: f32 = 350.0;
// static ASTEROID_MIN_SPEED: f32 = 80.0;

// fn random_asteroid_position_and_speed(rng: &mut rand::rngs::ThreadRng, viewport: &Viewport) -> (Vec2, Vec2) {
//     // Generate a random asteroid
//     let position = rng.random_unit_vec2() * Vec2::new(viewport.width, viewport.height) / 2.0;
//     let velocity = ASTEROID_MIN_SPEED + rng.random_unit_vec2() * (ASTEROID_MAX_SPEED - ASTEROID_MIN_SPEED);
//     (position, velocity)
// }
