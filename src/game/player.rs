use std::f32::consts::TAU;
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use crate::AppState;
use super::FrameStage;
use super::assets::GameAssets;
use super::asteroid::AsteroidCollidable;
use super::hit::{HitEvent, distinct_hit_events};
use super::movable::{Movable, MovableTorusConstraint, Acceleration, AcceleratingTo};
use super::collidable::{Collidable, Collider};
use super::explosion::{ExplosionShapeId, SpawnExplosion, spawn_explosion};
use super::bullet::{BulletController, BulletFireResult, BulletSpawn, BulletSource, BulletCollidable, spawn_bullet};
use super::invulnerable::Invulnerable;
use super::svg::simple_svg_to_path;
use super::util::*;

// Player's Rocket

const ROCKET_RATE_OF_TURN: f32 = 999.0; // Instant rotation acceleration / deceleration
const ROCKET_RATE_OF_TURN_DRAG: f32 = 999.0;
const ROCKET_RATE_OF_ACCELERATION: f32 = 300.0;
const ROCKET_RATE_OF_ACCELERATION_DRAG: f32 = 50.0;
const ROCKET_MAX_SPEED: f32 = 200.0;
const ROCKET_MAX_DRAG_SPEED: f32 = 20.0;
const ROCKET_MAX_ROTATION_SPEED: f32 = TAU; // 1 rotation per second
const ROCKET_BULLET_SPEED: f32 = 250.0;
const ROCKET_BULLET_MAX_AGE_SECS: f32 = 1.0;
const ROCKET_FIRE_RATE: f32 = 5.0; // per second
const ROCKET_SPAWN_INVULNERABILITY_SECS: f32 = 3.0;
const ROCKET_Z: f32 = 10.0;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlayerRocketDestroyedEvent>();
        app.add_system_set(
            SystemSet::on_update(AppState::Game)
                .with_system(
                    player_keyboard_event_system
                        .label(FrameStage::Input)
                )
                .with_system(
                    player_bullet_system
                        .after(FrameStage::Movement)
                )
                .with_system(
                    player_update_movable_system
                        .after(player_keyboard_event_system)
                )
                .with_system(
                    rocket_exhaust_update_system
                        .after(player_keyboard_event_system)
                )
                .with_system(
                    player_hit_system
                        .label(FrameStage::CollisionEffect)
                        .after(FrameStage::Collision)
                )
        );
        app.add_system_set(
            SystemSet::on_exit(AppState::Game)
                .with_system(player_teardown_system)
        );
    }
}

// Events

pub struct PlayerRocketDestroyedEvent;

// Setup

pub struct RocketAssets {
    rocket_dimension: (f32, f32), // (w, h) of the rocket shape
    rocket_shape: Path,
    rocket_exhaust_shape: Path,
}

pub fn create_roket_assets() -> RocketAssets {
    // See: https://yqnn.github.io/svg-path-editor/
    let rocket_dimension = (4.0, 6.0);
    let rocket_path = "M 3 0 L -2 -2 M -1 -1.6 L -1 1.6 M 3 0 L -2 2";
    let exhaust_path = "M -1 1 L -3 0 L -1 -1";

    RocketAssets {
        rocket_dimension,
        rocket_shape: simple_svg_to_path(rocket_path),
        rocket_exhaust_shape: simple_svg_to_path(exhaust_path),
    }
}

// Teardown

fn player_teardown_system(mut commands: Commands, query: Query<Entity, With<PlayerRocket>>) {
    for entity in query.iter() {
        commands
            .entity(entity)
            .despawn_recursive();
    }
}

// Entity

#[derive(Component, Default)]
pub struct PlayerRocket {
    turning_left: bool,
    turning_right: bool,
    accelerating: bool,
}

#[derive(Component)]
pub struct PlayerRocketExhaust;

fn player_keyboard_event_system(
    kb: Res<Input<KeyCode>>,
    mut rocket_query: Query<(&mut PlayerRocket, &mut BulletController)>
) {
    let turning_left = kb.pressed(KeyCode::Left) || kb.pressed(KeyCode::A);
    let turning_right = kb.pressed(KeyCode::Right) || kb.pressed(KeyCode::D);
    let accelerating = kb.pressed(KeyCode::Up) || kb.pressed(KeyCode::W);
    let firing = kb.pressed(KeyCode::Space);

    for (mut player_rocket, mut bullet_controller) in rocket_query.iter_mut() {
        player_rocket.turning_left = turning_left;
        player_rocket.turning_right = turning_right;
        player_rocket.accelerating = accelerating;
        bullet_controller.try_set_firing_state(firing);
    }
}

