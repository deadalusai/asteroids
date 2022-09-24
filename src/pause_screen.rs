use bevy::prelude::*;

use crate::AppState;

// Plugins

pub struct PauseScreenPlugin;

impl Plugin for PauseScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_enter(AppState::Pause)
                .with_system(pause_setup_system)
        );
        app.add_system_set(
            SystemSet::on_exit(AppState::Pause)
                .with_system(pause_cleanup_system)
        );
        app.add_system_set(
            SystemSet::on_update(AppState::Pause)
                .with_system(pause_keyboard_system)
        );
    }
}

// Components

#[derive(Component)]
struct PauseRoot;

fn pause_setup_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>
) {
    let font_light = asset_server.load("fonts/RedHatMono-Light.ttf");

    let title_text_style = TextStyle {
        font: font_light.clone(),
        font_size: 90.0,
        color: Color::WHITE,
    };

    // Root node
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                flex_direction: FlexDirection::ColumnReverse,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceEvenly,
                ..default()
            },
            color: Color::NONE.into(),
            ..default()
        })
        .insert(PauseRoot)
        .with_children(|parent| {
            // Title
            parent.spawn_bundle(
                TextBundle::from_section("PAUSE", title_text_style)
                    .with_style(Style {
                        margin: UiRect::all(Val::Px(20.0)),
                        ..default()
                    })
            );
        });
}

fn pause_cleanup_system(mut commands: Commands, fragments: Query<Entity, With<PauseRoot>>) {
    for entity in fragments.iter() {
        commands
            .entity(entity)
            .despawn_recursive();
    }
}

fn pause_keyboard_system(
    mut kb: ResMut<Input<KeyCode>>,
    mut app_state: ResMut<State<AppState>>
) {
    if kb.clear_just_pressed(KeyCode::Escape) {
        app_state.pop().unwrap();
    }
}