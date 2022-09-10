use std::f32::consts::TAU;

use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use rand::random;
use crate::movable::*;
use crate::torus::*;

pub struct AsteroidPlugin;

impl Plugin for AsteroidPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(asset_initialisation_system);
        app.add_system(asteroid_spawn_system);
    }
}

// Setup

struct AsteroidAssets {
    asteroid_mesh: Handle<Mesh>,
    asteroid_material: Handle<ColorMaterial>,
}

fn asset_initialisation_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>
) {
    commands.insert_resource(AsteroidAssets {
        asteroid_mesh: meshes.add(create_asteroid_mesh()),
        asteroid_material: materials.add(ColorMaterial::from(Color::rgba(1., 0.,  0., 1.))),
    });
}

fn create_asteroid_mesh() -> Mesh {
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

// Asteroids

static ASTEROID_Y: f32 = 20.0;
static ASTEROID_MAX_SPEED: f32 = 350.0;
static ASTEROID_MIN_SPEED: f32 = 80.0;
static ASTEROID_MAX_SPIN_RATE: f32 = TAU;
static ASTEROID_MIN_SPIN_RATE: f32 = TAU * 0.2;

#[derive(Clone, Copy)]
pub enum AsteroidSize {
    Small, Medium, Large
}

#[derive(Component)]
pub struct Asteroid {
    size: AsteroidSize,
}

fn asteroid_scale(size: AsteroidSize) -> f32 {
    match size {
        AsteroidSize::Small => 40.0,
        AsteroidSize::Medium => 90.0,
        AsteroidSize::Large => 160.0,
    }
}

fn random_unit_point() -> Vec2 {
    let x = random::<f32>() * 2.0 - 1.0;
    let y = random::<f32>() * 2.0 - 1.0;
    Vec2::new(x, y).normalize()
}

fn spawn_asteroid(
    assets: &Res<AsteroidAssets>,
    commands: &mut Commands,
    window: &Window,
    size: AsteroidSize
) {
    let position = random_unit_point() * Vec2::new(window.width() / 2.0, window.height() / 2.0);
    let velocity = Vec2::splat(ASTEROID_MIN_SPEED) + random_unit_point() * (ASTEROID_MAX_SPEED - ASTEROID_MIN_SPEED);
    let rotation = random::<f32>() * (ASTEROID_MAX_SPIN_RATE - ASTEROID_MIN_SPIN_RATE);

    commands
        .spawn()
        .insert(Asteroid { size })
        .insert(Movable {
            position: position,
            velocity: velocity,
            acceleration: None,
            heading_angle: 0.,
            rotational_velocity: rotation,
            rotational_acceleration: None,
        })
        .insert(TorusConstraint::new(asteroid_scale(size)))
        .insert_bundle(MaterialMesh2dBundle {
            mesh: assets.asteroid_mesh.clone().into(),
            material: assets.asteroid_material.clone(),
            transform: Transform::default()
                .with_translation(Vec3::new(0., 0., ASTEROID_Y))
                .with_scale(Vec3::splat(asteroid_scale(size))),
            ..Default::default()
        });
}

pub struct SpawnAsteroidEvent(pub AsteroidSize);

fn asteroid_spawn_system(
    mut spawn_events: EventReader<SpawnAsteroidEvent>,
    windows: Res<Windows>,
    assets: Res<AsteroidAssets>,
    mut commands: Commands
) {
    for &SpawnAsteroidEvent(size) in spawn_events.iter() {
        spawn_asteroid(&assets, &mut commands, windows.get_primary().unwrap(), size);
    }
}