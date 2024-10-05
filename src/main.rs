use board::*;
use move_compute::*;
use functions::*;

mod board;
mod move_compute;
mod functions;

fn main() {
    println!("{}", ROOK_CACHE_SIZE);
    // let rook_move_cache : [u64; (ROOK_CACHE_ENTRY_SIZE * 2) as usize] = get_rook_legal_move_cache();

    // let rook_blocker_mask: u64 = ROOK_BLOCKERS_MASK[0];

    // let rook_blocker_mask_combination: [u64; 20000] = get_blocker_combinations(rook_blocker_mask);

    // print_bitboard(rook_blocker_mask_combination[2603]);
    
    // let rook_movement_bitboard: u64 = rook_move_cache[get_rook_move_cache_index(0, rook_blocker_mask_combination[2603])];

    // print_bitboard(rook_movement_bitboard);

    // get_rook_legal_move_cache();

    let chess_board_1 :ChessBoard = fen_to_board("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    print_board_info(&chess_board_1);
}
