use std::{f32::consts::TAU, ops::Neg};

use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use crate::movable::*;
use crate::torus::*;

// Player's Rocket

static ROCKET_RATE_OF_TURN: f32 = 999.0; // Instant rotation acceleration / deceleration
static ROCKET_RATE_OF_TURN_DRAG: f32 = 999.0;
static ROCKET_RATE_OF_ACCELERATION: f32 = 500.0;
static ROCKET_RATE_OF_ACCELERATION_DRAG: f32 = 180.0;
static ROCKET_MAX_SPEED: f32 = 500.0;
static ROCKET_MAX_DRAG_SPEED: f32 = 50.0;
static ROCKET_MAX_ROTATION_SPEED: f32 = TAU; // 1 rotation per second
static ROCKET_SCALE: f32 = 50.0;
static ROCKET_Y: f32 = 10.0;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(asset_initialisation_system);
        app.add_system(rocket_keyboard_event_system);
        app.add_system(asteroid_spawn_system);
    }
}

// Setup

struct PlayerAssets {
    rocket_mesh: Handle<Mesh>,
    rocket_material: Handle<ColorMaterial>,
}

fn asset_initialisation_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>
) {
    commands.insert_resource(PlayerAssets {
        rocket_mesh: meshes.add(create_rocket_mesh()),
        rocket_material: materials.add(ColorMaterial::from(Color::rgba(1., 0.,  0., 1.))),
    });
}

fn create_rocket_mesh() -> Mesh {
    let mut mesh = Mesh::new(bevy::render::render_resource::PrimitiveTopology::TriangleList);
  
    mesh.insert_attribute(
      Mesh::ATTRIBUTE_POSITION,
      vec![[0.0, 0.5, 0.0], [-0.25, -0.5, 0.0], [0.25, -0.5, 0.0]],
    );
    mesh.set_indices(Some(bevy::render::mesh::Indices::U32(vec![0, 1, 2])));
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 0.0, 1.0]; 3]);
    mesh.insert_attribute(
      Mesh::ATTRIBUTE_UV_0,
      vec![[0.5, 0.0], [0.0, 1.0], [1.0, 1.0]],
    );
  
    mesh
}

// Entity

#[derive(Component)]
pub struct PlayerRocket;

fn rocket_keyboard_event_system(
    kb: Res<Input<KeyCode>>,
    mut query: Query<&mut Movable, With<PlayerRocket>>,
    mut spawn_events: EventWriter<SpawnPlayerRocketEvent>,
) {
    if kb.just_released(KeyCode::T) {
        spawn_events.send(SpawnPlayerRocketEvent);
    }

    let turning_left = kb.pressed(KeyCode::Left);
    let turning_right = kb.pressed(KeyCode::Right);
    let accelerating = kb.pressed(KeyCode::Up);

    for mut movable in query.iter_mut() {

        // DEBUG: Reset position?
        if kb.pressed(KeyCode::Space) {
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
                let acc = movable.velocity.normalize().neg() * ROCKET_RATE_OF_ACCELERATION_DRAG;
                Some(Acceleration::new(acc).with_limit(AcceleratingTo::Zero))
            }
            // Not accelerating
            else {
                None
            };
    }
}


// // Bullets

// #[derive(Component)]
// struct Bullet;

// Spawning

pub struct SpawnPlayerRocketEvent;

fn asteroid_spawn_system(
    mut spawn_events: EventReader<SpawnPlayerRocketEvent>,
    assets: Res<PlayerAssets>,
    mut commands: Commands
) {
    for _ in spawn_events.iter() {
        spawn_player_rocket(&assets, &mut commands);
    }
}

fn spawn_player_rocket(
    assets: &Res<PlayerAssets>,
    commands: &mut Commands
) {
    commands
        .spawn()
        .insert(PlayerRocket)
        .insert(Movable {
            position: Vec2::new(0., 0.),
            velocity: Vec2::splat(0.),
            acceleration: None,
            heading_angle: 0.,
            rotational_velocity: 0.,
            rotational_acceleration: None,
        })
        .insert(TorusConstraint::new(ROCKET_SCALE))
        .insert_bundle(MaterialMesh2dBundle {
            mesh: assets.rocket_mesh.clone().into(),
            material: assets.rocket_material.clone(),
            transform: Transform::default()
                .with_translation(Vec3::new(0., 0., ROCKET_Y))
                .with_scale(Vec3::splat(ROCKET_SCALE)),
            ..Default::default()
        });
}
