use bevy::prelude::*;

use crate::game::Game;

// Plugin

pub struct HeadsUpDisplayPlugin;

impl Plugin for HeadsUpDisplayPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_system);
        app.add_system(text_update_system);
    }
}

// HUD

#[derive(Component)]
struct StatusText;

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
}

fn text_update_system(
    game: Res<Game>,
    mut status_text: Query<&mut Text, With<StatusText>>,
) {
    let mut text = status_text.get_single_mut().unwrap();
    text.sections[1].value = format!("{}", game.player_points);
    text.sections[3].value = format!("{}", game.player_lives_remaining);
}