use std::f32::consts::TAU;
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use crate::SystemLabel;
use crate::assets::GameAssets;
use crate::asteroid::AsteroidCollidable;
use crate::hit::{HitEvent, distinct_hit_events};
use crate::movable::{Movable, MovableTorusConstraint, Acceleration, AcceleratingTo};
use crate::collidable::{Collidable, Collider};
use crate::explosion::{ExplosionShapeId, SpawnExplosion, spawn_explosion};
use crate::bullet::{BulletController};
use crate::svg::simple_svg_to_path;
use crate::util::*;

// Player's Rocket

static ROCKET_RATE_OF_TURN: f32 = 999.0; // Instant rotation acceleration / deceleration
static ROCKET_RATE_OF_TURN_DRAG: f32 = 999.0;
static ROCKET_RATE_OF_ACCELERATION: f32 = 300.0;
static ROCKET_RATE_OF_ACCELERATION_DRAG: f32 = 50.0;
static ROCKET_MAX_SPEED: f32 = 200.0;
static ROCKET_MAX_DRAG_SPEED: f32 = 20.0;
static ROCKET_MAX_ROTATION_SPEED: f32 = TAU; // 1 rotation per second
static ROCKET_BULLET_SPEED: f32 = 250.0;
static ROCKET_BULLET_MAX_AGE_SECS: f32 = 2.0;
static ROCKET_FIRE_RATE: f32 = 5.0; // per second
static ROCKET_Z: f32 = 10.0;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(
            player_keyboard_event_system
                .label(SystemLabel::Input)
        );
        app.add_system(
            player_hit_system
                .after(SystemLabel::Collision)
        );
        app.add_system(rocket_exhaust_system);
        app.add_event::<PlayerRocketDestroyedEvent>();
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
    let rocket_path = "M 0 -3 L -2 2 M -1.6 1 L 1.6 1 M 0 -3 L 2 2";
    let exhaust_path = "M -1 1 L 0 3 L 1 1";

    RocketAssets {
        rocket_dimension,
        rocket_shape: simple_svg_to_path(rocket_path),
        rocket_exhaust_shape: simple_svg_to_path(exhaust_path),
    }
}

// Entity

#[derive(Component)]
pub struct PlayerRocket;

#[derive(Component)]
pub struct PlayerRocketExhaust {
    is_firing: bool,
}

