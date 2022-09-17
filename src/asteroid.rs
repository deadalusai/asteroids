use std::f32::consts::TAU;
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use rand::{rngs::ThreadRng, thread_rng};
use crate::SystemLabel;
use crate::assets::GameAssets;
use crate::bullet::BulletCollidable;
use crate::hit::{HitEvent, distinct_hit_events};
use crate::movable::{Movable, MovableTorusConstraint};
use crate::collidable::{Collidable, Collider};
use crate::explosion::{ExplosionAssetId, SpawnExplosion, spawn_explosions};
use crate::svg::simple_svg_to_path;
use crate::util::*;

pub struct AsteroidPlugin;

impl Plugin for AsteroidPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(
            asteroid_collision_system
                .label(SystemLabel::Collision)
                .after(SystemLabel::Movement)
        );
        app.add_system(
            asteroid_hit_system
                .after(SystemLabel::Collision)
        );
        app.add_event::<AsteroidDestroyedEvent>();
    }
}

// Events

#[derive(Clone)]
pub struct AsteroidDestroyedEvent {
    pub size: AsteroidSize,
    pub kind: AsteroidKind,
    pub position: Vec2,
    pub velocity: Vec2,
}

// Setup

pub struct AsteroidAssets {
    asteroid_shapes: Vec<(f32, Path)>,
}

pub fn create_asteroid_assets() -> AsteroidAssets {
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

    AsteroidAssets { asteroid_shapes }
}

// Asteroids

static ASTEROID_Z: f32 = 20.0;
static ASTEROID_MAX_SPIN_RATE: f32 = TAU * 0.7;
static ASTEROID_MIN_SPIN_RATE: f32 = TAU * 0.05;
static ASTEROID_SMALL_SCALE: f32 = 1.0;
static ASTEROID_MEDIUM_SCALE: f32 = 2.0;
static ASTEROID_LARGE_SCALE: f32 = 3.0;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AsteroidSize {
    Small, Medium, Large
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AsteroidKind {
    Original,
    Chunk
}

#[derive(Component)]
pub struct Asteroid {
    size: AsteroidSize,
    kind: AsteroidKind,
}

/// Marker component which indicates that an entity should be considered for asteroid collisions
#[derive(Component)]
pub struct AsteroidCollidable;

// Spawning

const LINE_WIDTH: f32 = 0.2;

fn asteroid_scale(size: AsteroidSize) -> f32 {
    match size {
        AsteroidSize::Small => ASTEROID_SMALL_SCALE,
        AsteroidSize::Medium => ASTEROID_MEDIUM_SCALE,
        AsteroidSize::Large => ASTEROID_LARGE_SCALE,
    }
}

#[derive(Clone, Copy)]
pub struct AsteroidSpawn {
    pub size: AsteroidSize,
    pub kind: AsteroidKind,
    pub position: Vec2,
    pub velocity: Vec2,
}

pub fn spawn_asteroid(
    commands: &mut Commands,
    assets: &AsteroidAssets,
    rng: &mut ThreadRng,
    spawn: AsteroidSpawn
) {
    let position = spawn.position;
    let velocity = spawn.velocity;
    let rotation = ASTEROID_MIN_SPIN_RATE + rng.random_f32() * (ASTEROID_MAX_SPIN_RATE - ASTEROID_MIN_SPIN_RATE);

    // Mesh
    let (diameter, shape) = rng.random_choice(&assets.asteroid_shapes).unwrap();
    let scale = asteroid_scale(spawn.size);

    let color = Color::rgba(0.6, 0.6, 0.6, 1.);
    let draw_mode = DrawMode::Stroke(StrokeMode::new(color, LINE_WIDTH / scale));
    let transform = Transform::default()
        .with_translation(Vec3::new(position.x, position.y, ASTEROID_Z))
        .with_scale(Vec3::splat(scale));

    // Collision detection
    let collider = Collider::circle(position.into(), scale * diameter / 2.);

    commands
        .spawn()
        .insert(Asteroid {
            size: spawn.size,
            kind: spawn.kind,
        })
        .insert(Movable {
            position,
            velocity,
            acceleration: None,
            heading_angle: 0.,
            rotational_velocity: rotation,
            rotational_acceleration: None,
        })
        .insert(MovableTorusConstraint)
        .insert_bundle(GeometryBuilder::build_as(shape, draw_mode, transform))
        // Collision detection
        .insert(Collidable { collider })
        .insert(BulletCollidable);
}

// Collision detection

fn  asteroid_collision_system(
    asteroids: Query<(Entity, &Collidable), With<Asteroid>>,
    collidables: Query<(Entity, &Collidable), With<AsteroidCollidable>>,
    mut hit_events: EventWriter<HitEvent>
)
{
    for (asteroid, bullet) in asteroids.iter() {
        for (other, target) in collidables.iter() {
            if bullet.test_collision_with(&target) {
                // Collision!
                hit_events.send(HitEvent(asteroid));
                hit_events.send(HitEvent(other));
            }
        }
    }
}

// Hit handling system

fn asteroid_hit_system(
    mut commands: Commands,
    mut hit_events: EventReader<HitEvent>,
    mut asteroid_destroyed: EventWriter<AsteroidDestroyedEvent>,
    assets: Res<GameAssets>,
    query: Query<(&Asteroid, &Movable)>
) {
    for &HitEvent(entity) in distinct_hit_events(&mut hit_events) {
        if let Ok((asteroid, movable)) = query.get(entity) {
            let mut rng = thread_rng();
            // Despawn the entity
            commands.entity(entity).despawn();
            // Start the explosion
            spawn_explosions(
                &mut commands,
                &assets.explosion,
                &[
                    make_explosion_spawn(&mut rng, asteroid, movable, ExplosionAssetId::AsteroidDebrisA),
                    make_explosion_spawn(&mut rng, asteroid, movable, ExplosionAssetId::AsteroidDebrisB),
                    make_explosion_spawn(&mut rng, asteroid, movable, ExplosionAssetId::AsteroidDebrisC),
                ]
            );
            // Send events
            asteroid_destroyed.send(AsteroidDestroyedEvent {
                size: asteroid.size,
                kind: asteroid.kind,
                position: movable.position,
                velocity: movable.velocity
            });
        }
    }
}

static ASTEROID_EXPLOSION_DESPAWN_AFTER_SECS: f32 = 0.8;
static ASTEROID_EXPLOSION_MAX_ADD_SPEED: f32 = 100.0;
static ASTEROID_EXPLOSION_MAX_ADD_ROT_SPEED: f32 = 0.5;

fn make_explosion_spawn(
    rng: &mut rand::rngs::ThreadRng,
    asteroid: &Asteroid,
    movable: &Movable,
    mesh_id: ExplosionAssetId
) -> SpawnExplosion {
    // Add some random spin to the individual parts
    let add_velocity = rng.random_unit_vec2() * rng.random_f32() * ASTEROID_EXPLOSION_MAX_ADD_SPEED;
    let add_rot_velocity = (rng.random_f32() - 0.5) * 2.0 * ASTEROID_EXPLOSION_MAX_ADD_ROT_SPEED;
    SpawnExplosion {
        shape_id: mesh_id,
        shape_scale: asteroid_scale(asteroid.size),
        position: movable.position,
        velocity: movable.velocity + add_velocity,
        heading_angle: movable.heading_angle,
        rotational_velocity: movable.rotational_velocity + add_rot_velocity,
        despawn_after_secs: ASTEROID_EXPLOSION_DESPAWN_AFTER_SECS,
    }
}