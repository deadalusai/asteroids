use std::time::Duration;
use bevy::prelude::*;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(game_startup_system);
        app.add_system(game_system);
    }
}

fn game_startup_system() {

}

fn game_system() {
    
}

// Game Controller

static GAME_ASTEROID_RESPAWN_TIME_SECS: f32 = 3.0;
static GAME_PLAYER_RESPAWN_TIME_SECS: f32 = 1.5;
static GAME_PLAYER_INVULNERABLE_TIME_SECS: f32 = 1.5;

struct GameInit {
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
    fn new(init: GameInit) -> Self {
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
