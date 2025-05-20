use rand::prelude::*;

/// The grid shift direction.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Direction {
  Up,
  Down,
  Left,
  Right,
}

/// An implementation of 2048 the game.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Board<const N: usize>([[u8; N]; N]);

impl<const N: usize> Board<N> {
  const TWO_TO_FOUR_SPAWN_CHANCE: f64 = 90.0; // %

  /// Creates an empty 2048 board.
  pub fn empty() -> Self {
    Self([[0; N]; N])
  }

  /// Creates an new 2048 board and [`spawn`](Self::spawn)s two numbers on it.
  pub fn new() -> Self {
    let mut board = Self::empty();
    board.spawn();
    board.spawn();
    board
  }

  /// Returns the size of the board's side.
  pub fn size(&self) -> usize {
    N
  }

  /// Returns a flat iterator over board's numbers.
  pub fn iter_numbers(&self) -> impl Iterator<Item = u8> {
    self.0.iter().flatten().cloned()
  }

  /// Returns a value from the board.
  pub fn get(&self, row: usize, col: usize) -> u8 {
    self.0[row][col]
  }

  /// Sets a value on the board.
  fn set(&mut self, row: usize, col: usize, num: u8) {
    self.0[row][col] = num;
  }

  /// Tries to add a 2 or 4 value to the board. Returns [`Some`] coordinates of
  /// spawned value on success, [`None`] otherwise.
  pub fn spawn(&mut self) -> Option<(u8, (usize, usize))> {
    let coords = self
      .iter_numbers()
      .enumerate()
      .filter_map(|(i, v)| v.eq(&0).then_some(i))
      .choose(&mut rand::rng())
      .map(|idx| (idx / N, idx % N));
    let (row, col) = coords?;
    let num = if rand::random_bool(Self::TWO_TO_FOUR_SPAWN_CHANCE / 100.0) {
      1
    } else {
      2
    };
    self.set(row, col, num);
    coords.map(|c| (num, c))
  }

  /// Returns `true` if [`Board`] can be shifted to any direction, `false`
  /// otherwise.
  pub fn is_shiftable(&self) -> bool {
    if self.0[0][0] == 0 {
      return true;
    }
    for i in 0..N - 1 {
      for j in 0..N {
        let (it, down) = (self.0[i][j], self.0[i + 1][j]);
        if down == 0 || it == down {
          return true;
        }
        let (it, right) = (self.0[j][i], self.0[j][i + 1]);
        if right == 0 || it == right {
          return true;
        }
      }
    }
    false
  }

  /// Moves values on the board to given `direction` and returns [TileAction]s
  /// that were taken to update the board.
  pub fn shift(&mut self, direction: Direction) -> Vec<TileAction> {
    match direction {
      Direction::Left => self
        .0
        .iter_mut()
        .enumerate()
        .flat_map(|(idx, row)| Self::shift_nums_left(row.each_mut(), idx))
        .collect(),
      Direction::Right => self
        .0
        .iter_mut()
        .enumerate()
        .flat_map(|(idx, row)| {
          let mut row_rev = row.each_mut();
          row_rev.reverse();
          Self::shift_nums_left(row_rev, idx)
            .into_iter()
            .map(|a| TileAction {
              from: (a.from.0, N - 1 - a.from.1),
              to: (a.to.0, N - 1 - a.to.1),
              ..a
            })
        })
        .collect(),
      Direction::Up => (0..N)
        .flat_map(|j| {
          let col = self.0.each_mut().map(|row| &mut row[j]);
          Self::shift_nums_left(col, j)
            .into_iter()
            .map(|a| TileAction {
              from: (a.from.1, a.from.0),
              to: (a.to.1, a.to.0),
              ..a
            })
        })
        .collect(),
      Direction::Down => (0..N)
        .flat_map(|j| {
          let mut col = self.0.each_mut().map(|row| &mut row[j]);
          col.reverse();
          Self::shift_nums_left(col, j)
            .into_iter()
            .map(|a| TileAction {
              from: (N - 1 - a.from.1, a.from.0),
              to: (N - 1 - a.to.1, a.to.0),
              ..a
            })
        })
        .collect(),
    }
  }

  /// In the given array of references to values, shifts values to the right
  /// by 2048 rules.
  fn shift_nums_left(row: [&mut u8; N], row_idx: usize) -> Vec<TileAction> {
    let mut actions = Vec::new();
    let mut i = 0;
    for j in 1..N {
      if *row[j] != 0 {
        if *row[i] == 0 {
          actions.push(TileAction {
            kind: TileActionKind::Move,
            value: *row[j],
            from: (row_idx, j),
            to: (row_idx, i),
          });
          *row[i] = *row[j];
          *row[j] = 0;
        } else if *row[j] == *row[i] {
          *row[i] = row[i].saturating_add(1);
          actions.push(TileAction {
            kind: TileActionKind::Merge,
            value: *row[i],
            from: (row_idx, j),
            to: (row_idx, i),
          });
          *row[j] = 0;
          i += 1;
        } else {
          i += 1;
          if i != j {
            actions.push(TileAction {
              kind: TileActionKind::Move,
              value: *row[j],
              from: (row_idx, j),
              to: (row_idx, i),
            });
            *row[i] = *row[j];
            *row[j] = 0;
          }
        }
      }
    }
    actions
  }
}

