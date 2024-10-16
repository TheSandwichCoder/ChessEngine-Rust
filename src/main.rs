use std::time::Instant;

use board::*;
use move_compute::*;
use functions::*;
use magic_numbers::*;

mod board;
mod move_compute;
mod functions;
mod magic_numbers;

fn main() {
    // println!("{}", temp_thing[0]);
    // let mut chess_board_1 :ChessBoard = fen_to_board("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    
    let mut chess_board_1: ChessBoard = fen_to_board("4K3/3QRB2/8/4r2r/b3q3/8/8k7 w KQkq - 0 1");
    print_board_info(&chess_board_1);

    let mut move_vec: Vec<u16> = Vec::new();

    // add_pawn_moves(
    //     &mut move_vec,
    //     53,
    //     chess_board_1.white_piece_bitboard,
    //     chess_board_1.black_piece_bitboard,
    //     0,
    //     0,
    //     1,
    // );

    get_moves(&chess_board_1, &mut move_vec);

    // make_move(&mut chess_board_1, move_vec[0]);

    for mv in move_vec{
        print_move(&mv);

        let mut temp_board: ChessBoard = chess_board_1.clone();
        // print_board_info(&temp_board);

        make_move(&mut temp_board, mv);

        print_board(&temp_board);
    }
}
