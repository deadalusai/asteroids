use std::f32::consts::TAU;

use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use crate::bullet::BulletCollidable;
use crate::hit::*;
use crate::viewport::*;
use crate::movable::*;
use crate::explosion::*;
use crate::svg::*;
use crate::util::*;

pub struct AsteroidPlugin;

impl Plugin for AsteroidPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(asset_initialisation_system);
        app.add_system(asteroid_spawn_system);
        app.add_system(asteroid_collision_system);
        app.add_system_to_stage(CoreStage::PostUpdate, asteroid_hit_system);
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

#[derive(Clone, Copy, PartialEq, Eq)]
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
#[derive(Clone, Copy)]
pub struct SpawnAsteroidPositionVelocity {
    position: Vec2,
    velocity: Vec2,
}

pub struct SpawnAsteroidEvent(pub AsteroidSize, pub Option<SpawnAsteroidPositionVelocity>);

fn asteroid_spawn_system(
    viewport: Res<Viewport>,
    assets: Res<AsteroidAssets>,
    mut spawn_events: EventReader<SpawnAsteroidEvent>,
    mut commands: Commands
) {
    for &SpawnAsteroidEvent(size, pos_vel) in spawn_events.iter() {
        spawn_asteroid(&mut commands, &assets, &viewport, size, pos_vel);
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
    viewport: &Viewport,
    size: AsteroidSize,
    pos_vel: Option<SpawnAsteroidPositionVelocity>
) {
    let mut rng = rand::thread_rng();
    let (position, velocity) = match pos_vel {
        Some(loc) => (loc.position, loc.velocity),
        None => {
            // Generate a random asteroid
            let position = rng.random_unit_vec2() * Vec2::new(viewport.width, viewport.height) / 2.0;
            let velocity = ASTEROID_MIN_SPEED + rng.random_unit_vec2() * (ASTEROID_MAX_SPEED - ASTEROID_MIN_SPEED);
            (position, velocity)
        }
    };
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
        .insert(MovableTorusConstraint)
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

// Hit handling system

fn asteroid_hit_system(
    mut commands: Commands,
    mut spawn_asteroid: EventWriter<SpawnAsteroidEvent>,
    mut spawn_explosion: EventWriter<SpawnExplosionEvent>,
    mut hit_events: EventReader<HitEvent>,
    query: Query<(&Asteroid, &Movable)>
) {
    let mut rng = rand::thread_rng();
    for &HitEvent(entity) in distinct_hit_events(&mut hit_events) {
        if let Ok((asteroid, movable)) = query.get(entity) {
            // Despawn the entity
            commands.entity(entity).despawn();
            // Start the explosion
            spawn_explosion.send(make_explosion_event(&mut rng, asteroid, movable, ExplosionAssetId::AsteroidDebrisA));
            spawn_explosion.send(make_explosion_event(&mut rng, asteroid, movable, ExplosionAssetId::AsteroidDebrisB));
            spawn_explosion.send(make_explosion_event(&mut rng, asteroid, movable, ExplosionAssetId::AsteroidDebrisC));
            // Spawn asteroid chunks
            if asteroid.size == AsteroidSize::Large || asteroid.size == AsteroidSize::Medium {
                spawn_asteroid.send_batch(make_chunk_asteroids(&mut rng, asteroid, movable).into_iter());
            }
        }
    }
}

static CHILD_ASTEROID_SPAWN_DISTANCE: f32 = 4.0;
static CHILD_ASTEROID_MIN_ADD_SPEED: f32 = 50.0;
static CHILD_ASTEROID_MAX_ADD_SPEED: f32 = 150.0;

fn make_chunk_asteroids(
    rng: &mut rand::rngs::ThreadRng,
    parent_asteroid: &Asteroid,
    parent_asteroid_movable: &Movable
) -> [SpawnAsteroidEvent; 2] {

    let size = match parent_asteroid.size {
        AsteroidSize::Large => AsteroidSize::Medium,
        AsteroidSize::Medium => AsteroidSize::Small,
        AsteroidSize::Small => unreachable!(),
    };

    // Generate some random position and velocity for these two asteroids
    let chunk_direction = rng.random_unit_vec2();
    let chunk_velocity = CHILD_ASTEROID_MIN_ADD_SPEED + rng.random_f32() * (CHILD_ASTEROID_MAX_ADD_SPEED - CHILD_ASTEROID_MIN_ADD_SPEED);

    let p1 = parent_asteroid_movable.position + chunk_direction * CHILD_ASTEROID_SPAWN_DISTANCE * asteroid_scale(parent_asteroid.size);
    let p2 = parent_asteroid_movable.position + -chunk_direction * CHILD_ASTEROID_SPAWN_DISTANCE * asteroid_scale(parent_asteroid.size);

    let v1 = chunk_direction * chunk_velocity;
    let v2 = -chunk_direction * chunk_velocity;
    
    [
        SpawnAsteroidEvent(size, Some(SpawnAsteroidPositionVelocity { position: p1, velocity: v1 })),
        SpawnAsteroidEvent(size, Some(SpawnAsteroidPositionVelocity { position: p2, velocity: v2 })),
    ]
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