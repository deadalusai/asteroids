use bevy::prelude::*;

use crate::AppState;

// Plugins

pub struct GameOverScreenPlugin;

impl Plugin for GameOverScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_enter(AppState::GameOver)
                .with_system(setup_game_over_system)
        );
        app.add_system_set(
            SystemSet::on_update(AppState::GameOver)
                .with_system(game_over_update_system)
        );
        app.add_system_set(
            SystemSet::on_exit(AppState::GameOver)
                .with_system(destroy_game_over_system)
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

fn setup_game_over_system(
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

    let points_text_style = TextStyle {
        font: font_light,
        font_size: 70.0,
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
                justify_content: JustifyContent::SpaceEvenly,
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
                    TextSection::new("SCORE ", points_text_style.clone()),
                    TextSection::new(format!("{}", game_results.score), TextStyle { color: Color::GOLD, ..points_text_style }),
                ])
                .with_style(margin_style.clone())
            );
        });
}

fn destroy_game_over_system(mut commands: Commands, fragments: Query<Entity, With<GameOverRoot>>) {
    commands.remove_resource::<GameResults>();
    for entity in fragments.iter() {
        commands
            .entity(entity)
            .despawn_recursive();
    }
}

fn game_over_update_system(
    kb: Res<Input<KeyCode>>,
    mut app_state: ResMut<State<AppState>>,
) {
    let should_continue =
        kb.just_pressed(KeyCode::Escape);

    if should_continue {
        app_state.replace(AppState::Menu).unwrap();
    }
}