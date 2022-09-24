use bevy::prelude::*;
use bevy::render::render_resource::encase::rts_array::Length;
use rand::thread_rng;
use crate::AppState;
use super::FrameStage;
use super::assets::GameAssets;
use super::alien::{AlienSpawn, AlienUfoDestroyedEvent, spawn_alien_ufo};
use super::player::{PlayerRocketDestroyedEvent, RocketSpawn, spawn_player_rocket};
use super::asteroid::{Asteroid, AsteroidDestroyedEvent, AsteroidSize, AsteroidSpawn, AsteroidShapeId, spawn_asteroid};
use super::util::*;

pub struct GameManagerPlugin;

impl Plugin for GameManagerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WorldBoundaries::default());
        app.add_system_set(
            SystemSet::on_enter(AppState::Game)
                .with_system(game_setup_system)
        );
        app.add_system_set(
            SystemSet::on_exit(AppState::Game)
                .with_system(game_teardown_system)
        );
        app.add_system_set(
            SystemSet::on_update(AppState::Game)
                .with_system(
                    world_boundaries_update_system
                        .label(FrameStage::Start)
                )
                .with_system(
                    game_effects_system
                        .label(FrameStage::Start)
                        .after(world_boundaries_update_system)
                )
                .with_system(
                    game_events_system
                )
                .with_system(
                    game_update_system
                        .after(game_events_system)
                )
                .with_system(
                    game_keyboard_system
                )
        );
    }
}

const ALIEN_SPAWN_MIN_SECS: f32 = 5.0;
const ALIEN_SPAWN_MAX_SECS: f32 = 60.0;

fn game_setup_system(mut commands: Commands) {
    let mut rng = thread_rng();
    let alien_spawn_secs = ALIEN_SPAWN_MIN_SECS + rng.random_f32() * (ALIEN_SPAWN_MAX_SECS - ALIEN_SPAWN_MIN_SECS);
    let game_init = GameInit {
        asteroid_count: 8,
        player_lives: 3,
        alien_spawn_secs,
    };
    commands.insert_resource(GameManager::new(game_init));
}

fn game_teardown_system(mut commands: Commands) {
    commands.remove_resource::<GameManager>();
}

// World boundary information

#[derive(Default)]
pub struct WorldBoundaries {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
}

fn world_boundaries_update_system(
    mut world_boundaries: ResMut<WorldBoundaries>,
    projection: Query<&bevy::render::camera::OrthographicProjection>
) {
    let projection = projection.get_single().unwrap();
    world_boundaries.left = projection.left;
    world_boundaries.right = projection.right;
    world_boundaries.top = projection.top;
    world_boundaries.bottom = projection.bottom;
}

// Game Controller

static GAME_PLAYER_RESPAWN_TIME_SECS: f32 = 1.5;
static GAME_ASTEROID_SPAWN_TIME_SECS: f32 = 5.0;

#[derive(Clone)]
pub struct GameInit {
    /// The number of asteroids the game will try to maintain on screen
    pub asteroid_count: u32,
    pub player_lives: u32,
    pub alien_spawn_secs: f32,
}

#[derive(PartialEq, Eq, Debug)]
pub enum PlayerState {
    FirstSpawn,
    Respawning,
    Ready,
    Destroyed,
}

#[derive(PartialEq, Eq, Debug)]
pub enum AlienState {
    Spawning,
    Ready,
}

enum AsteroidSpawnInstruction {
    Anywhere,
    OffScreen,
    FromDestroyedAsteroid(AsteroidDestroyedEvent),
    AtPosition(Vec2),
}

pub struct ScheduledAsteroidSpawn {
    spawn_timer: Timer,
    instruction: AsteroidSpawnInstruction
}

