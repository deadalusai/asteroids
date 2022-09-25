use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use crate::AppState;
use super::FrameStage;
use super::assets::GameAssets;
use super::hit::{HitEvent, distinct_hit_events};
use super::movable::{Movable, MovableTorusConstraint};
use super::collidable::{Collidable, Collider};
use super::explosion::{ExplosionShapeId, SpawnExplosion, spawn_explosion};
use super::bullet::{BulletController, BulletCollidable};
use super::svg::simple_svg_to_path;

// Player's Rocket

const ALIEN_BULLET_SPEED: f32 = 250.0;
const ALIEN_BULLET_MAX_AGE_SECS: f32 = 0.8;
const ALIEN_FIRE_RATE: f32 = 0.5; // per second
const ALIEN_Z: f32 = 10.0;

pub struct AlienPlugin;

impl Plugin for AlienPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<AlienUfoDestroyedEvent>();
        app.add_system_set(
            SystemSet::on_update(AppState::Game)
                .with_system(
                    alien_hit_system
                        .label(FrameStage::CollisionEffect)
                        .after(FrameStage::Collision)
                )
        );
        app.add_system_set(
            SystemSet::on_exit(AppState::Game)
                .with_system(alien_teardown_system)
        );
    }
}

// Events

pub struct AlienUfoDestroyedEvent;

// Setup

pub struct AlienAssets {
    alien_ufo_dimension: (f32, f32), // (w, h) of the rocket shape
    alien_ufo_shape: Path,
}

pub fn create_alien_assets() -> AlienAssets {
    // See: https://yqnn.github.io/svg-path-editor/
    let alien_ufo_dimension = (10.0, 6.0);
    let alien_ufo_path = "
        M 5 0
        C 2 -2 -2 -2 -5 0
        L -5 1
        C -2 3 2 3 5 1 L 5 0
        M 3 -1
        Q 0 -5 -3 -1
    ";

    AlienAssets {
        alien_ufo_dimension,
        alien_ufo_shape: simple_svg_to_path(alien_ufo_path),
    }
}

// Teardown

fn alien_teardown_system(mut commands: Commands, query: Query<Entity, With<AlienUfo>>) {
    for entity in query.iter() {
        commands
            .entity(entity)
            .despawn_recursive();
    }
}

// Entity

#[derive(Component, Default)]
pub struct AlienUfo;

// Spawning

#[derive(Clone)]
pub struct AlienSpawn {
    pub position: Vec2,
    pub velocity: Vec2,
}

const LINE_WIDTH: f32 = 0.2;

pub fn spawn_alien_ufo(
    commands: &mut Commands,
    assets: &AlienAssets,
    spawn: AlienSpawn
) {
     // Spawn stationary, in the middle of the screen
    let position = spawn.position;
    let velocity = spawn.velocity;
    let (width, height) = assets.alien_ufo_dimension;

    // Rocket
    let alien_color = Color::rgba(1., 1., 1., 1.);
    let alien_draw_mode = DrawMode::Stroke(StrokeMode::new(alien_color, LINE_WIDTH));

    // Transform
    let transform = Transform::from_translation(Vec3::new(position.x, position.y, ALIEN_Z));
    
    // Collision detection
    let arm = Vec2::new(width / 2., 0.);
    let radius = height / 2.;
    let collider = Collider::capsule(position, arm, radius);

    commands
        .spawn()
        .insert(AlienUfo)
        .insert(Movable {
            position,
            velocity,
            acceleration: None,
            heading_angle: 0.0,
            rotational_velocity: 0.,
            rotational_acceleration: None,
        })
        .insert(MovableTorusConstraint { radius })
        .insert(BulletController::new(ALIEN_FIRE_RATE, radius, ALIEN_BULLET_SPEED, ALIEN_BULLET_MAX_AGE_SECS))
        // Collision detection
        .insert(BulletCollidable)
        .insert(Collidable { collider })
        // Rendering
        .insert_bundle(GeometryBuilder::build_as(
            &assets.alien_ufo_shape,
            alien_draw_mode,
            transform
        ));
}

// Destruction system

static PLAYER_ALIEN_EXPLOSION_DESPAWN_AFTER_SECS: f32 = 3.0;

fn alien_hit_system(
    mut commands: Commands,
    mut hit_events: EventReader<HitEvent>,
    mut alien_destroyed: EventWriter<AlienUfoDestroyedEvent>,
    assets: Res<GameAssets>,
    query: Query<&Movable, With<AlienUfo>>
) {
    for &HitEvent(entity) in distinct_hit_events(&mut hit_events) {
        if let Ok(movable) = query.get(entity) {
            let mut rng = rand::thread_rng();
            // Despawn the entity
            commands.entity(entity).despawn_recursive();
            // Start the explosion
            spawn_explosion(&mut commands, &mut rng, &assets.explosion, SpawnExplosion {
                shape_id: ExplosionShapeId::AsteroidDebris,
                shape_scale: 1.0,
                position: movable.position,
                velocity: movable.velocity,
                heading_angle: movable.heading_angle,
                rotational_velocity: movable.rotational_velocity,
                despawn_after_secs: PLAYER_ALIEN_EXPLOSION_DESPAWN_AFTER_SECS,
            });
            // Send events
            alien_destroyed.send(AlienUfoDestroyedEvent);
        }
    }
}