#[derive(PartialEq, Eq, Clone)]
pub struct TileAction {
  pub kind: TileActionKind,
  pub value: u8,
  pub from: (usize, usize),
  pub to: (usize, usize),
}

impl std::fmt::Debug for TileAction {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{:?} {}: {:?} -> {:?}",
      self.kind, self.value, self.from, self.to
    )
  }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum TileActionKind {
  Move,
  Merge,
}

#[cfg(test)]
mod tests {
  use super::*;

  fn moved(value: u8, from: (usize, usize), to: (usize, usize)) -> TileAction {
    TileAction {
      kind: TileActionKind::Move,
      value,
      from,
      to,
    }
  }
  fn merged(value: u8, from: (usize, usize), to: (usize, usize)) -> TileAction {
    TileAction {
      kind: TileActionKind::Merge,
      value,
      from,
      to,
    }
  }

  #[test]
  fn empty() {
    const SIZE: usize = 4;
    let board = Board::<SIZE>::empty();
    assert_eq!(board.0, [[0; SIZE]; SIZE]);
    assert_eq!(board.size(), SIZE);
  }

  #[test]
  fn new() {
    const SIZE: usize = 4;
    let board = Board::<SIZE>::new();
    assert_eq!(board.iter_numbers().filter(|n| *n != 0).count(), 2);
    assert_eq!(board.size(), SIZE);
  }

  #[test]
  fn get_and_set_number() {
    let mut board = Board::<4>::empty();
    assert_eq!(board.get(1, 3), 0);
    board.set(1, 3, 255);
    assert_eq!(board.get(1, 3), 255);
  }

  #[test]
  fn add_number() {
    let mut board = Board::<4>::empty();
    board.spawn();
    assert_eq!(board.iter_numbers().filter(|n| *n != 0).count(), 1);
    for _ in 0..15 {
      assert!(board.spawn().is_some());
    }
    assert_eq!(board.iter_numbers().filter(|n| *n == 0).count(), 0);
    assert!(board.spawn().is_none());
  }

  #[test]
  fn is_shiftable() {
    for board in [
      Board::<4>::empty(),
      Board([
        [0, 2, 3, 4], //
        [5, 6, 7, 8],
        [9, 10, 11, 12],
        [13, 14, 15, 16],
      ]),
      Board([
        [1, 2, 3, 4], //
        [5, 6, 0, 8],
        [9, 10, 11, 12],
        [13, 14, 15, 16],
      ]),
      Board([
        [1, 2, 3, 4], //
        [5, 6, 7, 8],
        [9, 10, 11, 12],
        [13, 14, 15, 0],
      ]),
      Board([
        [1, 1, 3, 4], //
        [5, 6, 7, 8],
        [9, 10, 11, 12],
        [13, 14, 15, 16],
      ]),
      Board([
        [1, 2, 3, 4], //
        [1, 6, 7, 8],
        [9, 10, 11, 12],
        [13, 14, 15, 16],
      ]),
      Board([
        [1, 2, 3, 4], //
        [5, 6, 7, 7],
        [9, 10, 11, 12],
        [13, 14, 15, 16],
      ]),
      Board([
        [1, 2, 3, 4], //
        [5, 6, 7, 8],
        [9, 10, 7, 12],
        [13, 14, 15, 16],
      ]),
      Board([
        [1, 2, 3, 4], //
        [5, 6, 7, 8],
        [9, 10, 11, 12],
        [13, 14, 15, 15],
      ]),
      Board([
        [1, 2, 3, 4], //
        [5, 6, 7, 8],
        [9, 10, 11, 12],
        [13, 14, 15, 12],
      ]),
    ] {
      assert!(board.is_shiftable(), "{board:#?} should be shiftable");
    }
    let board = Board([
      [1, 2, 3, 4], //
      [5, 6, 7, 8],
      [9, 10, 11, 12],
      [13, 14, 15, 16],
    ]);
    assert!(!board.is_shiftable());
  }