pub struct GameManager {
    pub player_lives_remaining: u32,
    pub player_points: u32,
    pub debug_asteroid_count_on_screen: u32,
    pub scheduled_asteroid_spawns: Vec<ScheduledAsteroidSpawn>,
    pub player_state: PlayerState,
    pub alien_state: AlienState,
    player_spawn_timer: Timer,
    alien_spawn_timer: Timer,
    init: GameInit,
}

impl GameManager {
    pub fn new(init: GameInit) -> Self {
        let asteroid_count = init.asteroid_count;
        let mut game = Self {
            player_lives_remaining: init.player_lives,
            player_points: 0,
            player_state: PlayerState::FirstSpawn,
            alien_state: AlienState::Spawning,
            player_spawn_timer: Timer::from_seconds(0.0, false),
            alien_spawn_timer: Timer::from_seconds(0.0, false),
            scheduled_asteroid_spawns: Vec::new(),
            debug_asteroid_count_on_screen: 0,
            init,
        };
        game.schedule_alien_ufo_to_spawn();
        for _ in 0..asteroid_count {
            game.schedule_asteroid_to_spawn(0.0, AsteroidSpawnInstruction::Anywhere);
        }
        game
    }

    fn on_rocket_destroyed(&mut self) {
        if self.player_state != PlayerState::Ready {
            return;
        }
        self.player_state = match self.player_lives_remaining {
            0 => PlayerState::Destroyed,
            _ => PlayerState::Respawning,
        };
        if self.player_state == PlayerState::Respawning {
            self.player_lives_remaining -= 1;
            self.player_spawn_timer = Timer::from_seconds(GAME_PLAYER_RESPAWN_TIME_SECS, false);
        }
    }

    fn on_asteroid_destroyed(&mut self, event: AsteroidDestroyedEvent) {
        self.player_points += get_points_for_asteroid(event.size);
        // Break apart large asteroids
        if event.size == AsteroidSize::Medium || event.size == AsteroidSize::Large {
            self.schedule_asteroid_to_spawn(0.0, AsteroidSpawnInstruction::FromDestroyedAsteroid(event));
        }
    }

    fn on_alien_ufo_destroyed(&mut self) {
        self.player_points += get_points_for_alien_ufo();
        self.schedule_alien_ufo_to_spawn();
    }

    fn on_asteroid_count_update(&mut self, current_asteroid_count: u32) {
        self.debug_asteroid_count_on_screen = current_asteroid_count;
        // Schedule asteroids to "refill" the configured number of asteroids
        let pending_asteroid_count = self.scheduled_asteroid_spawns.length() as i32;
        let missing_asteroid_count = self.init.asteroid_count as i32 - current_asteroid_count as i32 - pending_asteroid_count;
        for _ in 0..missing_asteroid_count {
            self.schedule_asteroid_to_spawn(GAME_ASTEROID_SPAWN_TIME_SECS, AsteroidSpawnInstruction::OffScreen);
        }
    }

    fn schedule_asteroid_to_spawn(&mut self, time_secs: f32, instruction: AsteroidSpawnInstruction) {
        self.scheduled_asteroid_spawns.push(ScheduledAsteroidSpawn {
            spawn_timer: Timer::from_seconds(time_secs, false),
            instruction
        });
    }

    fn schedule_alien_ufo_to_spawn(&mut self) {
        self.alien_state = AlienState::Spawning;
        self.alien_spawn_timer = Timer::from_seconds(self.init.alien_spawn_secs, false);
    }

    fn tick(&mut self, delta: std::time::Duration) {
        self.player_spawn_timer.tick(delta);
        self.alien_spawn_timer.tick(delta);
        for s in self.scheduled_asteroid_spawns.iter_mut() {
            s.spawn_timer.tick(delta);
        }
    }

    fn should_spawn_player(&self) -> bool {
        let should_spawn =
            self.player_state == PlayerState::FirstSpawn ||
            (self.player_state == PlayerState::Respawning && self.player_spawn_timer.finished());
            
        return should_spawn;
    }

