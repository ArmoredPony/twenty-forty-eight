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
      .add_plugins((DefaultPlugins, BoardPlugin));
  }
}

#[derive(States, PartialEq, Eq, Clone, Copy, Hash, Default, Debug)]
enum AppState {
  #[default]
  Playing,
  GameOver,
}