  #[test]
  fn shift_row_left() {
    for (before, after) in [
      ([0, 0, 0, 0], [0, 0, 0, 0]),
      ([1, 0, 0, 0], [1, 0, 0, 0]),
      ([0, 1, 0, 0], [1, 0, 0, 0]),
      ([0, 0, 0, 1], [1, 0, 0, 0]),
      ([1, 2, 0, 0], [1, 2, 0, 0]),
      ([1, 0, 2, 0], [1, 2, 0, 0]),
      ([0, 1, 2, 0], [1, 2, 0, 0]),
      ([0, 1, 0, 2], [1, 2, 0, 0]),
      ([1, 1, 0, 0], [2, 0, 0, 0]),
      ([1, 0, 1, 0], [2, 0, 0, 0]),
      ([1, 0, 0, 1], [2, 0, 0, 0]),
      ([0, 1, 1, 0], [2, 0, 0, 0]),
      ([0, 1, 0, 1], [2, 0, 0, 0]),
      ([0, 0, 1, 1], [2, 0, 0, 0]),
      ([1, 1, 1, 0], [2, 1, 0, 0]),
      ([1, 0, 1, 1], [2, 1, 0, 0]),
      ([1, 1, 1, 1], [2, 2, 0, 0]),
      ([1, 2, 0, 2], [1, 3, 0, 0]),
      ([2, 1, 0, 2], [2, 1, 2, 0]),
      ([2, 0, 1, 2], [2, 1, 2, 0]),
      ([1, 1, 0, 2], [2, 2, 0, 0]),
      ([1, 0, 1, 2], [2, 2, 0, 0]),
      ([0, 1, 1, 2], [2, 2, 0, 0]),
      ([1, 2, 1, 2], [1, 2, 1, 2]),
    ] {
      let mut shifted = before;
      Board::<4>::shift_nums_left(shifted.each_mut(), 0);
      assert_eq!(
        after, shifted,
        "expected {after:?}, got {shifted:?} (originally {before:?})"
      );
    }
  }

  #[test]
  fn shift_empty() {
    use Direction::*;

    for dir in [Up, Down, Left, Right] {
      let left = Board::<4>::empty();
      let mut right = left.clone();
      let actions = right.shift(dir);
      assert_eq!(left, right);
      assert!(actions.is_empty());
    }
  }

  #[test]
  fn shift() {
    use Direction::*;

    for (before, dir, after, actions) in [
      (
        Board([[1, 0, 0, 2], [1, 0, 1, 2], [1, 0, 2, 2], [1, 1, 2, 2]]),
        Left,
        Board([
          [1, 2, 0, 0], //
          [2, 2, 0, 0],
          [1, 3, 0, 0],
          [2, 3, 0, 0],
        ]),
        vec![
          moved(2, (0, 3), (0, 1)),
          merged(2, (1, 2), (1, 0)),
          moved(2, (1, 3), (1, 1)),
          moved(2, (2, 2), (2, 1)),
          merged(3, (2, 3), (2, 1)),
          merged(2, (3, 1), (3, 0)),
          moved(2, (3, 2), (3, 1)),
          merged(3, (3, 3), (3, 1)),
        ],
      ),
      (
        Board([
          [2, 0, 0, 1], //
          [2, 1, 0, 1],
          [2, 2, 0, 1],
          [2, 2, 1, 1],
        ]),
        Right,
        Board([
          [0, 0, 2, 1], //
          [0, 0, 2, 2],
          [0, 0, 3, 1],
          [0, 0, 3, 2],
        ]),
        vec![
          moved(2, (0, 0), (0, 2)),
          merged(2, (1, 1), (1, 3)),
          moved(2, (1, 0), (1, 2)),
          moved(2, (2, 1), (2, 2)),
          merged(3, (2, 0), (2, 2)),
          merged(2, (3, 2), (3, 3)),
          moved(2, (3, 1), (3, 2)),
          merged(3, (3, 0), (3, 2)),
        ],
      ),
      (
        Board([
          [1, 1, 1, 1], //
          [0, 0, 0, 1],
          [0, 1, 2, 2],
          [2, 2, 2, 2],
        ]),
        Up,
        Board([
          [1, 2, 1, 2], //
          [2, 2, 3, 3],
          [0, 0, 0, 0],
          [0, 0, 0, 0],
        ]),
        vec![
          moved(2, (3, 0), (1, 0)),
          merged(2, (2, 1), (0, 1)),
          moved(2, (3, 1), (1, 1)),
          moved(2, (2, 2), (1, 2)),
          merged(3, (3, 2), (1, 2)),
          merged(2, (1, 3), (0, 3)),
          moved(2, (2, 3), (1, 3)),
          merged(3, (3, 3), (1, 3)),
        ],
      ),
      (
        Board([
          [2, 2, 2, 2], //
          [0, 1, 2, 2],
          [0, 0, 0, 1],
          [1, 1, 1, 1],
        ]),
        Down,
        Board([
          [0, 0, 0, 0], //
          [0, 0, 0, 0],
          [2, 2, 3, 3],
          [1, 2, 1, 2],
        ]),
        vec![
          moved(2, (0, 0), (2, 0)),
          merged(2, (1, 1), (3, 1)),
          moved(2, (0, 1), (2, 1)),
          moved(2, (1, 2), (2, 2)),
          merged(3, (0, 2), (2, 2)),
          merged(2, (2, 3), (3, 3)),
          moved(2, (1, 3), (2, 3)),
          merged(3, (0, 3), (2, 3)),
        ],
      ),
    ] {
      let mut shifted = before.clone();
      let taken_actions = shifted.shift(dir);
      assert_eq!(
        after, shifted,
        "expected {after:?}, got {shifted:?} (originally {before:?})"
      );
      assert_eq!(taken_actions, actions);
    }
  }
}
