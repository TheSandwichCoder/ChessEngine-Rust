use crate::move_compute::*;
use crate::functions::*;
use crate::board::*;

const PIECE_TYPE_VALUES : [i16; 12] = [
    100,
    300,
    300,
    500,
    900,
    1000,
    -100,
    -300,
    -300,
    -500,
    -900,
    -1000,
];

// eval based on piece value
pub fn get_board_piece_value_score(board: &ChessBoard) -> i16{
    let mut score: i16 = 0;

    // skip the king
    for i in 0..5{
        score += (board.piece_bitboards[i].count_ones() as i16) * PIECE_TYPE_VALUES[i];
    }

    for i in 6..11{
        score += (board.piece_bitboards[i].count_ones() as i16) * PIECE_TYPE_VALUES[i];
    }

    return score;
}

pub fn get_board_score(board: &ChessBoard) -> i16{
    let mut score: i16 = 0;

    score += get_board_piece_value_score(board);

    return score;
}
