use crate::move_compute::*;
use crate::functions::*;
use crate::board::*;
use crate::game_board::*;
use crate::app_settings::MOVE_LIMIT_MAX;

// 0 - still going
// 1 - white checkmate
// 2 - black checkmate
// 3 - draw
pub fn get_gamestate(game_board: &mut GameChessBoard) -> u8{
    for (_, counter) in &game_board.game_tree {
        // repeated 3 times. Draw
        if *counter >= 3{
            return 3;
        }
    }

    let mut move_buffer = MoveBuffer::new();

    let chess_board : &mut ChessBoard = &mut game_board.board; 

    get_moves(chess_board, &mut move_buffer);

    // no moves
    if move_buffer.index == 0{
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

// heavily inspired from Blunder Engine
const PASS_PAWN_PIECE_SQUARE_TABLE : [i16; 64] = [
	0, 0, 0, 0, 0, 0, 0, 0,
	77, 74, 63, 53, 59, 60, 72, 77,
	91, 83, 66, 40, 30, 61, 67, 84,
	55, 52, 42, 35, 30, 34, 56, 52,
	29, 26, 21, 18, 17, 19, 34, 30,
	8, 6, 5, 1, 1, -1, 14, 7,
	2, 3, -4, 0, -2, -1, 7, 6,
	0, 0, 0, 0, 0, 0, 0, 0,
];

pub const KING_DANGER_SQUARES_MASK : [u64; 64] = GET_KING_DANGER_SQUARES_MASK();



const IMPORTANT_ATTACK_SQUARES_MASK: u64 = 0x3C7E7E3C0000;

const PIECE_SQUARE_TABLE_REF : [&[i16; 64]; 4] = [
    &BISHOP_PIECE_SQUARE_TABLE,
    &KNIGHT_PIECE_SQUARE_TABLE,
    &ROOK_PIECE_SQUARE_TABLE,
    &QUEEN_PIECE_SQUARE_TABLE
];

const fn GET_KING_DANGER_SQUARES_MASK() -> [u64; 64]{
    let mut bb_array = [0u64; 64];
    
    let mut i : i16 = 0;
    let mut i2 : i16 = 0;

    while i < 64{
        let mut bb : u64 = 0;
        i2 = 0;
        while i2 < 64{
            let x = (i % 8 - i2 % 8).abs();
            let y = (i / 8 - i2 / 8).abs();

            if !(x >= 3 || y >= 3){
                if x + y <= 3{
                    bb |= 1 << i2;
                }
            }

            i2 += 1;
        }
        
        bb_array[i as usize] = bb;
        i += 1;
    }

    return bb_array;
}

pub fn reverse_piece_square_index(index: usize) -> usize{
    let y = index / 8;
    let x = index % 8;

    return (7-y) * 8 + x;
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

pub fn king_endgame_square_weight(board: &ChessBoard, endgame_weight: f32) -> i16{
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

const HORIZONTAL_SLICE_BITBOARD : u64 = 0xFF00000000000000;
const VERTICLE_SLICE_BITBOARD : u64 = 0x8080808080808080;
const HALF_SLICE_BITBOARD: u64 = (!0)<<32;

pub fn doubled_pawn_score(board: &ChessBoard, endgame_weight: f32) -> i16{
    let mut score : f32 = 0.0;

    let white_pawn_bitboard = board.piece_bitboards[0];
    let black_pawn_bitboard = board.piece_bitboards[6];

    for x in 0..8{
        // white doubled pawns
        if (white_pawn_bitboard & (VERTICLE_SLICE_BITBOARD >> (7-x))).count_ones() > 1{
            score -= 10. * endgame_weight;
        }

        // black doubled pawns
        if (black_pawn_bitboard & (VERTICLE_SLICE_BITBOARD >> (7-x))).count_ones() > 1{
            score += 10. * endgame_weight;
        }
    }
    
    return score as i16
}

pub fn get_vert_side_bitboards(x: u8) -> u64{
    let mut bitboard: u64 = 0;
    if x != 7{
        bitboard |= VERTICLE_SLICE_BITBOARD >> (6-x);
    }

    if x != 0{
        bitboard |= VERTICLE_SLICE_BITBOARD >> (8-x);
    }

    return bitboard;
}

// pass pawn, isolated pawn, 
pub fn pawn_surrounding_score(board: &ChessBoard, endgame_weight: f32) -> i16{
    let white_pawn_bitboard : u64 = board.piece_bitboards[0];
    let black_pawn_bitboard: u64 = board.piece_bitboards[6];

    let mut white_pawn_temp = white_pawn_bitboard;
    let mut black_pawn_temp = black_pawn_bitboard;

    let mut score: i16 = 0;

    while white_pawn_temp != 0{
        let pawn_square : usize = white_pawn_temp.trailing_zeros() as usize;
        
        let pawn_x : u8 = pawn_square as u8 % 8;
        let pawn_y : u8 = pawn_square as u8 / 8;

        let infront_bb: u64 = !0 >> (8 * (8-pawn_y));

        let side_bb: u64 = get_vert_side_bitboards(pawn_x);

        let pawn_col_bb = (VERTICLE_SLICE_BITBOARD >> (7-pawn_x)) & infront_bb;

        let side_pawn_bb = side_bb & infront_bb;

        if (side_pawn_bb | pawn_col_bb) & black_pawn_bitboard == 0 && pawn_col_bb & white_pawn_bitboard == 0{
            score += PASS_PAWN_PIECE_SQUARE_TABLE[pawn_square as usize];
        }

        white_pawn_temp ^= 1 << pawn_square;
    }

    while black_pawn_temp != 0{
        let pawn_square : usize = black_pawn_temp.trailing_zeros() as usize;        
        
        let pawn_x : u8 = pawn_square as u8 % 8;
        let pawn_y : u8 = pawn_square as u8 / 8;

        let infront_bb: u64 = !0 << (8 * (pawn_y+1));

        let side_bb: u64 = get_vert_side_bitboards(pawn_x);

        // let is_accompanied = side_pawn_bitboard & black_pawn_bitboard != 0;
        // let is_supported = WHITE_PAWN_ATTACK_MASK[pawn_square] & black_pawn_bitboard != 0;
        let pawn_col_bb = (VERTICLE_SLICE_BITBOARD >> (7-pawn_x)) & infront_bb;

        let side_pawn_bb = side_bb & infront_bb;

        if (side_pawn_bb | pawn_col_bb) & white_pawn_bitboard == 0 && pawn_col_bb & black_pawn_bitboard == 0{
            let reversed_pawn_square = reverse_piece_square_index(pawn_square) as usize;
            score -= PASS_PAWN_PIECE_SQUARE_TABLE[reversed_pawn_square];
        }
        black_pawn_temp ^= 1 << pawn_square;
    }

    return (score as f32 * endgame_weight) as i16;
}

pub fn get_attack_square_score(mut white_attack_bitboard : u64, mut black_attack_bitboard: u64, inv_endgame_weight: f32) -> i16{
    let mut score : i16 = 0;

    score += white_attack_bitboard.count_ones() as i16 * 2;
    score -= black_attack_bitboard.count_ones() as i16 * 2;

    white_attack_bitboard &= IMPORTANT_ATTACK_SQUARES_MASK;
    black_attack_bitboard &= IMPORTANT_ATTACK_SQUARES_MASK;

    score += white_attack_bitboard.count_ones() as i16 * 10;
    score -= black_attack_bitboard.count_ones() as i16 * 10;

    return int_float_mul(score, inv_endgame_weight);
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

// -40 if king is no near pawns
// 0 if 3 pawns surrounding
const PAWN_SHIELD_PENALTY : [i16; 8] = [-40, -30, -10, 0, 5, 5, 5, 5];

// directly copied from stockfish... gosh I love open source
const ATTACK_UNIT_TABLE: [i16; 100] = [
    0,  0,   1,   2,   3,   5,   7,   9,  12,  15,
  18,  22,  26,  30,  35,  39,  44,  50,  56,  62,
  68,  75,  82,  85,  89,  97, 105, 113, 122, 131,
 140, 150, 169, 180, 191, 202, 213, 225, 237, 248,
 260, 272, 283, 295, 307, 319, 330, 342, 354, 366,
 377, 389, 401, 412, 424, 436, 448, 459, 471, 483,
 494, 500, 500, 500, 500, 500, 500, 500, 500, 500,
 500, 500, 500, 500, 500, 500, 500, 500, 500, 500,
 500, 500, 500, 500, 500, 500, 500, 500, 500, 500,
 500, 500, 500, 500, 500, 500, 500, 500, 500, 500
];

// bishop and knight -> 2
// rook -> 3
// queen -> 5
const PIECE_ATTACK_UNIT: [u8; 4] = [
    2, 2, 3, 5
];

pub fn king_safety_score(board: &ChessBoard, board_color: bool, ind_piece_attack_squares : &[u64;12], inv_endgame_weight: f32) -> i16{
    let king_square: u8;
    let king_infront_rows_bitboard: u64;
    let friendly_piece_bitboard: u64;
    let friendly_pawn_bitboard: u64;
    let piece_offset: usize;

    if board_color{
        king_square = board.piece_bitboards[5].trailing_zeros() as u8;
        king_infront_rows_bitboard = HALF_SLICE_BITBOARD >> (7-king_square/8) * 8;
        friendly_piece_bitboard = board.white_piece_bitboard;
        friendly_pawn_bitboard = board.piece_bitboards[0];
        piece_offset = 6;
    }
    else{
        king_square = board.piece_bitboards[11].trailing_zeros() as u8;
        king_infront_rows_bitboard = !HALF_SLICE_BITBOARD << (king_square / 8) * 8;
        friendly_piece_bitboard = board.black_piece_bitboard;
        friendly_pawn_bitboard = board.piece_bitboards[6];
        piece_offset = 0;
    }

    let king_x = king_square % 8;
    let king_zone : u64 = KING_MOVE_MASK[king_square as usize] | (king_infront_rows_bitboard & VERTICLE_SLICE_BITBOARD >> (7-king_x)) ^ (1<<king_square);

    let mut score: i16 = 0;    
    // open file penalties
    // to the right
    if king_x != 7{
        if VERTICLE_SLICE_BITBOARD >> (7-king_x - 1) & friendly_piece_bitboard == 0{
            score -= 20;
        }
    }

    // to the left
    if king_x != 0{
        if VERTICLE_SLICE_BITBOARD >> (7-king_x + 1) & friendly_piece_bitboard == 0{
            score -= 20;
        }
    }

    // no close pawns penalty
    let close_pawns_num: usize = (KING_MOVE_MASK[king_square as usize] & friendly_pawn_bitboard).count_ones() as usize;

    score += PAWN_SHIELD_PENALTY[close_pawns_num];

    // attack unit score
    let mut attack_unit_num: u8 = 0;

    // skip king and pawns
    // loops through enemy pieces
    for rel_piece_type in 1..5{
        let attack_unit_bitboard : u64 = ind_piece_attack_squares[rel_piece_type + piece_offset] & king_zone;

        attack_unit_num += attack_unit_bitboard.count_ones() as u8 * PIECE_ATTACK_UNIT[rel_piece_type - 1];
    }

    // penalty
    score -= ATTACK_UNIT_TABLE[attack_unit_num as usize];

    return int_float_mul(score, inv_endgame_weight);
} 

pub fn get_cheap_board_score(board: &ChessBoard) -> i16{
    let mut score: i16 = 0;

    let endgame_weight : f32 = get_endgame_weight(board); 

    score += get_board_piece_value_score(board, endgame_weight);

    score += get_board_piece_square_score(board);
    
    if board.board_color{
        return score;
    }
    else{
        return -score;
    }
}

pub fn get_board_score(board: &ChessBoard) -> i16{
    let mut score: i16 = 0;

    let endgame_weight : f32 = get_endgame_weight(board); 
    let inv_endgame_weight: f32 = 1.0 - endgame_weight;

    
    score += get_board_piece_square_score(board);

    score += get_board_piece_value_score(board, endgame_weight);

    // prioritises pawn near end nearer to endgame
    score += get_pawn_piece_square_score(board, endgame_weight);

    // prioritises king near center
    score += king_endgame_square_weight(board, endgame_weight);
    
    // heavy evals / inv endgame affected scoring

    if inv_endgame_weight > 0.1{
        let mut ind_piece_attack_squares: [u64; 12] = [0;12];

        get_board_individual_attack_mask(board, &mut ind_piece_attack_squares);

        let white_attack_bitboard : u64 = or_together(&ind_piece_attack_squares[0..6]);
        let black_attack_bitboard : u64 = or_together(&ind_piece_attack_squares[6..12]);

        // incentivises control over center and piece mobility
        score += get_attack_square_score(white_attack_bitboard, black_attack_bitboard, inv_endgame_weight);

        score += king_safety_score(board, true, &ind_piece_attack_squares, inv_endgame_weight);
        score -= king_safety_score(board, false, &ind_piece_attack_squares, inv_endgame_weight);
    }
    
    

    // endgame affected scoring

    if endgame_weight > 0.1{
        // wants king to be closer to other king
        score += king_distance_weight(board, endgame_weight);

        // doubled pawn penalty
        score += doubled_pawn_score(board, endgame_weight);
        score += pawn_surrounding_score(board, endgame_weight);

        // bishop knight endgame - king square favours corners
        score += bishop_knight_endgame_bias(board, true, endgame_weight);
        score += bishop_knight_endgame_bias(board, false, endgame_weight);
    }

    

    // relative evaluation due to negamax
    if board.board_color{
        return score;
    }
    else{
        return -score;
    }   
}
