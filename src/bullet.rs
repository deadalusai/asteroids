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
        .insert(TorusConstraint::new(BULLET_SCALE))
        .insert_bundle(MaterialMesh2dBundle {
            mesh: assets.bullet_mesh.clone().into(),
            material: assets.bullet_material.clone(),
            transform: Transform::default()
                .with_translation(Vec3::new(spawn.position.x, spawn.position.y, BULLET_Z))
                .with_scale(Vec3::splat(BULLET_SCALE)),
            ..Default::default()
        });
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
    bullet_speed: f32,
    bullet_max_age_secs: f32,
}

impl BulletController {
    pub fn new(fire_rate: f32, bullet_speed: f32, bullet_max_age_secs: f32) -> Self {
        Self {
            is_firing: false,
            fire_count: 0,
            timer: Timer::from_seconds(1.0 / fire_rate, true),
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
        self.fire_count += 1;
        if self.fire_count == 1 {
            // Fire the first shot immediately
            FireState::Fire
        }
        else if self.timer.tick(time.delta()).just_finished() {
            // Fire any subsequent shots on the timer
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
                position: movable.position,
                velocity: movable.velocity + movable.heading_normal() * controller.bullet_speed,
                despawn_after: time.time_since_startup() + Duration::from_secs(controller.bullet_max_age_secs as u64),
            });
        }
    }
}