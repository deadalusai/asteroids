use bevy::prelude::*;

use crate::AppState;

// Plugins

pub struct GameOverScreenPlugin;

impl Plugin for GameOverScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_enter(AppState::GameOver)
                .with_system(game_over_setup_system)
        );
        app.add_system_set(
            SystemSet::on_update(AppState::GameOver)
                .with_system(game_over_keyboard_system)
        );
        app.add_system_set(
            SystemSet::on_exit(AppState::GameOver)
                .with_system(game_over_cleanup_system)
        );
    }
}

// Resources

pub struct GameResults {
    pub score: u32,
}

// Components

#[derive(Component)]
struct GameOverRoot;

fn game_over_setup_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    game_results: Res<GameResults>
) {
    let font_light = asset_server.load("fonts/RedHatMono-Light.ttf");

    let margin_style = Style {
        margin: UiRect::all(Val::Px(20.0)),
        align_self: AlignSelf::Center,
        ..default()
    };

    let title_text_style = TextStyle {
        font: font_light.clone(),
        font_size: 90.0,
        color: Color::WHITE,
    };

    let secondary_text_style = TextStyle {
        font: font_light,
        font_size: 50.0,
        color: Color::GRAY,
    };

    let background = Color::rgba(0.0, 0.0, 0.0, 0.2);

    // Root node
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                flex_direction: FlexDirection::ColumnReverse,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            color: background.into(),
            ..default()
        })
        .insert(GameOverRoot)
        .with_children(|parent| {
            // Title
            parent.spawn_bundle(
                TextBundle::from_section("GAME OVER", title_text_style)
                .with_style(margin_style.clone())
            );
            // Score
            parent.spawn_bundle(
                TextBundle::from_sections([
                    TextSection::new("SCORE ", secondary_text_style.clone()),
                    TextSection::new(game_results.score.to_string(), ts_with_color(&secondary_text_style, Color::GOLD)),
                ])
                .with_style(margin_style.clone())
            );
            parent.spawn_bundle(
                TextBundle::from_section("Press [esc] to continue", secondary_text_style)
                .with_style(margin_style.clone())
            );
        });
}

fn ts_with_color(ts: &TextStyle, color: Color) -> TextStyle {
    TextStyle { color, ..ts.clone() }
}

fn game_over_cleanup_system(mut commands: Commands, fragments: Query<Entity, With<GameOverRoot>>) {
    commands.remove_resource::<GameResults>();
    for entity in fragments.iter() {
        commands
            .entity(entity)
            .despawn_recursive();
    }
}

fn game_over_keyboard_system(
    mut kb: ResMut<Input<KeyCode>>,
    mut app_state: ResMut<State<AppState>>,
) {
    if kb.clear_just_released(KeyCode::Escape) {
        app_state.replace(AppState::Menu).unwrap();
    }
}