use bevy::color::*;

pub const GRID: Color = Color::srgb_u8(187, 173, 160);

const TILES: [Color; 12] = [
  Color::srgb_u8(205, 192, 181),
  Color::srgb_u8(0xEE, 0xE4, 0xDA),
  Color::srgb_u8(0xED, 0xE0, 0xC8),
  Color::srgb_u8(0xF2, 0xB1, 0x79),
  Color::srgb_u8(0xF5, 0x95, 0x63),
  Color::srgb_u8(0xF6, 0x7C, 0x5F),
  Color::srgb_u8(0xF6, 0x5E, 0x3B),
  Color::srgb_u8(0xED, 0xCF, 0x72),
  Color::srgb_u8(0xED, 0xCC, 0x61),
  Color::srgb_u8(0xED, 0xC8, 0x50),
  Color::srgb_u8(0xED, 0xC5, 0x3F),
  Color::srgb_u8(0xED, 0xC2, 0x2E),
];

const DEFAULT_TILE: Color = Color::srgb_u8(0x3C, 0x3A, 0x32);

#[inline]
pub fn tile_foreground(n: u8) -> Color {
  *TILES.get(n as usize).unwrap_or(&DEFAULT_TILE)
}

pub const TEXT_LIGHT: Color = Color::srgb_u8(0xFC, 0xF4, 0xF0);
pub const TEXT_DARK: Color = Color::srgb_u8(0x5C, 0x53, 0x4A);

#[inline]
pub fn tile_text(n: u8) -> Color {
  if n > 2 { TEXT_LIGHT } else { TEXT_DARK }
}
