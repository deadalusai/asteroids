use std::f32::consts::TAU;

use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use crate::bullet::BulletCollidable;
use crate::movable::*;
use crate::torus::*;
use crate::draw::*;

pub struct AsteroidPlugin;

impl Plugin for AsteroidPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(asset_initialisation_system);
        app.add_system(asteroid_spawn_system);
        app.add_system(asteroid_collision_system);
        app.add_event::<SpawnAsteroidEvent>();
    }
}

// Setup

struct AsteroidAssets {
    asteroid_shapes: Vec<(f32, Path)>,
}

fn asset_initialisation_system(mut commands: Commands) {
    let asteroid_shapes = vec![
        // diameter, path
        (10.0, "M -2 -5 L -5 -2 L -5 0 L -2 0 L -5 2 L -2 5 L 3 4 L 2 2 L 5 0 L 4 -2 L 1 -5 Z"),
        (10.0, "M -5 -3 L -5 2 L -3 5 L 2 5 L 5 3 L 4 1 L 6 -2 L 4 -5 L 1 -3 L -2 -6 Z"),
        (10.0, "M 4 -3 L 0 -5 L -3 -5 L -2 -2 L -5 -2 L -5 0 L -2 5 L 1 2 L 2 4 L 5 1 L 1 -1 L 5 -2 L 5 -3 Z")
    ];
    let asteroid_shapes = asteroid_shapes.into_iter()
        .map(|(dim, svg)| (dim, simple_svg_to_path(svg)))
        .collect();

    commands.insert_resource(AsteroidAssets { asteroid_shapes });
}

// Asteroids

static ASTEROID_Z: f32 = 20.0;
static ASTEROID_MAX_SPEED: f32 = 350.0;
static ASTEROID_MIN_SPEED: f32 = 80.0;
static ASTEROID_MAX_SPIN_RATE: f32 = TAU * 0.7;
static ASTEROID_MIN_SPIN_RATE: f32 = TAU * 0.05;
static ASTEROID_SCALE_SMALL: f32 = 5.0;
static ASTEROID_SCALE_MEDIUM: f32 = 10.0;
static ASTEROID_SCALE_LARGE: f32 = 15.0;

#[derive(Clone, Copy)]
pub enum AsteroidSize {
    Small, Medium, Large
}

#[derive(Component)]
pub struct Asteroid;

/// Marker component which indicates that an entity should be considered for asteroid collisions
#[derive(Component)]
pub struct AsteroidCollidable;

// Spawning

pub struct SpawnAsteroidEvent(pub AsteroidSize);

fn asteroid_spawn_system(
    windows: Res<Windows>,
    assets: Res<AsteroidAssets>,
    mut spawn_events: EventReader<SpawnAsteroidEvent>,
    mut commands: Commands
) {
    for &SpawnAsteroidEvent(size) in spawn_events.iter() {
        spawn_asteroid(&mut commands, &assets, windows.get_primary().unwrap(), size);
    }
}

fn random_unit_vec2(rng: &mut impl rand::Rng) -> Vec2 {
    let x = rng.gen::<f32>() * 2.0 - 1.0;
    let y = rng.gen::<f32>() * 2.0 - 1.0;
    Vec2::new(x, y).normalize()
}

fn asteroid_scale(size: AsteroidSize) -> f32 {
    match size {
        AsteroidSize::Small => ASTEROID_SCALE_SMALL,
        AsteroidSize::Medium => ASTEROID_SCALE_MEDIUM,
        AsteroidSize::Large => ASTEROID_SCALE_LARGE,
    }
}

const LINE_WIDTH: f32 = 2.0;

fn spawn_asteroid(
    commands: &mut Commands,
    assets: &AsteroidAssets,
    window: &Window,
    size: AsteroidSize
) {
    use rand::{Rng, seq::SliceRandom};
    let mut rng = rand::thread_rng();

    let position = random_unit_vec2(&mut rng) * Vec2::new(window.width(), window.height()) / 2.0;
    let velocity = ASTEROID_MIN_SPEED + random_unit_vec2(&mut rng) * (ASTEROID_MAX_SPEED - ASTEROID_MIN_SPEED);
    let rotation = ASTEROID_MIN_SPIN_RATE + rng.gen::<f32>() * (ASTEROID_MAX_SPIN_RATE - ASTEROID_MIN_SPIN_RATE);

    // Mesh
    let (diameter, shape) = assets.asteroid_shapes.choose(&mut rng).unwrap();
    let scale = asteroid_scale(size);

    let color = Color::rgba(0.6, 0.6, 0.6, 1.);
    let draw_mode = DrawMode::Stroke(StrokeMode::new(color, LINE_WIDTH / scale));
    let transform = Transform::default()
        .with_translation(Vec3::new(position.x, position.y, ASTEROID_Z))
        .with_scale(Vec3::splat(scale));

    // Collision detection
    let convex = bevy_sepax2d::Convex::Circle(sepax2d::circle::Circle::new(position.into(), scale * diameter / 2.0));
    let sepax = bevy_sepax2d::components::Sepax { convex };
    let sepax_movable = bevy_sepax2d::components::Movable { axes: Vec::new() };

    commands
        .spawn()
        .insert(Asteroid)
        .insert(Movable {
            position,
            velocity,
            acceleration: None,
            heading_angle: 0.,
            rotational_velocity: rotation,
            rotational_acceleration: None,
        })
        .insert(TorusConstraint)
        .insert(BulletCollidable)
        .insert_bundle(GeometryBuilder::build_as(shape, draw_mode, transform))
        // Collision detection
        .insert(sepax)
        .insert(sepax_movable);
}

// Collision detection

fn  asteroid_collision_system(
    mut commands: Commands,
    asteroids: Query<(Entity, &bevy_sepax2d::components::Sepax), With<Asteroid>>,
    collidables: Query<(Entity, &bevy_sepax2d::components::Sepax), With<AsteroidCollidable>>
)
{
    for (_, bullet) in asteroids.iter() {
        for (target_entity, target) in collidables.iter() {
            if sepax2d::sat_overlap(bullet.shape(), target.shape()) {
                // Collision!
                commands.entity(target_entity).despawn_recursive();
            }
        }
    }
}