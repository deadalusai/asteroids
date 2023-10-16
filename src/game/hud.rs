use bevy::prelude::*;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, DiagnosticsStore};

use crate::AppState;
use super::manager::GameManager;

// Plugin

pub struct HeadsUpDisplayPlugin;

impl Plugin for HeadsUpDisplayPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FrameTimeDiagnosticsPlugin::default());
        app.add_systems(
            OnEnter(AppState::Game),
            setup_system
        );
        app.add_systems(
            Update,
            (
                status_text_update_system,
                debug_text_update_system,
            )
            .run_if(in_state(AppState::Game))
        );
        app.add_systems(
            OnExit(AppState::Game),
            destroy_system
        );
    }
}

// HUD

#[derive(Component)]
struct HudPart;

#[derive(Component)]
struct StatusText;

#[derive(Component)]
struct DebugText;

fn setup_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    
    let font_light = asset_server.load(crate::asset_paths::FONT_MONO_LIGHT);

    let status_text_bundle =
        TextBundle::from_sections([
            TextSection::new(
                "POINTS: ",
                TextStyle {
                    font: font_light.clone(),
                    font_size: 30.0,
                    color: Color::WHITE,
                },
            ),
            TextSection::from_style(TextStyle {
                font: font_light.clone(),
                font_size: 30.0,
                color: Color::GOLD,
            }),
            TextSection::new(
                " LIVES: ",
                TextStyle {
                    font: font_light.clone(),
                    font_size: 30.0,
                    color: Color::WHITE,
                },
            ),
            TextSection::from_style(TextStyle {
                font: font_light.clone(),
                font_size: 30.0,
                color: Color::GOLD,
            }),
        ])
        .with_style(Style {
            position_type: PositionType::Absolute,
            justify_content: JustifyContent::FlexEnd,
            top: Val::Px(15.0),
            left: Val::Px(15.0),
            ..default()
        });

    commands.spawn((
        StatusText,
        HudPart,
        status_text_bundle
    ));

    let debug_text_bundle =
        TextBundle::from_sections([
            TextSection::new(
                "",
                TextStyle {
                    font: font_light.clone(),
                    font_size: 15.0,
                    color: Color::WHITE,
                },
            ),
        ])
        .with_style(Style {
            position_type: PositionType::Absolute,
            justify_content: JustifyContent::FlexStart,
            bottom: Val::Px(15.0),
            right: Val::Px(15.0),
            ..default()
        });

    commands.spawn((
        DebugText,
        HudPart,
        debug_text_bundle,
    ));
}

fn destroy_system(mut commands: Commands, query: Query<Entity, With<HudPart>>) {
    for entity in query.iter() {
        commands
            .entity(entity)
            .despawn_recursive();
    }
}

fn status_text_update_system(
    game: Res<GameManager>,
    mut status_text: Query<&mut Text, With<StatusText>>
) {
    if let Some(mut status_text) = status_text.get_single_mut().ok() {
        write_u32(&mut status_text.sections[1].value, game.player_points);
        write_u32(&mut status_text.sections[3].value, game.player_lives_remaining);
    }
}

fn write_u32(output: &mut String, value: u32) {
    use std::fmt::Write;
    output.clear();
    write!(output, "{}", value).unwrap();
}

fn debug_text_update_system(
    game: Res<GameManager>,
    diag: Res<DiagnosticsStore>,
    mut debug_text: Query<&mut Text, With<DebugText>>
) {
    if let Some(mut debug_text) = debug_text.get_single_mut().ok() {
        write_debug_info(&mut debug_text.sections[0].value, &diag, &game);
    }
}

fn write_debug_info(output: &mut String, diag: &DiagnosticsStore, game: &GameManager) {
    use std::fmt::Write;
    output.clear();
    // FPS
    if let Some(fps) = diag.get(FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(avg) = fps.average() {
            writeln!(output, "fps: {avg:.2}").unwrap();
        }
    };
    // Game state
    writeln!(output, "asteroids on screen: {}", game.debug_asteroid_count_on_screen).unwrap();
    writeln!(output, "asteroids pending spawn: {}", game.scheduled_asteroid_spawns.len()).unwrap();
    writeln!(output, "player state: {:?}", game.player_state).unwrap();
    writeln!(output, "alien state: {:?}", game.alien_state).unwrap();
}