use bevy::prelude::*;
use crate::movable::Movable;

pub struct TorusConstraintPlugin;

impl Plugin for TorusConstraintPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(CoreStage::Update, torus_constraint_system);
    }
}

#[derive(Component)]
pub struct TorusConstraint(f32);

impl TorusConstraint {
    pub fn new(gutter_size: f32) -> Self {
        TorusConstraint(gutter_size)
    }
}

fn torus_constraint_system(
    windows: Res<Windows>,
    mut query: Query<(&TorusConstraint, &mut Movable)>
) {
    // NOTE: 0,0 is in the middle of the window.
    let win = windows.get_primary().unwrap();
    let half_width = win.width() / 2.;
    let half_height = win.height() / 2.;

    for (&TorusConstraint(gutter_size), mut movable) in query.iter_mut() {
        let right = half_width + gutter_size;
        let left = -half_width - gutter_size;
        let top = half_height + gutter_size;
        let bottom = -half_height - gutter_size;
        // Has this Movable left the screen?
        // Teleport them to the other side of the Torus
        if movable.position.x > right {
            movable.position.x = left;
        }
        if movable.position.x < left {
            movable.position.x = right;
        }
        if movable.position.y > top {
            movable.position.y = bottom;
        }
        if movable.position.y < bottom {
            movable.position.y = top;
        }
    }
}