fn player_update_movable_system(
    mut rocket_query: Query<(&PlayerRocket, &mut Movable)>
) {

    for (rocket, mut movable) in rocket_query.iter_mut() {
        // Update rotational acceleration
        movable.rotational_acceleration = match (rocket.turning_left, rocket.turning_right) {
            (true, false) => Some(Acceleration::new(ROCKET_RATE_OF_TURN).with_limit(AcceleratingTo::Max(ROCKET_MAX_ROTATION_SPEED))),
            (false, true) => Some(Acceleration::new(-ROCKET_RATE_OF_TURN).with_limit(AcceleratingTo::Max(ROCKET_MAX_ROTATION_SPEED))),
            // Apply "turn drag"
            _ if movable.rotational_velocity > 0. => Some(Acceleration::new(-ROCKET_RATE_OF_TURN_DRAG).with_limit(AcceleratingTo::Zero)),
            _ if movable.rotational_velocity < 0. => Some(Acceleration::new(ROCKET_RATE_OF_TURN_DRAG).with_limit(AcceleratingTo::Zero)),
            _ => None
        };

        // Update acceleration
        movable.acceleration =
            if rocket.accelerating {
                let acc = movable.heading_normal() * ROCKET_RATE_OF_ACCELERATION;
                Some(Acceleration::new(acc).with_limit(AcceleratingTo::Max(ROCKET_MAX_SPEED)))
            }
            // Apply "space drag"
            else if movable.velocity.length() > ROCKET_MAX_DRAG_SPEED {
                let acc = -movable.velocity.normalize() * ROCKET_RATE_OF_ACCELERATION_DRAG;
                Some(Acceleration::new(acc).with_limit(AcceleratingTo::Zero))
            }
            // Not accelerating
            else {
                None
            };
    }
}

// Rocket exhaust flicker system

fn rocket_exhaust_update_system(
    time: Res<Time>,
    rocket_query: Query<(&PlayerRocket, &Children), With<PlayerRocket>>,
    mut exhaust_query: Query<&mut DrawMode, With<PlayerRocketExhaust>>
) {
    let t_secs = time.elapsed_seconds();
    for (rocket, children) in rocket_query.iter() {
        // Update child components
        for &child in children.iter() {
            if let Ok(mut draw_mode) = exhaust_query.get_mut(child) {
                let new_alpha =
                    if rocket.accelerating { exhaust_opacity_over_t(t_secs) }
                    else { 0. };
                update_drawmode_alpha(&mut draw_mode, new_alpha);
            }
        }
    }
}

fn exhaust_opacity_over_t(t_secs: f32) -> f32 {
    // flicker the exhaust between (0.2, 1.0), eight times per second
    let (min, max) = (0.2, 1.0);
    let frequency = 8.;
    let scale = ((t_secs * TAU * frequency).cos() + 1.0) / 2.0;
    min + (max - min) * scale
}

// Spawning

#[derive(Clone)]
pub struct RocketSpawn {
    pub position: Vec2,
    pub velocity: Vec2,
    pub invulnerable: Option<Timer>,
}

impl Default for RocketSpawn {
    fn default() -> Self {
        Self {
            position: default(),
            velocity: default(),
            invulnerable: Some(Timer::from_seconds(ROCKET_SPAWN_INVULNERABILITY_SECS, TimerMode::Once))
        }
    }
}

const LINE_WIDTH: f32 = 0.2;

