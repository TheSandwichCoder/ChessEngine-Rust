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

    println!("{}", total_bishop_num);

    // there are less than 2 bishops or less than 3 knights or less than 1 knight + bishop
    if total_bishop_num < 2 && total_knight_num < 3 && total_bishop_num + total_knight_num < 2{
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

// pawns, rooks and queens increase value
// bishops and knights decrease value
// this is all arbitrary without any research
const ENGAME_PIECE_TYPE_VALUES: [i16; 12] = [
    200,
    250,
    250,
    550,
    1000,
    1000,
    -200,
    -250,
    -250,
    -550,
    -1000,
    -1000,
];

// HIGHLY INSPIRED by Sebastian Lagues king square table
const KING_PIECE_SQUARE_TABLE: [i16; 64] = [
    -80,  -80,  -90,  -90,  -90,  -90,  -80,  -80,  
    -80,  -80,  -80,  -90,  -90,  -80,  -80,  -80,  
    -90,  -50,  -50,  -90,  -90,  -20,  -50,  -90,  
    -50,  -50,  -50,  -90,  -90,  -50,  -50,  -50,  
    -40,  -40,  -40,  -70,  -70,  -40,  -40,  -40,  
    -20,  -20,  -20,  -50,  -50,  -20,  -20,  -20,  
      5,    5,   -5,   -5,   -5,   -5,    5,    5,  
     10,   20,   20,    0,    0,   20,   30,   20,
];

const KING_PIECE_SQUARE_ENDGAME_TABLE: [i16; 64] = [
    -95,  -95,  -90,  -90,  -90,  -90,  -95,  -95,  
    -95,  -50,  -50,  -50,  -50,  -50,  -50,  -95,  
    -90,  -50,  -20,  -20,  -20,  -20,  -50,  -90,  
    -90,  -50,  -20,    0,    0,  -20,  -50,  -90,  
    -90,  -50,  -20,    0,    0,  -20,  -50,  -90,  
    -90,  -50,  -20,  -20,  -20,  -20,  -50,  -90,  
    -95,  -50,  -50,  -50,  -50,  -50,  -50,  -95,  
    -95,  -95,  -90,  -90,  -90,  -90,  -95,  -95,
];

const BISHOP_KNIGHT_ENDGAME_BIAS_TABLE: [i16; 64] = [
    -150, -100, -90,-70, 70, 90, 100, 150,
    -100,   0,  0 ,   0,  0,  0,  0, 100,
    -90,   0,  0 ,   0,  0,  0,  0, 90,
    -70,   0,  0 ,   0,  0,  0,  0, 70,
     70,   0,  0 ,   0,  0,  0,  0, -70,
     90,   0,  0 ,   0,  0,  0,  0, -90,
     100,   0,  0 ,   0,  0,  0,  0, -100,
     150,  100,  90,  70, -70, -90, -100, -150,
];

// HIGHLY HIGHLY INSPIRED (copied) from Sebastian Lagues tables
const PAWN_PIECE_SQUARE_TABLE : [i16; 64] = [
    0,   0,   0,   0,   0,   0,   0,   0,
   50,  50,  50,  50,  50,  50,  50,  50,
   10,  10,  20,  30,  30,  20,  10,  10,
    5,   5,  10,  25,  25,  10,   5,   5,
    0,   0,   0,  20,  20,   0,   0,   0,
    5,  -5, -10,   0,   0, -10,  -5,   5,
    5,  10,  10, -20, -20,  10,  10,   5,
    0,   0,   0,   0,   0,   0,   0,   0
];

const PAWN_PIECE_SQUARE_ENDGAME_TABLE : [i16; 64] = [
    0,   0,   0,   0,   0,   0,   0,   0,
   80,  80,  80,  80,  80,  80,  80,  80,
   50,  50,  50,  50,  50,  50,  50,  50,
   30,  30,  30,  30,  30,  30,  30,  30,
   20,  20,  20,  20,  20,  20,  20,  20,
   10,  10,  10,  10,  10,  10,  10,  10,
   10,  10,  10,  10,  10,  10,  10,  10,
    0,   0,   0,   0,   0,   0,   0,   0
];

const KNIGHT_PIECE_SQUARE_TABLE : [i16; 64] = [
    -50,-40,-30,-30,-30,-30,-40,-50,
    -40,-20,  0,  0,  0,  0,-20,-40,
    -30,  0, 10, 15, 15, 10,  0,-30,
    -30,  5, 15, 20, 20, 15,  5,-30,
    -30,  0, 15, 20, 20, 15,  0,-30,
    -30,  5, 10, 15, 15, 10,  5,-30,
    -40,-20,  0,  5,  5,  0,-20,-40,
    -50,-40,-30,-30,-30,-30,-40,-50,
];

const BISHOP_PIECE_SQUARE_TABLE : [i16; 64] = [
    -20,-10,-10,-10,-10,-10,-10,-20,
    -10,  0,  0,  0,  0,  0,  0,-10,
    -10,  0,  5, 10, 10,  5,  0,-10,
    -10,  5,  5, 10, 10,  5,  5,-10,
    -10,  0, 10, 10, 10, 10,  0,-10,
    -10, 10, 10, 10, 10, 10, 10,-10,
    -10,  5,  0,  0,  0,  0,  5,-10,
    -20,-10,-10,-10,-10,-10,-10,-20,
];

const ROOK_PIECE_SQUARE_TABLE: [i16; 64] = [
    0,  0,  0,  0,  0,  0,  0,  0,
    5, 10, 10, 10, 10, 10, 10,  5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    0,  0,  0,  5,  5,  0,  0,  0
];
const QUEEN_PIECE_SQUARE_TABLE : [i16; 64] =  [
    -20,-10,-10, -5, -5,-10,-10,-20,
    -10,  0,  0,  0,  0,  0,  0,-10,
    -10,  0,  5,  5,  5,  5,  0,-10,
    -5,   0,  5,  5,  5,  5,  0, -5,
    0,    0,  5,  5,  5,  5,  0, -5,
    -10,  5,  5,  5,  5,  5,  0,-10,
    -10,  0,  5,  0,  0,  0,  0,-10,
    -20,-10,-10, -5, -5,-10,-10,-20
];

const IMPORTANT_ATTACK_SQUARES_MASK: u64 = 0x3C7E7E3C0000;

const PIECE_SQUARE_TABLE_REF : [&[i16; 64]; 4] = [
    &BISHOP_PIECE_SQUARE_TABLE,
    &KNIGHT_PIECE_SQUARE_TABLE,
    &ROOK_PIECE_SQUARE_TABLE,
    &QUEEN_PIECE_SQUARE_TABLE
];

pub fn reverse_piece_square_index(index: usize) -> usize{
    let y = index / 8;
    let x = index % 8;

    return 7-y + x;
}


// 1 when highest, 0 at lowest
pub fn get_endgame_weight(board: &ChessBoard) -> f32{
    // let num_pieces = board.all_piece_bitboard.count_ones();

    let pawn_piece_bitboard = board.piece_bitboards[0] | board.piece_bitboards[6];

    // all pieces except pawns
    let important_piece_bitboard = board.all_piece_bitboard ^ pawn_piece_bitboard;

    // pawns are half counted as the rest of the pieces
    let num_pieces: f32 = pawn_piece_bitboard.count_ones() as f32 * 0.5 + important_piece_bitboard.count_ones() as f32;

    // engame weight func - [(-n+24)/24]^2
    let endgame_weight : f32 = (24.0-num_pieces) / 24.0;

    return endgame_weight * endgame_weight;
}

pub fn king_engame_square_weight(board: &ChessBoard, endgame_weight: f32) -> i16{
    let mut score: i16 = 0;

    let white_king_square : usize = board.piece_bitboards[5].trailing_zeros() as usize;
    let black_king_square : usize = board.piece_bitboards[11].trailing_zeros() as usize;

    // disregard engame weight if its too low
    if endgame_weight > 0.3{
        let reversed_black = reverse_piece_square_index(black_king_square);

        score += lerp_int(KING_PIECE_SQUARE_TABLE[white_king_square], KING_PIECE_SQUARE_ENDGAME_TABLE[white_king_square], endgame_weight);
        score -= lerp_int(KING_PIECE_SQUARE_TABLE[reversed_black], KING_PIECE_SQUARE_ENDGAME_TABLE[reversed_black], endgame_weight);
    }
    else{
        score += KING_PIECE_SQUARE_TABLE[white_king_square];
        score -= KING_PIECE_SQUARE_TABLE[reverse_piece_square_index(black_king_square)];   
    }

    return score;
}

pub fn king_distance_weight(board: &ChessBoard, endgame_weight: f32) -> i16{
    if endgame_weight < 0.3{
        return 0;
    }

    let white_king_square : i16 = board.piece_bitboards[5].trailing_zeros() as i16;
    let black_king_square : i16 = board.piece_bitboards[11].trailing_zeros() as i16;

    let distance : f32 = get_manhattan_distance(white_king_square, black_king_square) as f32;

    let weight: f32 = (12.0 - distance) * endgame_weight * 10.0;

    if board.board_color{
        return weight as i16;
    }
    else{
        return -(weight as i16);
    }
}

// eval based on piece value
pub fn get_board_piece_value_score(board: &ChessBoard, endgame_weight: f32) -> i16{
    let mut score: i16 = 0;

    // only if this then we start lerping
    if endgame_weight >= 0.3{
        // skip the king
        for i in 0..5{
            let piece_num = board.piece_bitboards[i].count_ones() as i16;
            score += lerp_int(PIECE_TYPE_VALUES[i], ENGAME_PIECE_TYPE_VALUES[i], endgame_weight) * piece_num;
        }

        for i in 6..11{
            let piece_num = board.piece_bitboards[i].count_ones() as i16;
            score += lerp_int(PIECE_TYPE_VALUES[i], ENGAME_PIECE_TYPE_VALUES[i], endgame_weight) * piece_num;
        }
    }

    else{
        // skip the king
        for i in 0..5{
            score += (board.piece_bitboards[i].count_ones() as i16) * PIECE_TYPE_VALUES[i];
        }

        for i in 6..11{
            score += (board.piece_bitboards[i].count_ones() as i16) * PIECE_TYPE_VALUES[i];
        }
    }
    
    return score;
}

// this only considers non pawns and kings (since they have endgame tables)
pub fn get_board_piece_square_score(board: &ChessBoard) -> i16{
    let mut score: i16 = 0;
    // bishop, knight, rook, queen
    for piece_type in 1..5{
        let mut temp_bitboard: u64 = board.piece_bitboards[piece_type];
        let piece_square_table: &[i16; 64] = PIECE_SQUARE_TABLE_REF[piece_type - 1];

        while temp_bitboard != 0{
            let piece_square: usize = temp_bitboard.trailing_zeros() as usize;

            score += piece_square_table[piece_square];

            temp_bitboard ^= 1 << piece_square;
        }
    }

    for piece_type in 7..11{
        let mut temp_bitboard: u64 = board.piece_bitboards[piece_type];
        let piece_square_table: &[i16; 64] = PIECE_SQUARE_TABLE_REF[piece_type - 7];

        while temp_bitboard != 0{
            let piece_square: usize = temp_bitboard.trailing_zeros() as usize;

            score -= piece_square_table[reverse_piece_square_index(piece_square)];

            temp_bitboard ^= 1 << piece_square;
        }
    }

    return score;
}

pub fn get_pawn_piece_square_score(board: &ChessBoard, endgame_weight: f32) -> i16{
    let mut score: i16 = 0;

    let mut white_pawn_bitboard : u64 = board.piece_bitboards[0];
    let mut black_pawn_bitboard : u64 = board.piece_bitboards[6];

    while white_pawn_bitboard != 0{
        let pawn_square: usize = white_pawn_bitboard.trailing_zeros() as usize;
        
        if endgame_weight > 0.3{
            score += lerp_int(PAWN_PIECE_SQUARE_TABLE[pawn_square], PAWN_PIECE_SQUARE_ENDGAME_TABLE[pawn_square], endgame_weight);
        }
        else{
            score += PAWN_PIECE_SQUARE_TABLE[pawn_square];
        }

        white_pawn_bitboard ^= 1 << pawn_square;
    }

    while black_pawn_bitboard != 0{
        let pawn_square: usize = black_pawn_bitboard.trailing_zeros() as usize;

        let reversed_pawn_square = reverse_piece_square_index(pawn_square);
        
        if endgame_weight > 0.3{
            score -= lerp_int(PAWN_PIECE_SQUARE_TABLE[reversed_pawn_square], PAWN_PIECE_SQUARE_ENDGAME_TABLE[reversed_pawn_square], endgame_weight);
        }
        else{
            score -= PAWN_PIECE_SQUARE_TABLE[reversed_pawn_square];
        }

        black_pawn_bitboard ^= 1 << pawn_square;
    }

    return score;
}

pub fn get_attack_square_score(board: &ChessBoard, endgame_weight: f32) -> i16{
    let mut opp_attack_bitboard : u64 = board.attack_mask;
    let mut self_attack_bitboard : u64 = get_board_attack_mask(board, board.board_color);
    let mut score : i16 = 0;

    let inv_endgame : f32 = 1.0 - endgame_weight;

    // king safety check
    let self_king_square: usize;
    let opp_king_square: usize;

    if board.board_color{
        self_king_square = board.piece_bitboards[5].trailing_zeros() as usize;
        opp_king_square = board.piece_bitboards[11].trailing_zeros() as usize;
    }
    else{
        self_king_square = board.piece_bitboards[11].trailing_zeros() as usize;
        opp_king_square = board.piece_bitboards[5].trailing_zeros() as usize;
    }

    // penalise if king is open
    if endgame_weight < 0.3{
        score -= ((KING_MOVE_MASK[self_king_square] & opp_attack_bitboard).count_ones() as f32 * 5.0 * inv_endgame) as i16;
        score += ((KING_MOVE_MASK[opp_king_square] & self_attack_bitboard).count_ones() as f32 * 5.0 * inv_endgame) as i16;
    }
    
    score -= (opp_attack_bitboard.count_ones() * 2) as i16;
    score += (self_attack_bitboard.count_ones() * 2) as i16;

    opp_attack_bitboard &= IMPORTANT_ATTACK_SQUARES_MASK;
    self_attack_bitboard &= IMPORTANT_ATTACK_SQUARES_MASK;

    score -= (opp_attack_bitboard.count_ones() * 10) as i16;
    score += (self_attack_bitboard.count_ones() * 10) as i16;

    if board.board_color{
        return score;
    }
    else{
        return -score;
    }
}

pub fn bishop_knight_endgame_bias(board: &ChessBoard, board_color: bool, endgame_weight: f32) -> i16{
    
    let bishop_bitboard: u64;
    let opp_king_square: usize;

    if board_color{
        bishop_bitboard = board.piece_bitboards[1];
        opp_king_square = board.piece_bitboards[11].trailing_zeros() as usize;
    }
    else{
        bishop_bitboard = board.piece_bitboards[7];
        opp_king_square = board.piece_bitboards[5].trailing_zeros() as usize;
    }

    // no bishop
    if bishop_bitboard == 0{
        return 0;
    }

    let bishop_square = bishop_bitboard.trailing_zeros();

    let bishop_color : bool = bishop_square % 8 == bishop_square / 8;


    let score: f32;
    if bishop_color{
        score = -BISHOP_KNIGHT_ENDGAME_BIAS_TABLE[opp_king_square] as f32 * endgame_weight;
    }
    else{
        score = BISHOP_KNIGHT_ENDGAME_BIAS_TABLE[opp_king_square] as f32 * endgame_weight;
    }


    if board_color{
        return score as i16;
    }
    else{
        // println!("{} {}", -score, opp_king_square);
        return -score as i16;
    }
}

pub fn get_board_score(board: &ChessBoard) -> i16{
    let mut score: i16 = 0;

    let endgame_weight : f32 = get_endgame_weight(board); 

    score += get_board_piece_value_score(board, endgame_weight);
    score += get_board_piece_square_score(board);

    // prioritises pawn near end nearer to endgame
    score += get_pawn_piece_square_score(board, endgame_weight);

    score += get_attack_square_score(board, endgame_weight);

    // prioritises king near center
    score += king_engame_square_weight(board, endgame_weight);

    // wants king to be closer to other king
    score += king_distance_weight(board, endgame_weight);

    // bishop knight endgame - king square favours corners
    score += bishop_knight_endgame_bias(board, true, endgame_weight);
    score += bishop_knight_endgame_bias(board, false, endgame_weight);

    // relative evaluation due to negamax
    if board.board_color{
        return score;
    }
    else{
        return -score;
    }
    
}
