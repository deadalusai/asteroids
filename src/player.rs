use std::{f32::consts::TAU, ops::Neg};

use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use crate::movable::*;

// Player's Rocket

static PLAYER_RATE_OF_TURN: f32 = 999.0; // Instant rotation acceleration / deceleration
static PLAYER_RATE_OF_TURN_DRAG: f32 = 999.0;
static PLAYER_RATE_OF_ACCELERATION: f32 = 500.0;
static PLAYER_RATE_OF_ACCELERATION_DRAG: f32 = 180.0;
static PLAYER_MAX_SPEED: f32 = 500.0;
static PLAYER_MAX_DRAG_SPEED: f32 = 50.0;
static PLAYER_MAX_ROTATION_SPEED: f32 = TAU; // 1 rotation per second
static PLAYER_Y: f32 = 10.0;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(CoreStage::Update, player_keyboard_event_system);
    }
}

#[derive(Component)]
pub struct PlayerRocket;

impl PlayerRocket {
    pub fn spawn(
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<ColorMaterial>>
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
            .insert_bundle(MaterialMesh2dBundle {
                mesh: meshes.add(create_rocket_mesh()).into(),
                material: materials.add(ColorMaterial::from(Color::rgba(1., 0.,  0., 1.))),
                transform: Transform::default()
                    .with_translation(Vec3::new(0., 0., PLAYER_Y))
                    .with_scale(Vec3::splat(50.0)),
                ..Default::default()
            });
    }
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

fn player_keyboard_event_system(
    kb: Res<Input<KeyCode>>,
    mut query: Query<&mut Movable, With<PlayerRocket>>,
) {
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
            (true, false) => Some(Acceleration::new(-PLAYER_RATE_OF_TURN).with_limit(AcceleratingTo::Max(PLAYER_MAX_ROTATION_SPEED))),
            (false, true) => Some(Acceleration::new(PLAYER_RATE_OF_TURN).with_limit(AcceleratingTo::Max(PLAYER_MAX_ROTATION_SPEED))),
            // Apply "turn drag"
            _ if movable.rotational_velocity > 0. => Some(Acceleration::new(-PLAYER_RATE_OF_TURN_DRAG).with_limit(AcceleratingTo::Zero)),
            _ if movable.rotational_velocity < 0. => Some(Acceleration::new(PLAYER_RATE_OF_TURN_DRAG).with_limit(AcceleratingTo::Zero)),
            _ => None
        };

        // Update acceleration
        movable.acceleration =
            if accelerating {
                let acc = movable.heading_normal() * PLAYER_RATE_OF_ACCELERATION;
                Some(Acceleration::new(acc).with_limit(AcceleratingTo::Max(PLAYER_MAX_SPEED)))
            }
            // Apply "space drag"
            else if movable.velocity.length() > PLAYER_MAX_DRAG_SPEED {
                let acc = movable.velocity.normalize().neg() * PLAYER_RATE_OF_ACCELERATION_DRAG;
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

// // Asteroids

// enum AsteroidSize {
//     Small, Medium, Large
// }

// #[derive(Component)]
// struct Asteroid {
//     size: AsteroidSize,
// }
