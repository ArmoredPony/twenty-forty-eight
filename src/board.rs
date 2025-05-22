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
      .add_event::<BoardShifted>()
      .add_event::<TileAnimated>()
      .add_systems(Startup, setup)
      .add_systems(OnEnter(AppState::Playing), restart)
      .add_systems(
        Update,
        (handle_input, shift_board, assign_animations)
          .chain()
          .run_if(player_can_interact())
          .before(animate_tiles),
      )
      .add_systems(Update, animate_tiles.run_if(animating))
      .add_systems(
        Update,
        (
          redraw_board.run_if(on_event::<BoardShifted>),
          check_game_over,
        )
          .chain()
          .run_if(player_can_interact())
          .after(animate_tiles),
      );
  }
}

const SIZE: usize = 4;

#[derive(Resource)]
struct BoardRes(Board<SIZE>);

#[derive(Component)]
struct Grid;

#[derive(Component)]
struct Tile;

#[derive(Component)]
enum Animation {
  Move {
    dir: Direction,
    tiles_to_move: f32,
    tiles_to_move_left: f32, // zero when animation is finished
  },
  Merge {
    value: u8,
    dir: Direction,
    tiles_to_move: f32,      // zero when animation is finished
    tiles_to_move_left: f32, // zero when animation is finished
  },
}

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

fn setup(mut commands: Commands) {
  commands.spawn(Camera2d);
  commands.run_system_cached(restart);
}

fn restart(
  mut board_res: ResMut<BoardRes>,
  old_grid: Query<Option<Entity>, With<Grid>>,
  mut commands: Commands,
) {
  if let Ok(Some(grid)) = old_grid.single() {
    commands.entity(grid).despawn();
  }
  let board = Board::<SIZE>::new();
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
    Tile,
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
  mut commands: Commands,
) {
  if keyboard_input.just_pressed(KeyCode::KeyR) {
    commands.run_system_cached(restart);
    return;
  }
  for (key, dir) in [
    (KeyCode::ArrowUp, Direction::Up),
    (KeyCode::ArrowDown, Direction::Down),
    (KeyCode::ArrowLeft, Direction::Left),
    (KeyCode::ArrowRight, Direction::Right),
    (KeyCode::KeyW, Direction::Up),
    (KeyCode::KeyS, Direction::Down),
    (KeyCode::KeyA, Direction::Left),
    (KeyCode::KeyD, Direction::Right),
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

fn assign_animations(
  mut tile_animated_events: EventReader<TileAnimated>,
  tiles: Single<&Children, With<Grid>>,
  mut commands: Commands,
) {
  for e in tile_animated_events.read() {
    let (row, col): (usize, usize);
    let anim = match e {
      TileAnimated::Moved { from, to, .. } => {
        (row, col) = *from;
        let dir = direction_from_position(from, to);
        let tiles_to_move =
          from.0.abs_diff(to.0).max(from.1.abs_diff(to.1)) as f32;
        Animation::Move {
          dir,
          tiles_to_move,
          tiles_to_move_left: tiles_to_move,
        }
      }
      TileAnimated::Merged { value, from, at } => {
        (row, col) = *from;
        let dir = direction_from_position(from, at);
        let tiles_to_move =
          from.0.abs_diff(at.0).max(from.1.abs_diff(at.1)) as f32;
        Animation::Merge {
          value: *value,
          dir,
          tiles_to_move,
          tiles_to_move_left: tiles_to_move,
        }
      }
      TileAnimated::Spawned { value, at } => {
        (row, col) = *at;
        return;
      }
    };
    let tile = tiles.get(row * SIZE + col).expect("tile out of bounds");
    commands.entity(*tile).insert(anim);
  }
}

fn direction_from_position(
  from: &(usize, usize),
  to: &(usize, usize),
) -> Direction {
  use std::cmp::Ordering::*;

  match to.0.cmp(&from.0) {
    Greater => Direction::Down,
    Less => Direction::Up,
    Equal => match to.1.cmp(&from.1) {
      Greater => Direction::Right,
      Less => Direction::Left,
      Equal => unreachable!("move event without position change"),
    },
  }
}

fn animate_tiles(
  time: Res<Time>,
  animated_tiles: Query<(&mut Transform, &Animation, &Node), With<Tile>>,
) {
  for (mut trans, anim, node) in animated_tiles {
    trans.translation.x += time.delta_secs() * 1000.0;
    println!("{}", trans.translation.x);
  }
}

fn animating(animated_tiles: Query<(&Tile, &Animation)>) -> bool {
  !animated_tiles.is_empty()
}

fn player_can_interact() -> impl Condition<()> {
  in_state(AppState::Playing).and(not(animating))
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
