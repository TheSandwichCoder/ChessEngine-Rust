use board::*;
use move_compute::*;
use functions::*;

mod board;
mod move_compute;
mod functions;

fn main() {
    let blocker_combinations: Vec<u64> = get_blocker_combinations(BISHOP_BLOCKERS_MASK[0]);

    for i in 0..10{
        let blocker_combination: u64 = blocker_combinations[i as usize];
        print_bitboard(blocker_combination);
        print_bitboard(get_bishop_legal_moves(0, blocker_combination));
    }

    let chess_board_1 :ChessBoard = fen_to_board("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    print_board_info(&chess_board_1);
}
