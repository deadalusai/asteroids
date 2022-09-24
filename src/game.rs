use bevy::prelude::*;
use bevy::render::render_resource::encase::rts_array::Length;
use rand::thread_rng;
use crate::assets::GameAssets;
use crate::player::{PlayerRocketDestroyedEvent, RocketSpawn, spawn_player_rocket};
use crate::asteroid::{AsteroidDestroyedEvent, AsteroidSize, AsteroidSpawn, AsteroidShapeId, spawn_asteroid, Asteroid};
use crate::util::*;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Game::new(GameInit { asteroid_count: 8, player_lives: 3 }));
        app.insert_resource(WorldBoundaries::default());
        app.add_system(game_keyboard_event_system);
        app.add_system_to_stage(CoreStage::PreUpdate, world_boundaries_update_system);
        app.add_system_to_stage(CoreStage::PreUpdate, game_effects_system.after(world_boundaries_update_system));
        app.add_system_to_stage(CoreStage::PostUpdate, game_events_system);
        app.add_system_to_stage(CoreStage::PostUpdate, game_update_system.after(game_events_system));
    }
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

pub struct GameInit {
    /// The number of asteroids the game will try to maintain on screen
    pub asteroid_count: u32,
    pub player_lives: u32,
}

#[derive(PartialEq, Eq, Debug)]
pub enum PlayerState {
    Start,
    Ready,
    Respawn,
    Destroyed,
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

pub struct Game {
    pub player_lives_remaining: u32,
    pub player_points: u32,
    pub debug_asteroid_count_on_screen: u32,
    pub scheduled_asteroid_spawns: Vec<ScheduledAsteroidSpawn>,
    pub player_state: PlayerState,
    player_spawn_timer: Timer,
    init: GameInit,
}

impl Game {
    pub fn new(init: GameInit) -> Self {
        let asteroid_count = init.asteroid_count;
        let mut game = Self {
            player_lives_remaining: init.player_lives,
            player_points: 0,
            player_state: PlayerState::Start,
            player_spawn_timer: Timer::from_seconds(GAME_PLAYER_RESPAWN_TIME_SECS, false),
            scheduled_asteroid_spawns: Vec::new(),
            debug_asteroid_count_on_screen: 0,
            init,
        };
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
            _ => PlayerState::Respawn,
        };
        if self.player_state == PlayerState::Respawn {
            self.player_lives_remaining -= 1;
            self.player_spawn_timer.reset();
        }
    }

    fn on_rocket_spawned(&mut self) {
        self.player_state = PlayerState::Ready;
    }

    fn on_asteroid_destroyed(&mut self, event: AsteroidDestroyedEvent) {
        // Update points
        self.player_points += get_points_for_asteroid(event.size);

        // Break apart large asteroids
        if event.size == AsteroidSize::Medium || event.size == AsteroidSize::Large {
            self.schedule_asteroid_to_spawn(0.0, AsteroidSpawnInstruction::FromDestroyedAsteroid(event));
        }
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

    fn tick(&mut self, delta: std::time::Duration) {
        self.player_spawn_timer.tick(delta);
        for s in self.scheduled_asteroid_spawns.iter_mut() {
            s.spawn_timer.tick(delta);
        }
    }

    fn should_spawn_player(&self) -> bool {
        let should_spawn =
            self.player_state == PlayerState::Start ||
            (self.player_state == PlayerState::Respawn && self.player_spawn_timer.finished());
            
        return should_spawn;
    }
}

fn get_points_for_asteroid(size: AsteroidSize) -> u32 {
    match size {
        AsteroidSize::Small => 10,
        AsteroidSize::Medium => 7,
        AsteroidSize::Large => 5,
    }
}

// Systems

// Listen for events and update the game state
fn game_events_system(
    mut game: ResMut<Game>,
    mut rocket_destructions: EventReader<PlayerRocketDestroyedEvent>,
    mut asteroid_destructions: EventReader<AsteroidDestroyedEvent>
) {
    if rocket_destructions.iter().next().is_some() {
        game.on_rocket_destroyed();
    }

    for ev in asteroid_destructions.iter() {
        game.on_asteroid_destroyed(ev.clone());
    }
}

fn game_update_system(
    mut game: ResMut<Game>,
    asteroids: Query<&Asteroid>
) {
    let asteroid_count = asteroids.iter().count();
    game.on_asteroid_count_update(asteroid_count as u32);
}

// Apply game effects to the world
fn game_effects_system(
    mut commands: Commands,
    mut game: ResMut<Game>,
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

    for spawn in game.scheduled_asteroid_spawns.drain_filter(|s| s.spawn_timer.finished()) {
        handle_asteroid_spawn(&mut commands, &mut rng, &world_boundaries, &assets, spawn);
    }
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
            let spawn = AsteroidSpawn { size, shape, position, velocity, rotation };
            spawn_asteroid(commands, &assets.asteroid, spawn);

        },
        AsteroidSpawnInstruction::OffScreen => {
            // Spawn off-screen asteroids
            let position = random_offscreen_position(rng, world_boundaries, 10.0);
            let velocity = random_asteroid_velocity(rng);
            let rotation = random_asteroid_rotation(rng);
            let size = random_asteroid_size(rng);
            let shape = random_asteroid_shape(rng);
            let spawn = AsteroidSpawn { size, shape, position, velocity, rotation };
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
            spawn_asteroid(commands, &assets.asteroid, AsteroidSpawn { size, position: a.0, velocity: a.1, rotation: a.2, shape: a.3 });
            spawn_asteroid(commands, &assets.asteroid, AsteroidSpawn { size, position: b.0, velocity: b.1, rotation: b.2, shape: b.3 });
        },
        AsteroidSpawnInstruction::AtPosition(position) => {
            let velocity = Vec2::ZERO; // random_asteroid_velocity(rng);
            let rotation = 0.0;
            let size = AsteroidSize::Large;
            let shape = AsteroidShapeId::A;
            let spawn = AsteroidSpawn { size, shape, position, velocity, rotation };
            spawn_asteroid(commands, &assets.asteroid, spawn);
        },
    };
}

static CHILD_ASTEROID_SPAWN_DISTANCE: f32 = 2.0;
static CHILD_ASTEROID_MIN_ADD_SPEED: f32 = 5.0;
static CHILD_ASTEROID_MAX_ADD_SPEED: f32 = 15.0;
static CHUNK_ASTEROID_VELOCITY_REDUCTION: f32 = 0.8;

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
    let ray = Ray::from_origin_and_direction(Vec2::ZERO, rng.random_unit_vec2());
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

fn game_keyboard_event_system(
    kb: Res<Input<KeyCode>>,
    mut game: ResMut<Game>
) {
    if kb.just_released(KeyCode::A) {
        game.schedule_asteroid_to_spawn(0.0, AsteroidSpawnInstruction::AtPosition(Vec2::new(0., 20.)));
    }
}