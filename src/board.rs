use bevy::{
  app::Plugin,
  ecs::{
    relationship::RelatedSpawner,
    spawn::{SpawnIter, SpawnWith},
  },
  prelude::*,
};

use crate::{
  AppState,
  domain::{Board, Direction, TileAction, TileActionKind},
  style,
};

pub struct BoardPlugin;

impl Plugin for BoardPlugin {
  fn build(&self, app: &mut App) {
    app
      .insert_resource(BoardRes(Board::empty()))
      .init_state::<AppState>()
      .add_event::<BoardShifted>()
      .add_event::<TileAnimated>()
      .add_systems(Startup, setup)
      .add_systems(
        Update,
        (
          check_game_over,
          handle_input,
          shift_board,
          animate_tiles,
          redraw_board.run_if(on_event::<BoardShifted>),
        )
          .chain()
          .run_if(in_state(AppState::Playing)),
      );
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
enum TileAnimated {
  Moved {
    value: u8,
    from: (usize, usize),
    to: (usize, usize),
  },
  Merged {
    value: u8,
    from: (usize, usize),
    at: (usize, usize),
  },
  Spawned {
    value: u8,
    at: (usize, usize),
  },
}

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
          Text::new(2u32.pow(n as u32).to_string()),
          TextFont {
            font_size: 56.0,
            ..default()
          },
          TextColor(style::tile_text(n)),
        ));
      }
    })),
  )
}

fn check_game_over(
  board_res: Res<BoardRes>,
  mut next_state: ResMut<NextState<AppState>>,
) {
  if !board_res.0.is_shiftable() {
    next_state.set(AppState::GameOver);
  }
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

fn shift_board(
  mut board_res: ResMut<BoardRes>,
  mut board_events: EventReader<BoardShifted>,
  mut tile_animated_events: EventWriter<TileAnimated>,
) {
  let Some(event) = board_events.read().next() else {
    return;
  };
  let actions = board_res.0.shift(event.0);
  if actions.is_empty() {
    return;
  }
  tile_animated_events.write_batch(actions.into_iter().map(|a: TileAction| {
    match a.kind {
      TileActionKind::Move => TileAnimated::Moved {
        value: a.value,
        from: a.from,
        to: a.to,
      },
      TileActionKind::Merge => TileAnimated::Merged {
        value: a.value,
        from: a.from,
        at: a.to,
      },
    }
  }));
  if let Some((value, coords)) = board_res.0.spawn() {
    tile_animated_events.write(TileAnimated::Spawned { value, at: coords });
  }
}

fn animate_tiles(mut tile_animated_events: EventReader<TileAnimated>) {
  // TODO: handle animation
}

fn redraw_board(
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
