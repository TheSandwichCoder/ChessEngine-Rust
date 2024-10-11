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
    let chess_board_1 :ChessBoard = fen_to_board("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPP1/RNBQKBNR w KQkq - 0 1");
    // // print_board_info(&chess_board_1);

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



    print_moves(&move_vec);

    // add_queen_moves(
    //     &mut move_vec, 
    //     36, 
    //     chess_board_1.white_piece_bitboard, 
    //     chess_board_1.black_piece_bitboard,
    //     0,
    //     0,
    //     0,
    // );

    add_queen_moves(
        &chess_board_1,
        &mut move_vec,
        36,
    );

    print_moves(&move_vec);

    // let square_pos: usize = 36;

    // let mut occupied : u64 = 429857200;

    // print_bitboard(occupied);
    // let random_index: usize = 124914;

    // let mut rook_move_bitboard: u64 = 0;

    // let now = Instant::now();
    
    

    // for i in 0..1000000{
    //     rook_move_bitboard = get_queen_move_bitboard(occupied, square_pos);
    // }
    // println!("{:?}", now.elapsed());

    // print_bitboard(rook_move_bitboard);

    // for i in 0..64{
    //     let rook_blocker_mask: u64 = BISHOP_BLOCKERS_MASK[i as usize];



    //     let rook_blocker_combinations: [u64; 20000] = get_blocker_combinations(rook_blocker_mask);

    //     get_Magic_Number(rook_blocker_combinations);
    // }
    

    
    // let important_blockers_rook: u64 = ROOK_BLOCKERS_MASK[36] & (chess_board_1.white_piece_bitboard|chess_board_1.black_piece_bitboard);

    // let move_bitboard_index_rook: usize = get_rook_move_cache_index(36, important_blockers_rook);

    // let mut something:&u64 = &ROOK_LEGAL_MOVE_CACHE[move_bitboard_index_rook];
    
    // let another_thing: u64 = *something;

    // let array_index: usize = 28917;
    // let now = Instant::now();

    // // let something: u64 = ROOK_LEGAL_MOVE_CACHE[array_index];

    // let something: &u64 = unsafe{ROOK_LEGAL_MOVE_CACHE.get_unchecked(array_index)};
    
    // println!("{:?}", now.elapsed());

    // println!("{}", something);

    // let another_thing: u64 = *something;
    // println!("{:?}", now.elapsed());

        // println!("{}", another_thing);

    
    // for mv in move_vec.iter(){
    //     print_move(mv);
    // }
}
