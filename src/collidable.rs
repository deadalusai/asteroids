use bevy::prelude::*;
use crate::movable::*;

// Component for entities which may collide (basically everything)

pub struct CollidablePlugin;

impl Plugin for CollidablePlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(CoreStage::PostUpdate, collidable_update_system);
    }
}

#[derive(Component)]
pub struct Collidable {
    pub collider: Collider,
}

impl Collidable {
    pub fn test_collision_with(&self, other: &Collidable) -> bool {
        sepax2d::sat_overlap(self.shape(), other.shape())
    }

    fn shape(&self) -> &dyn sepax2d::Shape {
        match self.collider {
            Collider::Circle(ref circle) => circle,
        }
    }
}

fn collidable_update_system(mut query: Query<(&Movable, &mut Collidable)>) {
    // Update all movable colliders with their new positions
    for (movable, mut collidable) in query.iter_mut() {
        match collidable.collider {
            Collider::Circle(ref mut circle) => {
                circle.position = movable.position.into();
            }
        }
    }
}

pub enum Collider {
    Circle(sepax2d::circle::Circle),
} 

impl Collider {
    pub fn circle(position: Vec2, radius: f32) -> Collider {
        let circle = sepax2d::circle::Circle::new(position.into(), radius);
        Collider::Circle(circle)
    }
}