fn player_keyboard_event_system(
    kb: Res<Input<KeyCode>>,
    mut rocket_query: Query<(&mut Movable, &mut BulletController, &Children), With<PlayerRocket>>,
    mut exhaust_query: Query<&mut PlayerRocketExhaust>
) {
    let turning_left = kb.pressed(KeyCode::Left);
    let turning_right = kb.pressed(KeyCode::Right);
    let accelerating = kb.pressed(KeyCode::Up);
    let firing = kb.pressed(KeyCode::Space);

    for (mut movable, mut bullet_controller, children) in rocket_query.iter_mut() {

        // DEBUG: Reset rocket position
        if kb.pressed(KeyCode::R) {
            movable.position = Vec2::new(0., 0.);
            movable.velocity = Vec2::splat(0.);
            movable.acceleration = None;
            movable.heading_angle = 0.;
            movable.rotational_velocity = 0.;
            movable.rotational_acceleration = None;
        }

        // Update rotational acceleration
        movable.rotational_acceleration = match (turning_left, turning_right) {
            (true, false) => Some(Acceleration::new(-ROCKET_RATE_OF_TURN).with_limit(AcceleratingTo::Max(ROCKET_MAX_ROTATION_SPEED))),
            (false, true) => Some(Acceleration::new(ROCKET_RATE_OF_TURN).with_limit(AcceleratingTo::Max(ROCKET_MAX_ROTATION_SPEED))),
            // Apply "turn drag"
            _ if movable.rotational_velocity > 0. => Some(Acceleration::new(-ROCKET_RATE_OF_TURN_DRAG).with_limit(AcceleratingTo::Zero)),
            _ if movable.rotational_velocity < 0. => Some(Acceleration::new(ROCKET_RATE_OF_TURN_DRAG).with_limit(AcceleratingTo::Zero)),
            _ => None
        };

        // Update acceleration
        movable.acceleration =
            if accelerating {
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

        // Start up the guns?
        bullet_controller.try_set_firing_state(firing);

        // Update child components
        for &child in children.iter() {
            if let Ok(mut exhaust) = exhaust_query.get_mut(child) {
                exhaust.is_firing = accelerating;
            }
        }
    }
}

// Spawning

#[derive(Default)]
pub struct RocketSpawn {
    position: Vec2,
    velocity: Vec2,
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

    // Rocket
    let rocket_color = Color::rgba(1., 1., 1., 1.);
    let rocket_draw_mode = DrawMode::Stroke(StrokeMode::new(rocket_color, LINE_WIDTH));

    // Rocket exhaust
    let rocket_exhaust_color = Color::rgba(1., 1., 1., 0.);
    let rocket_exhaust_draw_mode = DrawMode::Stroke(StrokeMode::new(rocket_exhaust_color, LINE_WIDTH));
    
    // Transform
    let transform = Transform::from_translation(Vec3::new(position.x, position.y, ROCKET_Z));

    // Bullet controller
    let b_fire_rate = ROCKET_FIRE_RATE;
    let b_bullet_speed = ROCKET_BULLET_SPEED;
    let b_bullet_max_age_secs = ROCKET_BULLET_MAX_AGE_SECS;
    let b_start_offset = rocket_shape_height / 2.0; // Offset the bullets forward of the rocket
    
    // Collision detection
    // NOTE: Currently using a spherical collision box, shrunk down to fit within the hull
    // TODO(benf): Make a triangular Polygon collision shape
    let collider = Collider::circle(position.into(), rocket_shape_height / 4.0);

    commands
        .spawn()
        .insert(PlayerRocket)
        .insert(Movable {
            position,
            velocity,
            acceleration: None,
            heading_angle: 0.,
            rotational_velocity: 0.,
            rotational_acceleration: None,
        })
        .insert(MovableTorusConstraint)
        .insert(BulletController::new(b_fire_rate, b_start_offset, b_bullet_speed, b_bullet_max_age_secs))
        // Collision detection
        .insert(AsteroidCollidable)
        .insert(Collidable { collider })
        // Rendering
        .insert_bundle(GeometryBuilder::build_as(
            &assets.rocket_shape,
            rocket_draw_mode,
            transform
        ))
        .with_children(|child_commands| {
            child_commands
                .spawn()
                .insert(PlayerRocketExhaust {
                    is_firing: false,
                })
                .insert_bundle(GeometryBuilder::build_as(
                    &assets.rocket_exhaust_shape,
                    rocket_exhaust_draw_mode,
                    Transform::default()
                ));
        });
}

// Rocket exhaust flicker system

fn rocket_exhaust_system(
    time: Res<Time>,
    mut query: Query<(&PlayerRocketExhaust, &mut DrawMode)>
) {
    let t_secs = time.seconds_since_startup() as f32;

    for (exhaust, mut draw_mode) in query.iter_mut() {
        let new_alpha = if exhaust.is_firing { exhaust_opacity_over_t(t_secs) } else { 0. };
        update_drawmode_alpha(&mut draw_mode, new_alpha);
    }
}

fn exhaust_opacity_over_t(t_secs: f32) -> f32 {
    // flicker the exhaust between (0.2, 1.0), eight times per second
    let (min, max) = (0.2, 1.0);
    let frequency = 8.;
    let scale = ((t_secs * TAU * frequency).cos() + 1.0) / 2.0;
    min + (max - min) * scale
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
                heading_angle_rads: movable.heading_angle,
                rotational_velocity: movable.rotational_velocity,
                despawn_after_secs: PLAYER_ROCKET_EXPLOSION_DESPAWN_AFTER_SECS,
            });
            // Send events
            rocket_destroyed.send(PlayerRocketDestroyedEvent);
        }
    }
}