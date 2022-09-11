use std::time::Duration;

use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use crate::hit::HitEvent;
use crate::movable::*;
use crate::svg::*;

// Bullets

static BULLET_SCALE: f32 = 5.0;
static BULLET_Z: f32 = 5.0;

pub struct BulletPlugin;

impl Plugin for BulletPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(asset_initialisation_system);
        app.add_system(bullet_controller_system);
        app.add_system(bullet_collision_system);
        app.add_system_to_stage(CoreStage::PostUpdate, bullet_despawn_system);
    }
}

// Setup

struct BulletAssets {
    bullet_dimension: f32, // h of the bullet shape
    bullet_shape: Path,
}

fn asset_initialisation_system(
    mut commands: Commands
) {
    // See: https://yqnn.github.io/svg-path-editor/
    let bullet_dimension = 2.0;
    let bullet_path = "M 0 1 L 0 -1";

    commands.insert_resource(BulletAssets {
        bullet_dimension,
        bullet_shape: simple_svg_to_path(bullet_path),
    });
}

// Entity

#[derive(Component)]
pub struct Bullet {
    despawn_after: Duration,
}

/// Marker component which indicates that an entity should be considered for bullet collisions
#[derive(Component)]
pub struct BulletCollidable;

// Spawning

pub struct SpawnBullet {
    pub position: Vec2,
    pub velocity: Vec2,
    pub heading_angle: f32,
    pub despawn_after: Duration,
}

const LINE_WIDTH: f32 = 2.0;

fn spawn_bullet(
    assets: &Res<BulletAssets>,
    commands: &mut Commands,
    spawn: SpawnBullet
) {
    let scale = BULLET_SCALE;
    let bullet_color = Color::rgba(0.8, 0.8, 0.8, 1.0);
    let bullet_draw_mode = DrawMode::Stroke(StrokeMode::new(bullet_color, LINE_WIDTH / scale));

    // Transform
    let transform = Transform::default()
        .with_translation(Vec3::new(spawn.position.x, spawn.position.y, BULLET_Z))
        .with_rotation(heading_angle_to_transform_rotation(spawn.heading_angle))
        .with_scale(Vec3::splat(scale));

    // collision detection
    let convex = bevy_sepax2d::Convex::Circle(sepax2d::circle::Circle::new(spawn.position.into(), scale * assets.bullet_dimension / 2.));
    let sepax = bevy_sepax2d::components::Sepax { convex };
    let sepax_movable = bevy_sepax2d::components::Movable { axes: Vec::new() };

    commands
        .spawn()
        .insert(Bullet {
            despawn_after: spawn.despawn_after
        })
        .insert(Movable {
            position: spawn.position,
            velocity: spawn.velocity,
            acceleration: None,
            heading_angle: spawn.heading_angle,
            rotational_velocity: 0.,
            rotational_acceleration: None,
        })
        .insert(MovableTorusConstraint)
        // Rendering
        .insert_bundle(GeometryBuilder::build_as(
            &assets.bullet_shape,
            bullet_draw_mode,
            transform
        ))
        // Collision detection
        .insert(sepax)
        .insert(sepax_movable);
}

fn bullet_despawn_system(
    time: Res<Time>,
    mut commands: Commands,
    mut hit_events: EventReader<HitEvent>,
    query: Query<(Entity, &Bullet)>
) {
    // Despawn bullets which have hit something
    for &HitEvent(entity) in hit_events.iter() {
        if let Ok(_) = query.get(entity) {
            commands.entity(entity).despawn();
        }
    }

    // Despawn bullets which have expired
    let time_since_startup = time.time_since_startup();
    for (entity, bullet) in query.iter() {
        if time_since_startup >= bullet.despawn_after {
            commands.entity(entity).despawn();
        }
    }
}

// Fire control

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FireState {
    Fire,
    None
}

#[derive(Component)]
pub struct BulletController {
    timer: Timer,
    is_firing: bool,
    fire_count: i32,
    bullet_start_offset: f32,
    bullet_speed: f32,
    bullet_max_age_secs: f32,
}

impl BulletController {
    pub fn new(fire_rate: f32, bullet_start_offset: f32, bullet_speed: f32, bullet_max_age_secs: f32) -> Self {
        Self {
            is_firing: false,
            fire_count: 0,
            timer: Timer::from_seconds(1.0 / fire_rate, true),
            bullet_start_offset,
            bullet_speed,
            bullet_max_age_secs: bullet_max_age_secs,
        }
    }

    pub fn set_firing(&mut self, firing: bool) {
        if self.is_firing != firing {
            self.is_firing = firing;
            self.fire_count = 0;
            self.timer.reset();
        }
    }

    fn update(&mut self, time: &Time) -> FireState {
        if !self.is_firing {
            return FireState::None;
        }
        let should_fire = self.fire_count == 0 || self.timer.tick(time.delta()).just_finished();
        if should_fire {
            self.fire_count += 1;
            FireState::Fire
        }
        else {
            FireState::None
        }
    }
}

fn bullet_controller_system(
    time: Res<Time>,
    assets: Res<BulletAssets>,
    mut commands: Commands,
    mut query: Query<(&Movable, &mut BulletController)>
) {
    for (movable, mut controller) in query.iter_mut() {
        let fire_state = controller.update(&time);
        if fire_state == FireState::Fire {
            spawn_bullet(&assets, &mut commands, SpawnBullet {
                position: movable.position + movable.heading_normal() * controller.bullet_start_offset,
                velocity: movable.velocity + movable.heading_normal() * controller.bullet_speed,
                heading_angle: movable.heading_angle,
                despawn_after: time.time_since_startup() + Duration::from_secs(controller.bullet_max_age_secs as u64),
            });
        }
    }
}

// Collision detection

fn  bullet_collision_system(
    bullets: Query<(Entity, &bevy_sepax2d::components::Sepax), With<Bullet>>,
    collidables: Query<(Entity, &bevy_sepax2d::components::Sepax), With<BulletCollidable>>,
    mut hit_events: EventWriter<HitEvent>
)
{
    for (bullet_entity, bullet) in bullets.iter() {
        for (collidable_entity, target) in collidables.iter() {
            if sepax2d::sat_overlap(bullet.shape(), target.shape()) {
                // Collision!
                hit_events.send(HitEvent(bullet_entity));
                hit_events.send(HitEvent(collidable_entity));
            }
        }
    }
}