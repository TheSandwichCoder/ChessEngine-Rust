use std::time::Instant;

use board::*;
use move_compute::*;
use functions::*;
use magic_numbers::*;
use engine::*;

mod board;
mod move_compute;
mod functions;
mod magic_numbers;
mod engine;

fn main() {
    // println!("{}", temp_thing[0]);
    let mut chess_board_1 :ChessBoard = fen_to_board("7q/b1p5/1p1Npkb1/pPP2ppP/P1P5/3B2P1/5P1R/K3R3 w");
    // let depth: u16 = 3;
    // // println!("Depth: {}, Nodes: {}", depth, perft(&chess_board_1, depth));
    // print_board(&chess_board_1);
    // sub_perft(&chess_board_1, depth);

    debug(&mut chess_board_1);
}
