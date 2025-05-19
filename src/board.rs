// TODO: check game over

use bevy::{
  app::Plugin,
  ecs::{
    relationship::RelatedSpawner,
    spawn::{SpawnIter, SpawnWith},
  },
  prelude::*,
};

use crate::{
  domain::{Board, Direction, TileAction},
  style,
};

pub struct BoardPlugin;

impl Plugin for BoardPlugin {
  fn build(&self, app: &mut App) {
    app
      .insert_resource(BoardRes(Board::empty()))
      .add_event::<BoardShifted>()
      .add_event::<TileAnimated>()
      .add_event::<TileSpawned>()
      .add_event::<RedrawRequested>()
      .add_systems(Startup, setup)
      .add_systems(Update, (handle_input, update_board, animate_tiles).chain())
      .add_observer(redraw_board);
  }
}

const SIZE: usize = 4;

#[derive(Resource)]
struct BoardRes(Board<SIZE>);

#[derive(Component)]
struct Grid;

#[derive(Component)]
struct BoardTile;

#[derive(Event)]
struct BoardShifted(Direction);

#[derive(Event)]
struct TileAnimated(TileAction);

#[derive(Event)]
struct TileSpawned(usize, usize);

#[derive(Event)]
struct RedrawRequested;

fn setup(mut board_res: ResMut<BoardRes>, mut commands: Commands) {
  let board = Board::<SIZE>::new();
  commands.spawn(Camera2d);
  commands.spawn(grid(&board));
  board_res.0 = board;
}

fn grid(board: &Board<SIZE>) -> impl Bundle {
  let nums = board.iter_numbers().collect::<Vec<_>>();
  (
    Grid,
    Node {
      width: Val::Percent(100.0),
      max_width: Val::VMin(100.0),
      aspect_ratio: Some(1.0),
      display: Display::Grid,
      grid_template_columns: RepeatedGridTrack::flex(SIZE as u16, 1.0),
      grid_template_rows: RepeatedGridTrack::flex(SIZE as u16, 1.0),
      padding: UiRect::all(Val::VMin(3.0)),
      row_gap: Val::VMin(3.0),
      column_gap: Val::VMin(3.0),
      ..default()
    },
    BackgroundColor(style::GRID),
    Children::spawn(SpawnIter(nums.into_iter().map(tile))),
  )
}

fn tile(n: u8) -> impl Bundle {
  (
    BoardTile,
    Node {
      height: Val::Percent(100.0),
      width: Val::Percent(100.0),
      flex_direction: FlexDirection::Column,
      justify_content: JustifyContent::Center,
      align_items: AlignItems::Center,
      ..default()
    },
    BackgroundColor(style::tile_foreground(n)),
    Children::spawn(SpawnWith(move |parent: &mut RelatedSpawner<ChildOf>| {
      if n > 0 {
        parent.spawn((
          Text::new(if n > 0 {
            2u32.pow(n as u32).to_string()
          } else {
            "".to_string()
          }),
          TextFont {
            font_size: 56.0,
            ..default()
          },
          // TODO: add black/white text color (as `palette` function)
        ));
      }
    })),
  )
}

fn redraw_board(
  _redraw_trigger: Trigger<RedrawRequested>, // TODO: refactor into subsystem call
  board: Res<BoardRes>,
  grid: Single<Entity, With<Grid>>,
  mut commands: Commands,
) {
  let tiles = board
    .0
    .iter_numbers()
    .map(|n| commands.spawn(tile(n)).id())
    .collect::<Vec<_>>();
  commands
    .entity(*grid)
    .despawn_related::<Children>()
    .replace_children(&tiles);
}

fn handle_input(
  keyboard_input: Res<ButtonInput<KeyCode>>,
  mut events: EventWriter<BoardShifted>,
) {
  for (key, dir) in [
    (KeyCode::ArrowDown, Direction::Down),
    (KeyCode::ArrowUp, Direction::Up),
    (KeyCode::ArrowLeft, Direction::Left),
    (KeyCode::ArrowRight, Direction::Right),
  ] {
    if keyboard_input.just_pressed(key) {
      events.write(BoardShifted(dir));
    }
  }
}

fn update_board(
  mut board_events: EventReader<BoardShifted>,
  mut tile_animated_events: EventWriter<TileAnimated>,
  mut tile_spawned_events: EventWriter<TileSpawned>,
  mut board_res: ResMut<BoardRes>,
) {
  let mut actions = Vec::new();
  let mut spawns = Vec::new();
  for e in board_events.read() {
    let new_actions = board_res.0.shift(e.0);
    if new_actions.is_empty() {
      continue;
    }
    if let Some(coords) = board_res.0.spawn() {
      spawns.push(coords);
    }
    actions.extend(new_actions);
  }
  tile_animated_events.write_batch(actions.into_iter().map(TileAnimated));
  tile_spawned_events
    .write_batch(spawns.into_iter().map(|(x, y)| TileSpawned(x, y)));
}

fn animate_tiles(
  mut tile_events: EventReader<TileAnimated>,
  mut commands: Commands,
) {
  if tile_events.is_empty() {
    return;
  }
  for e in tile_events.read() {
    // TODO: handle animation
  }
  commands.trigger(RedrawRequested);
}
