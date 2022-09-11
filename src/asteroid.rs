use std::f32::consts::TAU;

use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use rand::random;
use crate::movable::*;
use crate::torus::*;

pub struct AsteroidPlugin;

impl Plugin for AsteroidPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(asset_initialisation_system);
        app.add_system(asteroid_spawn_system);
        app.add_event::<SpawnAsteroidEvent>();
    }
}

// Setup

struct AsteroidAssets {
    asteroid_shape_large: shapes::RegularPolygon,
    asteroid_shape_medium: shapes::RegularPolygon,
    asteroid_shape_small: shapes::RegularPolygon,
}

fn asset_initialisation_system(
    mut commands: Commands,
) {
    commands.insert_resource(AsteroidAssets {
        asteroid_shape_large: asteroid_shape(AsteroidSize::Large),
        asteroid_shape_medium: asteroid_shape(AsteroidSize::Medium),
        asteroid_shape_small: asteroid_shape(AsteroidSize::Small),
    });
}

fn asteroid_shape(size: AsteroidSize) -> shapes::RegularPolygon {
    shapes::RegularPolygon {
        sides: 5,
        feature: shapes::RegularPolygonFeature::Radius(asteroid_diameter(size) / 2.0),
        ..Default::default()
    }
}

// Asteroids

static ASTEROID_Z: f32 = 20.0;
static ASTEROID_MAX_SPEED: f32 = 350.0;
static ASTEROID_MIN_SPEED: f32 = 80.0;
static ASTEROID_MAX_SPIN_RATE: f32 = TAU * 0.7;
static ASTEROID_MIN_SPIN_RATE: f32 = TAU * 0.05;
static ASTEROID_DIAMETER_SMALL: f32 = 25.0;
static ASTEROID_DIAMETER_MEDIUM: f32 = 70.0;
static ASTEROID_DIAMETER_LARGE: f32 = 120.0;

#[derive(Clone, Copy)]
pub enum AsteroidSize {
    Small, Medium, Large
}

#[derive(Component)]
pub struct Asteroid;

// Spawning

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

fn asteroid_diameter(size: AsteroidSize) -> f32 {
    match size {
        AsteroidSize::Small => ASTEROID_DIAMETER_SMALL,
        AsteroidSize::Medium => ASTEROID_DIAMETER_MEDIUM,
        AsteroidSize::Large => ASTEROID_DIAMETER_LARGE,
    }
}

fn random_unit_vec2() -> Vec2 {
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
    let position = random_unit_vec2() * Vec2::new(window.width(), window.height()) / 2.0;
    let velocity = ASTEROID_MIN_SPEED + random_unit_vec2() * (ASTEROID_MAX_SPEED - ASTEROID_MIN_SPEED);
    let rotation = ASTEROID_MIN_SPIN_RATE + random::<f32>() * (ASTEROID_MAX_SPIN_RATE - ASTEROID_MIN_SPIN_RATE);

    // Mesh
    let shape = match size {
        AsteroidSize::Small => &assets.asteroid_shape_small,
        AsteroidSize::Medium => &assets.asteroid_shape_medium,
        AsteroidSize::Large => &assets.asteroid_shape_large,
    };
    let draw_mode = DrawMode::Stroke(StrokeMode::color(Color::rgba(0.6, 0.6, 0.6, 1.)));
    let transform = Transform::default().with_translation(Vec3::new(position.x, position.y, ASTEROID_Z));

    // collision detection
    let convex = bevy_sepax2d::Convex::Circle(sepax2d::circle::Circle::new(position.into(), asteroid_diameter(size) / 2.0));
    let sepax = bevy_sepax2d::components::Sepax { convex };
    let sepax_movable = bevy_sepax2d::components::Movable { axes: Vec::new() };

    commands
        .spawn()
        .insert(Asteroid)
        .insert(Movable {
            position: position,
            velocity: velocity,
            acceleration: None,
            heading_angle: 0.,
            rotational_velocity: rotation,
            rotational_acceleration: None,
        })
        .insert(TorusConstraint::new(asteroid_diameter(size)))
        .insert_bundle(GeometryBuilder::build_as(shape, draw_mode, transform))
        // Collision detection
        .insert(sepax)
        .insert(sepax_movable);
}