    fn on_rocket_spawned(&mut self) {
        self.player_state = PlayerState::Ready;
    }

    fn should_spawn_alien_ufo(&self) -> bool {
        return self.alien_state == AlienState::Spawning && self.alien_spawn_timer.finished();
    }

    fn on_alien_ufo_spawned(&mut self) {
        self.alien_state = AlienState::Ready;
    }
}

fn get_points_for_asteroid(size: AsteroidSize) -> u32 {
    match size {
        AsteroidSize::Small => 10,
        AsteroidSize::Medium => 7,
        AsteroidSize::Large => 5,
    }
}

fn get_points_for_alien_ufo() -> u32 {
    15
}

// Systems

// Listen for events and update the game state
fn game_events_system(
    mut game: ResMut<GameManager>,
    mut rocket_destructions: EventReader<PlayerRocketDestroyedEvent>,
    mut asteroid_destructions: EventReader<AsteroidDestroyedEvent>,
    mut alien_destructions: EventReader<AlienUfoDestroyedEvent>
) {
    if rocket_destructions.iter().next().is_some() {
        game.on_rocket_destroyed();
    }

    for ev in asteroid_destructions.iter() {
        game.on_asteroid_destroyed(ev.clone());
    }

    for _ in alien_destructions.iter() {
        game.on_alien_ufo_destroyed();
    }
}

fn game_update_system(
    mut game: ResMut<GameManager>,
    asteroids: Query<&Asteroid>
) {
    let asteroid_count = asteroids.iter().count();
    game.on_asteroid_count_update(asteroid_count as u32);
}

// Apply game effects to the world
fn game_effects_system(
    mut commands: Commands,
    mut game: ResMut<GameManager>,
    mut app_state: ResMut<State<AppState>>,
    world_boundaries: Res<WorldBoundaries>,
    time: Res<Time>,
    assets: Res<GameAssets>,
) {
    let mut rng = thread_rng();

    game.tick(time.delta());
    
    if game.should_spawn_player() {
        game.on_rocket_spawned();
        spawn_player_rocket(&mut commands, &assets.rocket, RocketSpawn::default());
    }
    
    if game.should_spawn_alien_ufo() {
        game.on_alien_ufo_spawned();
        handle_alien_ufo_spawn(&mut commands, &mut rng, &world_boundaries, &assets);
    }

    for spawn in game.scheduled_asteroid_spawns.drain_filter(|s| s.spawn_timer.finished()) {
        handle_asteroid_spawn(&mut commands, &mut rng, &world_boundaries, &assets, spawn);
    }

    // Game over?
    if game.player_state == PlayerState::Destroyed {
        commands.insert_resource(crate::game_over_screen::GameResults {
            score: game.player_points,
        });
        app_state.push(AppState::GameOver).unwrap();
        return;
    }
}

const ALIEN_UFO_SPEED: f32 = 50.0;

fn handle_alien_ufo_spawn(
    commands: &mut Commands,
    rng: &mut rand::rngs::ThreadRng,
    world_boundaries: &WorldBoundaries,
    assets: &GameAssets
) {

    // Pick a position off-screen
    let from_left = rng.random_bool();
    let x = if from_left { world_boundaries.left - 10.0 } else { world_boundaries.right + 10.0 };
    let y = (rng.random_f32() * 2. - 1.) * (world_boundaries.top * 0.8);
    let x_speed = if from_left { ALIEN_UFO_SPEED } else { -ALIEN_UFO_SPEED };
    
    spawn_alien_ufo(commands, &assets.alien, AlienSpawn {
        position: Vec2::new(x, y),
        velocity: Vec2::new(x_speed, 0.),
    });
}

