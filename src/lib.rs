pub mod frame;
pub mod invaders;
pub mod level;
pub mod menu;
pub mod player;
pub mod render;
pub mod score;
pub mod shot;

pub const NUM_ROWS: usize = 20;
pub const NUM_COLS: usize = 40;

pub const INVADER_SPRITE: [char; 2] = ['x', '+'];
pub const PLAYER_SPRITE: char = '▴';
