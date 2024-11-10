use crate::move_compute::*;
use crate::functions::*;
use crate::board::*;
use crate::game_board::*;
use crate::app_settings::MOVE_LIMIT_MAX;

// 0 - still going
// 1 - white checkmate
// 2 - black checkmate
// 3 - draw
pub fn get_gamestate(game_board: &GameChessBoard) -> u8{
    for (_, counter) in &game_board.game_tree {
        // repeated 3 times. Draw
        if *counter >= 3{
            return 3;
        }
    }

    let mut move_vec: Vec<u16> = Vec::new();

    let chess_board : &ChessBoard = &game_board.board; 

    get_moves(chess_board, &mut move_vec);

    // no moves
    if move_vec.len() == 0{
        if chess_board.check_mask != 0{
            if chess_board.board_color{
                return 2;
            }
            else{
                return 1;
            }
        }

        else{
            // stalemate
            return 3;
        }
    }

    // this code should be temporary but who knows
    if game_board.move_limit >= MOVE_LIMIT_MAX{
        return 3;
    }

    let total_bishop_num = chess_board.piece_bitboards[1].count_ones() + chess_board.piece_bitboards[7].count_ones();
    let total_knight_num = chess_board.piece_bitboards[2].count_ones() + chess_board.piece_bitboards[8].count_ones();

    // there is a pawn or queen or rooks on the board
    if chess_board.piece_bitboards[0] | 
    chess_board.piece_bitboards[6] | 
    chess_board.piece_bitboards[4] | 
    chess_board.piece_bitboards[10] |
    chess_board.piece_bitboards[3] | 
    chess_board.piece_bitboards[9] != 0{
        return 0;
    }

    // there are less than 2 bishops
    if total_bishop_num < 2{
        return 3;
    }

    // there are less than 2 knights
    if total_knight_num < 2{
        return 3;
    }

    return 0;
}

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
