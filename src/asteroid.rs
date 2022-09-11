use std::f32::consts::TAU;

use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use crate::bullet::BulletCollidable;
use crate::hit::HitEvent;
use crate::explosion::*;
use crate::movable::*;
use crate::torus::*;
use crate::svg::*;
use crate::util::*;

pub struct AsteroidPlugin;

impl Plugin for AsteroidPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(asset_initialisation_system);
        app.add_system(asteroid_spawn_system);
        app.add_system(asteroid_collision_system);
        app.add_system_to_stage(CoreStage::PostUpdate, asteroid_destruction_system);
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
        // See: https://yqnn.github.io/svg-path-editor/
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
pub struct Asteroid {
    size: AsteroidSize,
}

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
    let mut rng = rand::thread_rng();

    let position = rng.random_unit_vec2() * Vec2::new(window.width(), window.height()) / 2.0;
    let velocity = ASTEROID_MIN_SPEED + rng.random_unit_vec2() * (ASTEROID_MAX_SPEED - ASTEROID_MIN_SPEED);
    let rotation = ASTEROID_MIN_SPIN_RATE + rng.random_f32() * (ASTEROID_MAX_SPIN_RATE - ASTEROID_MIN_SPIN_RATE);

    // Mesh
    let (diameter, shape) = rng.random_choice(&assets.asteroid_shapes).unwrap();
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
        .insert(Asteroid { size })
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
    asteroids: Query<(Entity, &bevy_sepax2d::components::Sepax), With<Asteroid>>,
    collidables: Query<(Entity, &bevy_sepax2d::components::Sepax), With<AsteroidCollidable>>,
    mut hit_events: EventWriter<HitEvent>
)
{
    for (_, bullet) in asteroids.iter() {
        for (collidable_entity, target) in collidables.iter() {
            if sepax2d::sat_overlap(bullet.shape(), target.shape()) {
                // Collision!
                hit_events.send(HitEvent(collidable_entity));
            }
        }
    }
}

// Destruction system

fn asteroid_destruction_system(
    mut commands: Commands,
    mut explosion_events: EventWriter<SpawnExplosionEvent>,
    mut hit_events: EventReader<HitEvent>,
    query: Query<(&Asteroid, &Movable)>
) {
    for &HitEvent(entity) in hit_events.iter() {
        if let Ok((asteroid, movable)) = query.get(entity) {
            let mut rng = rand::thread_rng();
            // Despawn the entity
            commands.entity(entity).despawn();
            // Start the explosion
            explosion_events.send(make_explosion_event(&mut rng, asteroid, movable, ExplosionAssetId::AsteroidDebrisA));
            explosion_events.send(make_explosion_event(&mut rng, asteroid, movable, ExplosionAssetId::AsteroidDebrisB));
            explosion_events.send(make_explosion_event(&mut rng, asteroid, movable, ExplosionAssetId::AsteroidDebrisC));
        }
    }
}

static ASTEROID_EXPLOSION_DESPAWN_AFTER_SECS: f32 = 0.8;
static ASTEROID_EXPLOSION_MAX_ADD_SPEED: f32 = 100.0;
static ASTEROID_EXPLOSION_MAX_ADD_ROT_SPEED: f32 = 0.5;

fn make_explosion_event(
    rng: &mut rand::rngs::ThreadRng,
    asteroid: &Asteroid,
    movable: &Movable,
    mesh_id: ExplosionAssetId
) -> SpawnExplosionEvent {
    // Add some random spin to the individual parts
    let add_velocity = rng.random_unit_vec2() * rng.random_f32() * ASTEROID_EXPLOSION_MAX_ADD_SPEED;
    let add_rot_velocity = (rng.random_f32() - 0.5) * 2.0 * ASTEROID_EXPLOSION_MAX_ADD_ROT_SPEED;
    SpawnExplosionEvent {
        mesh_id,
        mesh_scale: asteroid_scale(asteroid.size),
        position: movable.position,
        velocity: movable.velocity + add_velocity,
        heading_angle: movable.heading_angle,
        rotational_velocity: movable.rotational_velocity + add_rot_velocity,
        despawn_after_secs: ASTEROID_EXPLOSION_DESPAWN_AFTER_SECS,
    }
}