use bevy::prelude::*;

use crate::AppState;
use super::manager::GameManager;

// Plugin

pub struct HeadsUpDisplayPlugin;

impl Plugin for HeadsUpDisplayPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_enter(AppState::Game)
                .with_system(setup_system)
        );
        app.add_system_set(
            SystemSet::on_update(AppState::Game)
                .with_system(status_text_update_system)
                .with_system(debug_text_update_system)
        );
    }
}

// HUD

#[derive(Component)]
struct StatusText;

#[derive(Component)]
struct DebugText;

fn setup_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    
    let font_dotgothic16 = asset_server.load("fonts/RedHatMono-Light.ttf");
    
    let status_text_bundle =
        TextBundle::from_sections([
            TextSection::new(
                "POINTS: ",
                TextStyle {
                    font: font_dotgothic16.clone(),
                    font_size: 30.0,
                    color: Color::WHITE,
                },
            ),
            TextSection::from_style(TextStyle {
                font: font_dotgothic16.clone(),
                font_size: 30.0,
                color: Color::GOLD,
            }),
            TextSection::new(
                " LIVES: ",
                TextStyle {
                    font: font_dotgothic16.clone(),
                    font_size: 30.0,
                    color: Color::WHITE,
                },
            ),
            TextSection::from_style(TextStyle {
                font: font_dotgothic16.clone(),
                font_size: 30.0,
                color: Color::GOLD,
            }),
        ])
        .with_style(Style {
            align_self: AlignSelf::FlexEnd,
            position: UiRect {
                top: Val::Px(5.0),
                left: Val::Px(15.0),
                ..default()
            },
            ..default()
        });
    
    commands
        .spawn()
        .insert(StatusText)
        .insert_bundle(status_text_bundle);

    
    let debug_text_bundle =
        TextBundle::from_sections([
            TextSection::new(
                "",
                TextStyle {
                    font: font_dotgothic16.clone(),
                    font_size: 15.0,
                    color: Color::WHITE,
                },
            ),
        ])
        .with_style(Style {
            align_self: AlignSelf::FlexEnd,
            position_type: PositionType::Absolute,
            position: UiRect {
                bottom: Val::Px(5.0),
                right: Val::Px(10.0),
                ..default()
            },
            ..default()
        });

    commands
        .spawn()
        .insert(DebugText)
        .insert_bundle(debug_text_bundle);
}

fn status_text_update_system(
    game: Res<GameManager>,
    mut status_text: Query<&mut Text, With<StatusText>>
) {
    let mut status_text = status_text.get_single_mut().unwrap();
    status_text.sections[1].value = format!("{}", game.player_points);
    status_text.sections[3].value = format!("{}", game.player_lives_remaining);
}

fn debug_text_update_system(
    game: Res<GameManager>,
    mut debug_text: Query<&mut Text, With<DebugText>>
) {
    let mut debug_text = debug_text.get_single_mut().unwrap();
    let debug_s = &mut debug_text.sections[0].value;
    debug_s.clear();
    write_debug_info_text(&game, debug_s).unwrap();
}

fn write_debug_info_text(game: &GameManager, w: &mut impl std::fmt::Write) -> Result<(), std::fmt::Error> {
    writeln!(w, "asteroids on screen: {}", game.debug_asteroid_count_on_screen)?;
    writeln!(w, "asteroids pending spawn: {}", game.scheduled_asteroid_spawns.len())?;
    writeln!(w, "player state: {:?}", game.player_state)?;
    Ok(())
}