fn handle_asteroid_spawn(
    commands: &mut Commands,
    rng: &mut rand::rngs::ThreadRng,
    world_boundaries: &Res<WorldBoundaries>,
    assets: &Res<GameAssets>,
    sched: ScheduledAsteroidSpawn
) {
    match sched.instruction {
        AsteroidSpawnInstruction::Anywhere => {
            // Spawn on-screen asteroids
            let position = random_onscreen_position(rng, world_boundaries);
            let velocity = random_asteroid_velocity(rng);
            let rotation = random_asteroid_rotation(rng);
            let size = random_asteroid_size(rng);
            let shape = random_asteroid_shape(rng);
            let spawn = AsteroidSpawn { size, shape, position, velocity, rotation, invulnerable: None };
            spawn_asteroid(commands, &assets.asteroid, spawn);

        },
        AsteroidSpawnInstruction::OffScreen => {
            // Spawn off-screen asteroids
            let position = random_offscreen_position(rng, world_boundaries, 10.0);
            let velocity = random_asteroid_velocity(rng);
            let rotation = random_asteroid_rotation(rng);
            let size = random_asteroid_size(rng);
            let shape = random_asteroid_shape(rng);
            let spawn = AsteroidSpawn { size, shape, position, velocity, rotation, invulnerable: None };
            spawn_asteroid(commands, &assets.asteroid, spawn);
        },
        AsteroidSpawnInstruction::FromDestroyedAsteroid(ev) => {
            // Spawn child asteroids
            let [a, b] = random_chunk_asteroid_state(rng, ev.position, ev.velocity);
            let size = match ev.size {
                AsteroidSize::Small => unreachable!(),
                AsteroidSize::Medium => AsteroidSize::Small,
                AsteroidSize::Large => AsteroidSize::Medium,
            };
            let invulnerable = Some(Timer::from_seconds(CHUNK_ASTEROID_INVULNERABLE_SECS, false));
            spawn_asteroid(commands, &assets.asteroid, AsteroidSpawn { size, position: a.0, velocity: a.1, rotation: a.2, shape: a.3, invulnerable: invulnerable.clone() });
            spawn_asteroid(commands, &assets.asteroid, AsteroidSpawn { size, position: b.0, velocity: b.1, rotation: b.2, shape: b.3, invulnerable });
        },
        AsteroidSpawnInstruction::AtPosition(position) => {
            let velocity = Vec2::ZERO; // random_asteroid_velocity(rng);
            let rotation = 0.0;
            let size = random_asteroid_size(rng);
            let shape = random_asteroid_shape(rng);
            let spawn = AsteroidSpawn { size, shape, position, velocity, rotation, invulnerable: None };
            spawn_asteroid(commands, &assets.asteroid, spawn);
        },
    };
}

static CHILD_ASTEROID_SPAWN_DISTANCE: f32 = 2.0;
static CHILD_ASTEROID_MIN_ADD_SPEED: f32 = 5.0;
static CHILD_ASTEROID_MAX_ADD_SPEED: f32 = 15.0;
static CHUNK_ASTEROID_VELOCITY_REDUCTION: f32 = 0.8;
static CHUNK_ASTEROID_INVULNERABLE_SECS: f32 = 0.5;

pub fn random_chunk_asteroid_state(rng: &mut rand::rngs::ThreadRng, position: Vec2, velocity: Vec2) -> [(Vec2, Vec2, f32, AsteroidShapeId); 2] {

    // Generate some random position and velocity for these two asteroids
    let chunk_direction = rng.random_unit_vec2();
    let chunk_velocity = CHILD_ASTEROID_MIN_ADD_SPEED + rng.random_f32() * (CHILD_ASTEROID_MAX_ADD_SPEED - CHILD_ASTEROID_MIN_ADD_SPEED);

    let p1 = position + chunk_direction * CHILD_ASTEROID_SPAWN_DISTANCE;
    let p2 = position + -chunk_direction * CHILD_ASTEROID_SPAWN_DISTANCE;

    let v1 = velocity * CHUNK_ASTEROID_VELOCITY_REDUCTION + chunk_direction * chunk_velocity;
    let v2 = velocity * CHUNK_ASTEROID_VELOCITY_REDUCTION + -chunk_direction * chunk_velocity;
    
    let r1 = random_asteroid_rotation(rng);
    let r2 = random_asteroid_rotation(rng);

    let s1 = random_asteroid_shape(rng);
    let s2 = random_asteroid_shape(rng);
    
    [(p1, v1, r1, s1), (p2, v2, r2, s2)]
}

