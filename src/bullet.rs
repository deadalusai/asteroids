use std::time::Duration;

use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;
use crate::movable::*;
use crate::torus::*;

// Bullets

static BULLET_SCALE: f32 = 5.0;
static BULLET_Z: f32 = 5.0;

pub struct BulletPlugin;

impl Plugin for BulletPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(asset_initialisation_system);
        app.add_system(bullet_controller_system);
        app.add_system(bullet_despawn_system);
        app.add_system(bullet_collision_system);
    }
}

// Setup

struct BulletAssets {
    bullet_mesh: Handle<Mesh>,
    bullet_material: Handle<ColorMaterial>,
}

fn asset_initialisation_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>
) {
    commands.insert_resource(BulletAssets {
        bullet_mesh: meshes.add(Mesh::from(shape::Circle::new(1.0))),
        bullet_material: materials.add(ColorMaterial::from(Color::rgba(1., 1., 1., 1.))),
    });
}

// Entity

#[derive(Component)]
pub struct Bullet {
    despawn_after: Duration,
}

// Spawning

pub struct SpawnBullet {
    pub position: Vec2,
    pub velocity: Vec2,
    pub despawn_after: Duration,
}

fn spawn_bullet(
    assets: &Res<BulletAssets>,
    commands: &mut Commands,
    spawn: SpawnBullet
) {

    // collision detection
    let convex = bevy_sepax2d::Convex::Circle(sepax2d::circle::Circle::new(spawn.position.into(), BULLET_SCALE / 2.));
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
            heading_angle: 0.,
            rotational_velocity: 0.,
            rotational_acceleration: None,
        })
        .insert(TorusConstraint)
        .insert_bundle(MaterialMesh2dBundle {
            mesh: assets.bullet_mesh.clone().into(),
            material: assets.bullet_material.clone(),
            transform: Transform::default()
                .with_translation(Vec3::new(spawn.position.x, spawn.position.y, BULLET_Z))
                .with_scale(Vec3::splat(BULLET_SCALE)),
            ..Default::default()
        })
        // Collision detection
        .insert(sepax)
        .insert(sepax_movable);
}

fn bullet_despawn_system(
    time: Res<Time>,
    mut commands: Commands,
    query: Query<(Entity, &Bullet)>
) {
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
                despawn_after: time.time_since_startup() + Duration::from_secs(controller.bullet_max_age_secs as u64),
            });
        }
    }
}

// Collision detection

fn  bullet_collision_system(
    mut commands: Commands,
    bullets: Query<(Entity, &bevy_sepax2d::components::Sepax), With<Bullet>>,
    targets: Query<(Entity, &bevy_sepax2d::components::Sepax), Without<Bullet>>
)
{
    for (bullet_entity, bullet) in bullets.iter() {
        for (target_entity, target) in targets.iter() {
            if sepax2d::sat_overlap(bullet.shape(), target.shape()) {
                // Collision!
                commands.entity(bullet_entity).despawn();
                commands.entity(target_entity).despawn();
            }
        }
    }
}