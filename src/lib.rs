use bevy::{prelude::*, winit::WinitSettings};
use board::BoardPlugin;

mod board;
mod domain;
mod style;

pub struct AppPlugin;

impl Plugin for AppPlugin {
  fn build(&self, app: &mut App) {
    app
      .insert_resource(WinitSettings::desktop_app())
      .add_plugins((DefaultPlugins, BoardPlugin))
      .init_state::<AppState>()
      .add_systems(OnEnter(AppState::GameOver), show_game_over_overlay)
      .add_systems(OnExit(AppState::GameOver), hide_game_over_overlay)
      .add_systems(Update, handle_restart.run_if(in_state(AppState::GameOver)));
  }
}

#[derive(States, PartialEq, Eq, Clone, Copy, Hash, Default, Debug)]
enum AppState {
  #[default]
  Playing,
  GameOver,
}

#[derive(Component)]
struct GameOverOverlay;

fn show_game_over_overlay(mut commands: Commands) {
  commands.spawn((
    GameOverOverlay,
    Node {
      width: Val::Percent(100.0),
      max_width: Val::VMin(100.0),
      aspect_ratio: Some(1.0),
      flex_direction: FlexDirection::Column,
      justify_content: JustifyContent::Center,
      align_items: AlignItems::Center,
      ..default()
    },
    children![
      (
        Text::new("GAME OVER"),
        TextLayout::new_with_justify(JustifyText::Center),
        TextColor(style::TEXT_DARK),
        TextFont {
          font_size: 96.0,
          ..default()
        }
      ),
      (
        Text::new("press any key to try again"),
        TextLayout::new_with_justify(JustifyText::Center),
        TextColor(style::TEXT_DARK),
        TextFont {
          font_size: 36.0,
          ..default()
        }
      ),
    ],
  ));
}

fn handle_restart(
  keyboard_input: Res<ButtonInput<KeyCode>>,
  mut next_state: ResMut<NextState<AppState>>,
) {
  if keyboard_input.get_pressed().next().is_some() {
    next_state.set(AppState::Playing);
  }
}

fn hide_game_over_overlay(
  query: Single<Entity, With<GameOverOverlay>>,
  mut commands: Commands,
) {
  commands.entity(*query).despawn();
}
