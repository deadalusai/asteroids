use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy_prototype_lyon::prelude::*;
use rand::thread_rng;
use crate::AppState;
use super::FrameStage;
use super::assets::GameAssets;
use super::bullet::{BulletCollidable, BulletSource};
use super::hit::{HitEvent, distinct_hit_events};
use super::invulnerable::{Invulnerable, TestInvulnerable};
use super::manager::GameCleanup;
use super::movable::{Movable, MovableTorusConstraint};
use super::collidable::{Collidable, Collider};
use super::explosion::{ExplosionShapeId, SpawnExplosion, spawn_explosion};
use super::svg::simple_svg_to_path;

pub struct AsteroidPlugin;

impl Plugin for AsteroidPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<AsteroidDestroyedEvent>();
        app.add_systems(Update, 
            asteroid_collision_system
                .in_set(FrameStage::Collision)
                .after(FrameStage::Movement)
                .run_if(in_state(AppState::Game))
        );
        app.add_systems(Update, 
            asteroid_hit_system
                .in_set(FrameStage::CollisionEffect)
                .after(FrameStage::Collision)
                .run_if(in_state(AppState::Game))
        );
        app.add_systems(GameCleanup, destroy_asteroids_system);
    }
}

// Events

#[derive(Clone, Event)]
pub struct AsteroidDestroyedEvent {
    pub size: AsteroidSize,
    pub position: Vec2,
    pub velocity: Vec2,
}

// Setup

pub struct AsteroidAssets {
    asteroid_shapes: HashMap<AsteroidShapeId, (f32, Path)>,
}

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub enum AsteroidShapeId { A, B, C, D, E }

impl AsteroidShapeId {
    pub const VALUES: [Self; 5] = [ Self::A, Self::B, Self::C, Self::D, Self::E ];
}

pub fn create_asteroid_assets() -> AsteroidAssets {
    let asteroid_shapes = vec![
        // diameter, path
        // See: https://yqnn.github.io/svg-path-editor/
        (AsteroidShapeId::A, 10.0, "M -2 -5 L -5 -2 L -5 0 L -2 0 L -5 2 L -2 5 L 3 4 L 2 2 L 5 0 L 4 -2 L 1 -5 Z"),
        (AsteroidShapeId::B, 10.0, "M -5 -3 L -5 2 L -3 5 L 2 5 L 5 3 L 4 1 L 6 -2 L 4 -5 L 1 -3 L -2 -6 Z"),
        (AsteroidShapeId::C, 10.0, "M 4 -3 L 0 -5 L -3 -5 L -2 -2 L -5 -2 L -5 0 L -2 5 L 1 2 L 2 4 L 5 1 L 1 -1 L 5 -2 L 5 -3 Z"),
        (AsteroidShapeId::D, 10.0, "M 0 -5 L -4 -4 L -5 -1 L -3 0 L -4 2 L -2 4 L 1 4 L 2 5 L 4 4 L 5 1 L 2 0 L 5 -1 L 5 -3 L 4 -5 L 1 -4 L 0 -5"),
        (AsteroidShapeId::E, 10.0, "M 5 0 L 4 -3 L 2 -4 L 1 -3 L 0 -3 L -2 -4 L -4 -3 L -5 0 L -3 1 L -3 3 L -1 3 L 0 5 L 2 3 L 4 3 L 4 1 L 5 0"),
    ];
    let asteroid_shapes = asteroid_shapes.into_iter()
        .map(|(id, dim, svg)| (id, (dim, simple_svg_to_path(svg))))
        .collect();

    AsteroidAssets { asteroid_shapes }
}

// Teardown

fn destroy_asteroids_system(mut commands: Commands, query: Query<Entity, With<Asteroid>>) {
    for entity in query.iter() {
        commands
            .entity(entity)
            .despawn_recursive();
    }
}

// Asteroids

static ASTEROID_Z: f32 = 20.0;
static ASTEROID_SMALL_SCALE: f32 = 1.0;
static ASTEROID_MEDIUM_SCALE: f32 = 2.0;
static ASTEROID_LARGE_SCALE: f32 = 3.0;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AsteroidSize {
    Small, Medium, Large
}

impl AsteroidSize {
    pub const VALUES: [Self; 3] = [ Self::Large, Self::Medium, Self::Small ];
}

#[derive(Component)]
pub struct Asteroid {
    size: AsteroidSize,
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

#[derive(Clone)]
pub struct AsteroidSpawn {
    pub size: AsteroidSize,
    pub shape: AsteroidShapeId,
    pub position: Vec2,
    pub velocity: Vec2,
    pub rotation: f32,
    pub invulnerable: Option<Timer>,
}

pub fn spawn_asteroid(
    commands: &mut Commands,
    assets: &AsteroidAssets,
    spawn: AsteroidSpawn
) {
    let position = spawn.position;
    let velocity = spawn.velocity;
    let rotation = spawn.rotation;

    // Mesh
    let (diameter, shape) = &assets.asteroid_shapes[&spawn.shape];
    let scale = asteroid_scale(spawn.size);

    let color = Color::rgba(0.6, 0.6, 0.6, 1.);
    let stroke = Stroke::new(color, LINE_WIDTH / scale);
    let transform = Transform::default()
        .with_translation(Vec3::new(position.x, position.y, ASTEROID_Z))
        .with_scale(Vec3::splat(scale));

    // Collision detection
    let radius = scale * diameter / 2.;
    let collider = Collider::circle(position.into(), radius);

    let entity = commands
        .spawn((
            Asteroid {
                size: spawn.size,
            },
            Movable {
                position,
                velocity,
                acceleration: None,
                heading_angle: 0.,
                rotational_velocity: rotation * std::f32::consts::TAU,
                rotational_acceleration: None,
            },
            MovableTorusConstraint { radius },
            // Render
            ShapeBundle {
                path: Path(shape.0.clone()),
                transform,
                ..default()
            },
            stroke,
            // Collision detection
            Collidable { collider },
            BulletCollidable { source: BulletSource::PlayerRocket }
        ))
        .id();

    if let Some(timer) = spawn.invulnerable {
        commands
            .entity(entity)
            .insert(Invulnerable::new(timer));
    }
}

// Collision detection

fn  asteroid_collision_system(
    asteroids: Query<(Entity, &Collidable), With<Asteroid>>,
    collidables: Query<(Entity, &Collidable, Option<&Invulnerable>), With<AsteroidCollidable>>,
    mut hit_events: EventWriter<HitEvent>
)
{
    for (asteroid, bullet) in asteroids.iter() {
        for (other, target, invulnerable) in collidables.iter() {
            if invulnerable.is_invulnerable() {
                continue;
            }
            if bullet.test_collision_with(&target) {
                // Collision!
                hit_events.send(HitEvent(asteroid));
                hit_events.send(HitEvent(other));
            }
        }
    }
}

// Hit handling system

const ASTEROID_EXPLOSION_DESPAWN_AFTER_SECS: f32 = 0.8;

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
            spawn_explosion(&mut commands, &mut rng, &assets.explosion, SpawnExplosion {
                shape_id: ExplosionShapeId::AsteroidDebris,
                shape_scale: asteroid_scale(asteroid.size),
                position: movable.position,
                velocity: movable.velocity,
                heading_angle: movable.heading_angle,
                rotational_velocity: movable.rotational_velocity,
                despawn_after_secs: ASTEROID_EXPLOSION_DESPAWN_AFTER_SECS,
            });
            // Send events
            asteroid_destroyed.send(AsteroidDestroyedEvent {
                size: asteroid.size,
                position: movable.position,
                velocity: movable.velocity
            });
        }
    }
}
