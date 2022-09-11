use std::f32::consts::TAU;
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use bevy_prototype_lyon::prelude::tess::geom::euclid::approxeq::ApproxEq;
use crate::asteroid::AsteroidCollidable;
use crate::bullet::*;
use crate::movable::*;
use crate::torus::*;
use crate::draw::*;

// Player's Rocket

static ROCKET_RATE_OF_TURN: f32 = 999.0; // Instant rotation acceleration / deceleration
static ROCKET_RATE_OF_TURN_DRAG: f32 = 999.0;
static ROCKET_RATE_OF_ACCELERATION: f32 = 700.0;
static ROCKET_RATE_OF_ACCELERATION_DRAG: f32 = 180.0;
static ROCKET_MAX_SPEED: f32 = 900.0;
static ROCKET_MAX_DRAG_SPEED: f32 = 50.0;
static ROCKET_MAX_ROTATION_SPEED: f32 = TAU; // 1 rotation per second
static ROCKET_BULLET_SPEED: f32 = 900.0;
static ROCKET_BULLET_MAX_AGE_SECS: f32 = 2.0;
static ROCKET_FIRE_RATE: f32 = 5.0; // per second
static ROCKET_SCALE: f32 = 10.0;
static ROCKET_Z: f32 = 10.0;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(asset_initialisation_system);
        app.add_system(player_keyboard_event_system);
        app.add_system(player_spawn_system);
        app.add_system(rocket_exhaust_system);
        app.add_event::<SpawnPlayerRocketEvent>();
    }
}

// Setup

struct PlayerAssets {
    rocket_dimension: (f32, f32), // (w, h) of the rocket shape
    rocket_shape: Path,
    rocket_exhaust_shape: Path,
}

fn asset_initialisation_system(
    mut commands: Commands,
) {
    let rocket_dimension = (4.0, 6.0);
    let rocket_path = "M 0 -3 L -2 2 M -1.6 1 L 1.6 1 M 0 -3 L 2 2";
    let exhaust_path = "M -1 1 L 0 3 L 1 1";

    commands.insert_resource(PlayerAssets {
        rocket_dimension,
        rocket_shape: simple_svg_to_path(rocket_path),
        rocket_exhaust_shape: simple_svg_to_path(exhaust_path),
    });
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
        bullet_controller.set_firing(firing);

        // Update child components
        for &child in children.iter() {
            if let Ok(mut exhaust) = exhaust_query.get_mut(child) {
                exhaust.is_firing = accelerating;
            }
        }
    }
}

// Spawning

pub struct SpawnPlayerRocketEvent;

fn player_spawn_system(
    assets: Res<PlayerAssets>,
    mut spawn_events: EventReader<SpawnPlayerRocketEvent>,
    mut commands: Commands
) {
    for _ in spawn_events.iter() {
        spawn_player_rocket(&assets, &mut commands);
    }
}

const LINE_WIDTH: f32 = 2.0;

fn spawn_player_rocket(
    assets: &Res<PlayerAssets>,
    commands: &mut Commands
) {
     // Spawn stationary, in the middle of the screen
    let position = Vec2::splat(0.);
    let velocity = Vec2::splat(0.);
    let (_, rocket_shape_height) = assets.rocket_dimension;
    let scale = ROCKET_SCALE;

    // Rocket
    let rocket_color = Color::rgba(1., 1., 1., 1.);
    let rocket_draw_mode = DrawMode::Stroke(StrokeMode::new(rocket_color, LINE_WIDTH / scale));

    // Rocket exhaust
    let rocket_exhaust_color = Color::rgba(1., 1., 1., 0.);
    let rocket_exhaust_draw_mode = DrawMode::Stroke(StrokeMode::new(rocket_exhaust_color, LINE_WIDTH / scale));
    
    // Transform
    let transform = Transform::default()
        .with_translation(Vec3::new(position.x, position.y, ROCKET_Z))
        .with_scale(Vec3::splat(scale));

    // Bullet controller
    let b_fire_rate = ROCKET_FIRE_RATE;
    let b_bullet_speed = ROCKET_BULLET_SPEED;
    let b_bullet_max_age_secs = ROCKET_BULLET_MAX_AGE_SECS;
    let b_start_offset = (rocket_shape_height / 2.) * scale; // Offset the bullets forward of the rocket
    
    // Collision detection
    // NOTE: Currently using a spherical collision box, shrunk down to fit within the hull
    // TODO(benf): Make a triangular Polygon collision shape
    let convex = bevy_sepax2d::Convex::Circle(sepax2d::circle::Circle::new(position.into(), scale * rocket_shape_height / 4.0));
    let sepax = bevy_sepax2d::components::Sepax { convex };
    let sepax_movable = bevy_sepax2d::components::Movable { axes: Vec::new() };

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
        .insert(BulletController::new(b_fire_rate, b_start_offset, b_bullet_speed, b_bullet_max_age_secs))
        .insert(TorusConstraint)
        .insert(AsteroidCollidable)
        // Collision detection
        .insert(sepax)
        .insert(sepax_movable)
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
        let stroke = read_stroke(&draw_mode);
        if !stroke.color.a().approx_eq(&new_alpha) {
            // Update the opacity of the stroke
            let color = Color::rgba(
                stroke.color.r(),
                stroke.color.g(),
                stroke.color.b(),
                new_alpha
            );
            *draw_mode = DrawMode::Stroke(StrokeMode { color, ..stroke })
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

fn read_stroke(draw_mode: &DrawMode) -> StrokeMode {
    match draw_mode {
        DrawMode::Stroke(stroke) => *stroke,
        _ => panic!("Called read_stroke_mode on non-stroke draw mode"),
    }
}