pub fn spawn_player_rocket(
    commands: &mut Commands,
    assets: &RocketAssets,
    spawn: RocketSpawn
) {
     // Spawn stationary, in the middle of the screen
    let position = spawn.position;
    let velocity = spawn.velocity;
    let (_, rocket_shape_height) = assets.rocket_dimension;

    let initial_heading_angle = std::f32::consts::PI / 2.0; // straight up

    // Rocket
    let rocket_color = Color::rgba(1., 1., 1., 1.);
    let rocket_draw_mode = DrawMode::Stroke(StrokeMode::new(rocket_color, LINE_WIDTH));

    // Rocket exhaust
    let rocket_exhaust_color = Color::rgba(1., 1., 1., 0.);
    let rocket_exhaust_draw_mode = DrawMode::Stroke(StrokeMode::new(rocket_exhaust_color, LINE_WIDTH));
    
    // Transform
    let transform = Transform::from_translation(Vec3::new(position.x, position.y, ROCKET_Z))
        .with_rotation(Quat::from_rotation_z(initial_heading_angle));
    
    // Collision detection
    // NOTE: Currently using a spherical collision box, shrunk down to fit within the hull
    // TODO(benf): Make a triangular Polygon collision shape
    let radius = rocket_shape_height / 2.;
    let collider = Collider::circle(position.into(), radius / 2.);

    // Bullet control
    let bullet_fire_rate = ROCKET_FIRE_RATE;
    let bullet_spawn_translation = Vec2::new(radius, 0.0);

    let entity = commands
        .spawn((
            PlayerRocket::default(),
            Movable {
                position,
                velocity,
                acceleration: None,
                heading_angle: initial_heading_angle,
                rotational_velocity: 0.,
                rotational_acceleration: None,
            },
            MovableTorusConstraint { radius },
            BulletController::new(bullet_fire_rate).with_spawn_translation(bullet_spawn_translation),
            // Collision detection
            AsteroidCollidable,
            BulletCollidable { source: BulletSource::AlienUfo },
            Collidable { collider },
            // Rendering
            GeometryBuilder::build_as(
                &assets.rocket_shape,
                rocket_draw_mode,
                transform
            ),
        ))
        .with_children(|child_commands| {
            child_commands.spawn((
                PlayerRocketExhaust,
                GeometryBuilder::build_as(
                    &assets.rocket_exhaust_shape,
                    rocket_exhaust_draw_mode,
                    Transform::default()
                )
            ));
        })
        .id();

    // Add invulnerability?
    if let Some(timer) = spawn.invulnerable {
        commands
            .entity(entity)
            .insert(Invulnerable::new(timer));
    }
}

// Bullet system

fn player_bullet_system(
    time: Res<Time>,
    assets: Res<GameAssets>,
    mut commands: Commands,
    mut query: Query<(&Movable, &mut BulletController), With<PlayerRocket>>
) {
    for (movable, mut controller) in query.iter_mut() {
        if controller.update(&time) == BulletFireResult::FireBullet {
            let translation = movable.heading_normal().rotate(controller.spawn_translation.unwrap_or_default());
            let velocity = movable.heading_normal() * ROCKET_BULLET_SPEED;
            spawn_bullet(&mut commands, &assets.bullet, BulletSpawn {
                source: BulletSource::PlayerRocket,
                position: movable.position + translation,
                velocity: movable.velocity + velocity,
                heading_angle: movable.heading_angle,
                despawn_after_secs: ROCKET_BULLET_MAX_AGE_SECS,
            });
        }
    }
}

// Destruction system

static PLAYER_ROCKET_EXPLOSION_DESPAWN_AFTER_SECS: f32 = 3.0;

fn player_hit_system(
    mut commands: Commands,
    mut hit_events: EventReader<HitEvent>,
    mut rocket_destroyed: EventWriter<PlayerRocketDestroyedEvent>,
    assets: Res<GameAssets>,
    query: Query<&Movable, With<PlayerRocket>>
) {
    for &HitEvent(entity) in distinct_hit_events(&mut hit_events) {
        if let Ok(movable) = query.get(entity) {
            let mut rng = rand::thread_rng();
            // Despawn the entity
            commands.entity(entity).despawn_recursive();
            // Start the explosion
            spawn_explosion(&mut commands, &mut rng, &assets.explosion, SpawnExplosion {
                shape_id: ExplosionShapeId::RocketDebris,
                shape_scale: 1.0,
                position: movable.position,
                velocity: movable.velocity,
                heading_angle: movable.heading_angle,
                rotational_velocity: movable.rotational_velocity,
                despawn_after_secs: PLAYER_ROCKET_EXPLOSION_DESPAWN_AFTER_SECS,
            });
            // Send events
            rocket_destroyed.send(PlayerRocketDestroyedEvent);
        }
    }
}