const ASTEROID_MAX_SPEED: f32 = 50.0;
const ASTEROID_MIN_SPEED: f32 = 5.0;
fn random_asteroid_velocity(rng: &mut rand::rngs::ThreadRng) -> Vec2 {
    ASTEROID_MIN_SPEED + rng.random_unit_vec2() * (ASTEROID_MAX_SPEED - ASTEROID_MIN_SPEED)
}

const ASTEROID_MAX_SPIN_RATE: f32 = 0.4;
const ASTEROID_MIN_SPIN_RATE: f32 = 0.05;
fn random_asteroid_rotation(rng: &mut rand::rngs::ThreadRng) -> f32 {
    ASTEROID_MIN_SPIN_RATE + rng.random_f32() * (ASTEROID_MAX_SPIN_RATE - ASTEROID_MIN_SPIN_RATE)
}

fn get_world_boundary_lines(world_boundaries: &WorldBoundaries) -> [Line; 4] {
    let left   = Vec2::new(world_boundaries.left, 0.0);
    let right  = Vec2::new(world_boundaries.right, 0.0);
    let top    = Vec2::new(0.0, world_boundaries.top);
    let bottom = Vec2::new(0.0, world_boundaries.bottom);
    [
        Line::from_origin_and_normal(left,   -left.normalize()),
        Line::from_origin_and_normal(right,  -right.normalize()),
        Line::from_origin_and_normal(top,    -top.normalize()),
        Line::from_origin_and_normal(bottom, -bottom.normalize()),
    ]
}

fn random_offscreen_position(rng: &mut rand::rngs::ThreadRng, world_boundaries: &WorldBoundaries, add_t: f32) -> Vec2 {
    use std::cmp::Ordering::*;
    // TODO: Pick a random position off the screen
    // Project this line until it intersects with one of the edges of the world_boundaries.
    let ray = Ray2::from_origin_and_direction(Vec2::ZERO, rng.random_unit_vec2());
    let t = get_world_boundary_lines(world_boundaries)
        .iter()
        .filter_map(|line| line.try_intersect_line(&ray))
        .min_by(|a, b| a.partial_cmp(b).unwrap_or(Equal))
        .unwrap();

    return ray.point_at_t(t + add_t);
}

fn random_onscreen_position(rng: &mut rand::rngs::ThreadRng, world_boundaries: &WorldBoundaries) -> Vec2 {
    rng.random_unit_vec2() * Vec2::new(world_boundaries.right, world_boundaries.top)
}

fn random_asteroid_size(rng: &mut rand::rngs::ThreadRng) -> AsteroidSize {
    *rng.random_choice(&AsteroidSize::VALUES).unwrap()
}

fn random_asteroid_shape(rng: &mut rand::rngs::ThreadRng) -> AsteroidShapeId {
    *rng.random_choice(&AsteroidShapeId::VALUES).unwrap()
}

// Keyboard handlers

fn game_keyboard_system(
    mut kb: ResMut<Input<KeyCode>>,
    mut game: ResMut<GameManager>,
    mut app_state: ResMut<State<AppState>>,
) {
    if kb.just_released(KeyCode::A) {
        game.schedule_asteroid_to_spawn(0.0, AsteroidSpawnInstruction::AtPosition(Vec2::new(0., 20.)));
    }

    if kb.clear_just_released(KeyCode::Escape) {
        app_state.push(AppState::Pause).unwrap();
    }
}