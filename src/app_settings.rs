use bevy::prelude::*;

pub const ENGINE_VERSION: &'static str = "6.0";

pub const SCREENSIZE: Vec3 = Vec3::new(650.0, 650.0, 0.0);
pub const HALF_SCREENSIZE: Vec3 = Vec3::new(SCREENSIZE.x / 2.0, SCREENSIZE.y / 2.0, 0.0);

pub const SQUARE_SIZE: f32 = SCREENSIZE.x / 8.0;
pub const PIECE_EDGE_OFFSET: Vec3 = Vec3::new(SQUARE_SIZE/2.0, SQUARE_SIZE/2.0, 0.0);

pub const IMAGE_SIZES: f32 = 100.0;
pub const SCALE_FACTOR: f32 = SQUARE_SIZE / IMAGE_SIZES; 

pub const DEFAULT_FEN: &'static str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
pub const DEFAULT_THINK_TIME: u16 = 1000;
pub const MAX_SEARCH_DEPTH: u8 = 10;
pub const BATTLE_DEPTH: u8 = 5;
pub const BATTLE_THINK_TIME: u16 = 200;

pub const MOVE_LIMIT_MAX : u16 = 200;