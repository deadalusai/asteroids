use bevy::prelude::*;
use crate::movable::Movable;

pub struct TorusConstraintPlugin;

impl Plugin for TorusConstraintPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(CoreStage::Update, torus_constraint_system);
    }
}

#[derive(Component)]
pub struct TorusConstraint;

static TORUS_GUTTER: f32 = 25.0;

fn torus_constraint_system(
    windows: Res<Windows>,
    mut query: Query<&mut Movable, With<TorusConstraint>>
) {
    // NOTE: 0,0 is in the middle of the window.
    let win = windows.get_primary().unwrap();
    let half_width = win.width() / 2.;
    let half_height = win.height() / 2.;

    for mut movable in query.iter_mut() {
        let right = half_width + TORUS_GUTTER;
        let left = -half_width - TORUS_GUTTER;
        let top = half_height + TORUS_GUTTER;
        let bottom = -half_height - TORUS_GUTTER;
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