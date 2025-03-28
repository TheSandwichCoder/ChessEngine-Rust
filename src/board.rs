// BOARD ENCODING SCHEME
//  0 0 0 0    0 0           0 0         
//  enpassant  white castle  black castle
// castle: Queen, King 
// enpassant : counting starts from  0 0 0 1

// 0 - pawn
// 1 - bishop
// 2 - knight
// 3 - rook
// 4 - queen
// 5 - king
// 6 - pawn
// 7 - bishop
// 8 - knight
// 9 - rook
// 10 - queen
// 11 - king

use crate::move_compute::*;
use crate::functions::*;
use crate::zobrist_hash::*;
use crate::app_settings::DEFAULT_FEN;

const PIECE_TYPE_STRING: &str = "PBNRQKpbnrqk/";

const ENPASSANT_CLEAR_MASK: u16 = 0xFF0F;

pub const BLACK_CASTLE_LEFT_BLOCKER_MASK: u64 = 0xE;
pub const BLACK_CASTLE_RIGHT_BLOCKER_MASK: u64 = 0x60;
pub const BLACK_CASTLE_LEFT_ATTACK_MASK: u64 = 0xC;

pub const WHITE_CASTLE_LEFT_BLOCKER_MASK: u64 = BLACK_CASTLE_LEFT_BLOCKER_MASK << 56;
pub const WHITE_CASTLE_RIGHT_BLOCKER_MASK: u64 = BLACK_CASTLE_RIGHT_BLOCKER_MASK << 56;
pub const WHITE_CASTLE_LEFT_ATTACK_MASK: u64 = BLACK_CASTLE_LEFT_ATTACK_MASK << 56;

pub const WHITE_LEFT_ROOK_DEFAULT: u64 = 0x100000000000000;
pub const WHITE_RIGHT_ROOK_DEFAULT: u64 = 0x8000000000000000;

pub const BLACK_LEFT_ROOK_DEFAULT: u64 = 0x1;
pub const BLACK_RIGHT_ROOK_DEFAULT: u64 = 0x80;


const MOVE_FUNCTIONS_ARRAY: [fn(&ChessBoard, &mut MoveBuffer, u8, u64); 6] = [
    add_pawn_moves, 
    add_bishop_moves, 
    add_knight_moves,
    add_rook_moves,
    add_queen_moves,
    add_king_moves
];

#[derive(Clone)]
pub struct ChessBoard{
    pub piece_bitboards: [u64; 12],
    pub piece_array: [u8; 64],
    pub board_info:u16,

    pub white_piece_bitboard: u64,
    pub black_piece_bitboard: u64,
    pub all_piece_bitboard: u64,

    pub check_mask: u64,
    pub is_double_check: bool,
    pub is_updated: bool,

    pub pin_mask: u64,
    pub attack_mask: u64,

    pub board_color: bool,

    pub zobrist_hash: u64,
}

pub fn create_empty_board() -> ChessBoard{
    return ChessBoard{
        piece_bitboards:[0; 12],
        piece_array: [0; 64],
        board_info: 0,
        white_piece_bitboard: 0,
        black_piece_bitboard: 0,
        all_piece_bitboard: 0,

        check_mask: 0,
        is_double_check: false,
        is_updated: false, 
        pin_mask: 0,
        attack_mask: 0,
        board_color: false,

        zobrist_hash: 0,
    }
}

fn get_piece_type_color(piece_type: &i32) -> bool{
    return *piece_type < 6;
}

fn add_piece_to_board(chess_board :&mut ChessBoard, piece_type: i32, piece_square: i32){
    let piece_bitboard: u64 = 1 << piece_square;
    chess_board.piece_bitboards[piece_type as usize] |= piece_bitboard;
    chess_board.piece_array[piece_square as usize] = (piece_type + 1) as u8;

    if get_piece_type_color(&piece_type){
        chess_board.white_piece_bitboard |= piece_bitboard;
    }
    else{
        chess_board.black_piece_bitboard |= piece_bitboard;
    }

    chess_board.all_piece_bitboard |= piece_bitboard;
}

pub fn print_board(chess_board: &ChessBoard){
    for i in 0..64{
        if i % 8 == 0{
            println!("");
        }
        let piece_type: u8 = chess_board.piece_array[i];

        if piece_type == 0{
            print!("_ ");
            continue;
        }

        print!("{} ", PIECE_TYPE_STRING.chars().nth((piece_type-1) as usize).unwrap());
    }
    println!("");
}

pub fn print_board_info(chess_board: &ChessBoard){
    // print bitboard info
    let mut chess_board_array: [char; 64] = ['_'; 64];

    let mut piece_bitboard_together: u64 = 1<<64 - 1;

    if chess_board.board_color{
        println!("To Move: White \n");
    }
    else{
        println!("To Move: Black \n");
    }

    println!("White Castle King: {}", chess_board.board_info & 4 > 0);
    println!("White Castle Queen: {}", chess_board.board_info & 8 > 0);
    println!("Black Castle King: {}", chess_board.board_info & 1 > 0);
    println!("Black Castle Queen: {}", chess_board.board_info & 2 > 0);
    println!("En Passant: {}", chess_board.board_info >> 4);

    print!("\n\nPiece Bitboard:");
    for i in 0..12{
        let mut bitboard: u64 = chess_board.piece_bitboards[i];
        piece_bitboard_together &= bitboard;

        while bitboard != 0{
            let piece_index: i32 = bitboard.trailing_zeros().try_into().unwrap();
            
            chess_board_array[piece_index as usize] = PIECE_TYPE_STRING.chars().nth(i).unwrap();

            bitboard ^= 1 << piece_index;
        }
    }

    for j in 0..64{
        if j % 8 == 0{
            println!("");
        }
        print!("{} ",chess_board_array[j]);
    }
    print!("\n\nPiece Array:");

    for i in 0..64{
        if i % 8 == 0{
            println!("");
        }
        let piece_type: u8 = chess_board.piece_array[i];

        if piece_type == 0{
            print!("_ ");
            continue;
        }

        print!("{} ", PIECE_TYPE_STRING.chars().nth((piece_type-1) as usize).unwrap());
    }

    print!("\n\nWhite Pieces Bitboard");
    for i in 0..64{
        let bitboard_mask: u64 = 1 << i;

        if i % 8 == 0{
            println!("");
        }

        if bitboard_mask & chess_board.white_piece_bitboard != 0{
            print!("1 ");
        }
        else{
            print!("_ ");
        }
    }

    print!("\n\nBlack Pieces Bitboard");
    for i in 0..64{
        let bitboard_mask: u64 = 1 << i;
        
        if i % 8 == 0{
            println!("");
        }

        if bitboard_mask & chess_board.black_piece_bitboard != 0{
            print!("1 ");
        }
        else{
            print!("_ ");
        }
    }

    print!("\n\nAll Pieces Bitboard");
    for i in 0..64{
        let bitboard_mask: u64 = 1 << i;
        
        if i % 8 == 0{
            println!("");
        }

        if bitboard_mask & chess_board.all_piece_bitboard != 0{
            print!("1 ");
        }
        else{
            print!("_ ");
        }
    }


    print!("\n\nPiece Attack Bitboard");

    print_bitboard(chess_board.attack_mask);

    print!("\n\nPiece Check Bitboard");
    println!("\nDouble Check: {}", chess_board.is_double_check);
    print_bitboard(chess_board.check_mask);

    print!("\n\nPiece Pin Bitboard");
    print_bitboard(chess_board.pin_mask);

    print!("\n\nPiece Bitboard Overlap: {} \n", piece_bitboard_together > 0);

    // print fen string
    println!("fen: {}", board_to_fen(&chess_board));

    println!("zobrist: {}", chess_board.zobrist_hash);

}

pub fn fen_to_board(fen_string: &str) -> ChessBoard{
    let mut chess_board: ChessBoard = create_empty_board();

    let mut move_turn: u16 = 0;
    let mut castle_priv: u16 = 0;

    let mut counter : i32 = 0;
    let mut fen_string_part: i32 = 0;
    let mut enpassant_square: u16 = 0;

    // actual fen string stuff
    for p in fen_string.chars() {
        if p == ' '{
            fen_string_part += 1;
            continue;
        }

        // fen - board pieces
        if fen_string_part == 0{
            if p.is_numeric(){
                counter += p.to_digit(10).unwrap() as i32;
            }
            else{
                if !in_string(PIECE_TYPE_STRING, p){
                    println!("FAILED TO PARSE");
                    return fen_to_board(DEFAULT_FEN);
                }

                let piece_array_index: i32 = PIECE_TYPE_STRING.chars().position(|c| c == p).unwrap() as i32;

                // '/' character
                if piece_array_index == 12{
                    continue;
                }
                else{
                    add_piece_to_board(&mut chess_board, piece_array_index, counter);
                }
                counter += 1;
            }
        }

        // fen - move turn 
        else if fen_string_part == 1{
            if p == 'w'{
                move_turn = 1;
            }
            else{
                move_turn = 0;
            }
        }

        // fen - castling
        else if fen_string_part == 2{
            if p == 'K'{
                castle_priv |= 4;
            }
            else if p == 'Q'{
                castle_priv |= 8;
            }
            else if p == 'k'{
                castle_priv |= 1;
            }
            else if p == 'q'{
                castle_priv |= 2;
            }
        }

        // fen - en passant
        else if fen_string_part == 3{
            // some idiot decided to represent enpassant with a target square
            if p == '-'{
                continue;
            }

            if !p.is_numeric(){
                // 'b' - 'a' is 1
                enpassant_square = (p as u16) - ('a' as u16) + 1;
            }            
        }

        // fen - fifty move rule
        else if fen_string_part == 4{
            // uhh yeah I'll also do this later. Thanks future me
        }
    }

    // updates
    chess_board.board_info |= castle_priv;
    chess_board.board_color = move_turn == 1;
    chess_board.board_info |= enpassant_square << 4;

    // reset all the important masks
    chess_board.attack_mask = 0;
    chess_board.pin_mask = 0;
    chess_board.check_mask = !0;

    // zobrist
    chess_board.zobrist_hash = get_full_zobrist_hash(&chess_board);
    
    return chess_board;
}

pub fn board_to_fen(chess_board: &ChessBoard) -> String{
    let mut square_counter: usize = 0;

    let mut fen_string = String::new();

    let mut fen_string_counter = 0;
    while square_counter < 64{
        let piece_type: u8 = chess_board.piece_array[square_counter];

        if piece_type == 0{
            fen_string_counter += 1;
        }

        else{
            if fen_string_counter != 0{
                fen_string.push_str(&fen_string_counter.to_string());
                fen_string_counter = 0;
            }

            fen_string.push_str(&PIECE_TYPE_STRING.chars().nth(piece_type as usize -1).expect("could not find char").to_string());
        }

        if square_counter%8 == 7{
            if fen_string_counter != 0{
                fen_string.push_str(&fen_string_counter.to_string());
                fen_string_counter = 0;
            }

            fen_string.push_str("/");
        }

        square_counter += 1;
    }

    // the other ugly stuff in a fen string

    fen_string.push_str(" ");

    if chess_board.board_color{
        fen_string.push_str("w");
    }
    else{
        fen_string.push_str("b");
    }

    fen_string.push_str(" ");

    if chess_board.board_info & 15 != 0{
        // right white castle
        if chess_board.board_info >> 2 & 1 != 0{
            fen_string.push_str("K");
        }

        // left white castle
        if chess_board.board_info >> 3 & 1 != 0{
            fen_string.push_str("Q");
        }

        // right black castle
        if chess_board.board_info & 1 != 0{
            fen_string.push_str("k");
        }

        // left black castle
        if chess_board.board_info >> 1 & 1 != 0{
            fen_string.push_str("q");
        }
    }
    else{
        fen_string.push_str("-");
    }

    fen_string.push_str(" ");

    // en passant
    if chess_board.board_info >> 4 > 0{
        // fen_string.push_str(&(chess_board.board_info >> 4).to_string());
        if chess_board.board_color{
            fen_string.push_str(&num_to_coord((chess_board.board_info >> 4) + 15));
        }
        else{
            fen_string.push_str(&num_to_coord((chess_board.board_info >> 4) + 39));
        }
    }
    else{
        fen_string.push_str("-");
    }

    // umm do this later thx
    fen_string.push_str(" 0 1");

    return fen_string;
}

// DOES NOT return a new board
pub fn make_move(chess_board: &mut ChessBoard, mv: u16){
    chess_board.is_updated = false; 

    let from_square: u8 = (mv & MOVE_DECODER_MASK) as u8;
    let to_square: u8 = ((mv >> 6) & MOVE_DECODER_MASK) as u8;

    let from_square_bitboard: u64 = 1 << from_square;
    let to_square_bitboard: u64 = 1 << to_square;

    let special: u8 = ((mv >> 12) & MOVE_DECODER_MASK) as u8;

    let previous_board_info = chess_board.board_info;

    let piece_type: usize = chess_board.piece_array[from_square as usize] as usize;
    let piece_color_offset: u8;

    let is_piece_capture: bool = chess_board.all_piece_bitboard & 1<<to_square != 0;

    let mut update_zobrist_castle: bool = false;

    // normal movement
    chess_board.piece_bitboards[piece_type-1] ^= from_square_bitboard;
    chess_board.piece_bitboards[piece_type-1] ^= to_square_bitboard;

    // update zobrist hash for moved piece 
    chess_board.zobrist_hash ^= zobrist_hash_table[get_zobrist_piece_index((piece_type-1) as u8, from_square)];
    chess_board.zobrist_hash ^= zobrist_hash_table[get_zobrist_piece_index((piece_type-1) as u8, to_square)];

    // piece is taken
    if is_piece_capture{
        // get rid piece at the bitboard
        let taken_piece_type: u8 = chess_board.piece_array[to_square as usize] - 1;

        // took a rook for castle permission updating
        if taken_piece_type % 6 == 3{
            if chess_board.board_color{
                // took black left rook
                if to_square == 0{
                    chess_board.board_info &= !(0x2);
                }
                // took black right rook
                else if to_square == 7{
                    chess_board.board_info &= !(0x1);
                }
            }
            else{
                // took white left rook
                if to_square == 56{
                    chess_board.board_info &= !(0x8);
                }

                // took white right rook
                else if to_square == 63{
                    chess_board.board_info &= !(0x4);
                }
            }
            
            update_zobrist_castle = true;
        }

        chess_board.piece_bitboards[taken_piece_type as usize] ^= to_square_bitboard;
        
        // updates the piece zobrist
        chess_board.zobrist_hash ^= zobrist_hash_table[get_zobrist_piece_index(taken_piece_type, to_square)];
    }

    // update castle permission
    if piece_type%6 == 4{
        // piece moved is a rook
        if chess_board.board_color{
            // moved the left white rook
            if from_square == 56{
                chess_board.board_info &= !(0x8);
            }
            else if from_square == 63{
                chess_board.board_info &= !(0x4);
            }
        }
        else{
            // moved the left black rook
            if from_square == 0{
                chess_board.board_info &= !(0x2);
            }
            else if from_square == 7{
                chess_board.board_info &= !(0x1);
            }
        }
        
        update_zobrist_castle = true;
    }

    // disallow castling once king moves
    if piece_type%6 == 0{
        if chess_board.board_color{
            chess_board.board_info &= !(0xC);
        }
        else{
            chess_board.board_info &= !(0x3);
        }
        update_zobrist_castle = true;
    }

    chess_board.piece_array[from_square as usize] = 0;
    chess_board.piece_array[to_square as usize] = piece_type as u8;

    if is_piece_capture{
        if chess_board.board_color{
            chess_board.black_piece_bitboard ^= to_square_bitboard;
        }
        else{
            chess_board.white_piece_bitboard ^= to_square_bitboard;
        }

        // this is the counteract the normal xor
        chess_board.all_piece_bitboard ^= to_square_bitboard;
    }

    if chess_board.board_color{
        chess_board.white_piece_bitboard ^= from_square_bitboard;
        chess_board.white_piece_bitboard ^= to_square_bitboard;
        piece_color_offset = 6;
    }
    else{
        chess_board.black_piece_bitboard ^= from_square_bitboard;
        chess_board.black_piece_bitboard ^= to_square_bitboard;
        piece_color_offset = 0;
    }

    chess_board.all_piece_bitboard ^= from_square_bitboard;
    chess_board.all_piece_bitboard ^= to_square_bitboard;

    // need to clear double move from zobrist
    if chess_board.board_info & !ENPASSANT_CLEAR_MASK != 0{
        let enpassant_column : u8 = (chess_board.board_info >> 4) as u8;

        chess_board.zobrist_hash ^= zobrist_hash_table[ENPASSANT_INDEX_START + enpassant_column as usize];

        // get rid of the double move
        chess_board.board_info &= ENPASSANT_CLEAR_MASK;
    }

    // special movement
    if special > 0{
        if special == 2{
            let enpassant_column = (from_square % 8 + 1) as u16;
            // add new possible en passant
            chess_board.board_info |= enpassant_column << 4;

            // add the enpassant hash
            chess_board.zobrist_hash ^= zobrist_hash_table[ENPASSANT_INDEX_START + enpassant_column as usize];
        }
        // enpassant
        else if special == 3{
            // remove the pawn there
            if chess_board.board_color{
                let capture_bitboard: u64 = 1 << (to_square + 8);

                chess_board.piece_bitboards[6] ^= capture_bitboard;
                chess_board.piece_array[(to_square + 8) as usize] = 0;
                
                chess_board.all_piece_bitboard ^= capture_bitboard;
                chess_board.black_piece_bitboard ^= capture_bitboard;

                chess_board.zobrist_hash ^= zobrist_hash_table[get_zobrist_piece_index(6, to_square+8)];
            }
            else{
                let capture_bitboard: u64 = 1 << (to_square - 8);

                chess_board.piece_bitboards[0] ^= capture_bitboard;
                chess_board.piece_array[(to_square - 8) as usize] = 0;

                chess_board.all_piece_bitboard ^= capture_bitboard;
                chess_board.white_piece_bitboard ^= capture_bitboard;

                chess_board.zobrist_hash ^= zobrist_hash_table[get_zobrist_piece_index(0, to_square-8)];
            }
        }

        // promotion
        else if special <= 8{
            // im not proud of this, but it makes my code readable
            let switched_piece_color_offset:u8 = 6-piece_color_offset;

            // bishop (5) - 4 -> local piece type (1)
            let promotion_piece_type: u8 = special - 4;
            
            // get rid of the pawn there
            chess_board.piece_bitboards[switched_piece_color_offset as usize] ^= to_square_bitboard;

            // add new piece to its bitboard
            chess_board.piece_bitboards[(promotion_piece_type + switched_piece_color_offset) as usize] ^= to_square_bitboard;

            // add new piece to piece array
            chess_board.piece_array[to_square as usize] = promotion_piece_type + switched_piece_color_offset + 1;

            // get rid of pawn there
            chess_board.zobrist_hash ^= zobrist_hash_table[get_zobrist_piece_index((piece_type-1) as u8, to_square)];

            // add new piece there
            chess_board.zobrist_hash ^= zobrist_hash_table[get_zobrist_piece_index((promotion_piece_type + switched_piece_color_offset) as u8, to_square)];
        }

        // castling
        else{
            if special == 9{
                chess_board.piece_bitboards[3] ^= WHITE_LEFT_ROOK_DEFAULT;
                chess_board.piece_bitboards[3] ^= chess_board.piece_bitboards[5] << 1;

                chess_board.white_piece_bitboard ^= WHITE_LEFT_ROOK_DEFAULT;
                chess_board.white_piece_bitboard ^= chess_board.piece_bitboards[5] << 1;

                chess_board.piece_array[56] = 0;
                chess_board.piece_array[59] = 4;

                chess_board.board_info &= !0xC;

                // update rook zobrist hash
                chess_board.zobrist_hash ^= zobrist_hash_table[get_zobrist_piece_index(3, 56)];
                chess_board.zobrist_hash ^= zobrist_hash_table[get_zobrist_piece_index(3, 59)];
            }
            else if special == 10{
                chess_board.piece_bitboards[3] ^= WHITE_RIGHT_ROOK_DEFAULT;
                chess_board.piece_bitboards[3] ^= chess_board.piece_bitboards[5] >> 1;

                chess_board.white_piece_bitboard ^= WHITE_RIGHT_ROOK_DEFAULT;
                chess_board.white_piece_bitboard ^= chess_board.piece_bitboards[5] >> 1;

                chess_board.piece_array[63] = 0;
                chess_board.piece_array[61] = 4;

                chess_board.board_info &= !0xC;

                // update rook zobrist hash
                chess_board.zobrist_hash ^= zobrist_hash_table[get_zobrist_piece_index(3, 63)];
                chess_board.zobrist_hash ^= zobrist_hash_table[get_zobrist_piece_index(3, 61)];
            }

            else if special == 11{
                chess_board.piece_bitboards[9] ^= BLACK_LEFT_ROOK_DEFAULT;
                chess_board.piece_bitboards[9] ^= chess_board.piece_bitboards[11] << 1;

                chess_board.black_piece_bitboard ^= BLACK_LEFT_ROOK_DEFAULT;
                chess_board.black_piece_bitboard ^= chess_board.piece_bitboards[11] << 1;

                chess_board.piece_array[0] = 0;
                chess_board.piece_array[3] = 10;

                chess_board.board_info &= !0x3;

                // update rook zobrist hash
                chess_board.zobrist_hash ^= zobrist_hash_table[get_zobrist_piece_index(9, 0)];
                chess_board.zobrist_hash ^= zobrist_hash_table[get_zobrist_piece_index(9, 3)];
            }

            else{
                chess_board.piece_bitboards[9] ^= BLACK_RIGHT_ROOK_DEFAULT;
                chess_board.piece_bitboards[9] ^= chess_board.piece_bitboards[11] >> 1;

                chess_board.black_piece_bitboard ^= BLACK_RIGHT_ROOK_DEFAULT;
                chess_board.black_piece_bitboard ^= chess_board.piece_bitboards[11] >> 1;

                chess_board.piece_array[7] = 0;
                chess_board.piece_array[5] = 10;

                chess_board.board_info &= !0x3;

                // update rook zobrist hash
                chess_board.zobrist_hash ^= zobrist_hash_table[get_zobrist_piece_index(9, 7)];
                chess_board.zobrist_hash ^= zobrist_hash_table[get_zobrist_piece_index(9, 5)];
            }

            update_zobrist_castle = true;

            chess_board.all_piece_bitboard = chess_board.black_piece_bitboard | chess_board.white_piece_bitboard;
        }
    }

    // some castle permissions were changed
    if update_zobrist_castle{
        let prev_castle_info : usize = (previous_board_info & 0xF) as usize;
        let curr_castle_info: usize = (chess_board.board_info & 0xF) as usize;
        
        chess_board.zobrist_hash ^= zobrist_hash_table[CASTLING_INDEX_START + prev_castle_info];
        chess_board.zobrist_hash ^= zobrist_hash_table[CASTLING_INDEX_START + curr_castle_info];
    }

    // flip the board color
    chess_board.board_color = !chess_board.board_color;

    // flip the color for zobrist hash
    chess_board.zobrist_hash ^= BOARD_COLOR_HASH;

    chess_board.attack_mask = 0;
    chess_board.pin_mask = 0;
    chess_board.check_mask = !0;

    // update_board_attack_mask(chess_board);
    // update_board(chess_board);
}

pub fn get_board_individual_attack_mask(chess_board: &ChessBoard, attack_arr: &mut [u64; 12]){
    for piece_type in 0..12{
        let mut temp_piece_bitboard: u64 = chess_board.piece_bitboards[piece_type];

        let rel_piece_type: u8 = piece_type as u8 % 6;

        while temp_piece_bitboard != 0{
            let square : usize = temp_piece_bitboard.trailing_zeros() as usize;
            if rel_piece_type == 0{
                if piece_type == 0{
                    attack_arr[piece_type as usize] |= WHITE_PAWN_ATTACK_MASK[square];
                }
                else{
                    attack_arr[piece_type as usize] |= BLACK_PAWN_ATTACK_MASK[square];
                }
                
            }
            else if rel_piece_type == 1{
                attack_arr[piece_type as usize] |= get_bishop_move_bitboard(square, chess_board.all_piece_bitboard);
            }
            else if rel_piece_type == 2{
                attack_arr[piece_type as usize] |= KNIGHT_MOVE_MASK[square];
            }
            else if rel_piece_type == 3{
                attack_arr[piece_type as usize] |= get_rook_move_bitboard(square, chess_board.all_piece_bitboard);
            }
            else if rel_piece_type == 4{
                attack_arr[piece_type as usize] |= get_queen_move_bitboard(square, chess_board.all_piece_bitboard);                    
            }
            else if rel_piece_type == 5{
                attack_arr[piece_type as usize] |= KING_MOVE_MASK[square];
            }

            temp_piece_bitboard ^= 1<<square;
        }
    }    
}

pub fn get_board_attack_mask(chess_board: &ChessBoard, color: bool) -> u64{
    let mut attack_mask : u64 = 0;

    if color{
        for piece_type in 0..6{
            let mut temp_piece_bitboard: u64 = chess_board.piece_bitboards[piece_type];

            while temp_piece_bitboard != 0{
                let square : usize = temp_piece_bitboard.trailing_zeros() as usize;

                if piece_type == 0{
                    attack_mask |= WHITE_PAWN_ATTACK_MASK[square];
                }
                else if piece_type == 1{
                    attack_mask |= get_bishop_move_bitboard(square, chess_board.all_piece_bitboard);
                }
                else if piece_type == 2{
                    attack_mask |= KNIGHT_MOVE_MASK[square];
                }
                else if piece_type == 3{
                    attack_mask |= get_rook_move_bitboard(square, chess_board.all_piece_bitboard);
                }
                else if piece_type == 4{
                    attack_mask |= get_queen_move_bitboard(square, chess_board.all_piece_bitboard);                    
                }
                else if piece_type == 5{
                    attack_mask |= KING_MOVE_MASK[square];
                }

                temp_piece_bitboard ^= 1<<square;
            }
        }    
    }
    else{
        for piece_type in 6..12{
            let mut temp_piece_bitboard: u64 = chess_board.piece_bitboards[piece_type];

            while temp_piece_bitboard != 0{
                let square : usize = temp_piece_bitboard.trailing_zeros() as usize;

                if piece_type == 6{
                    attack_mask |= BLACK_PAWN_ATTACK_MASK[square];
                }
                else if piece_type == 7{
                    attack_mask |= get_bishop_move_bitboard(square, chess_board.all_piece_bitboard);
                }
                else if piece_type == 8{
                    attack_mask |= KNIGHT_MOVE_MASK[square];
                }
                else if piece_type == 9{
                    attack_mask |= get_rook_move_bitboard(square, chess_board.all_piece_bitboard);
                }
                else if piece_type == 10{
                    attack_mask |= get_queen_move_bitboard(square, chess_board.all_piece_bitboard);                    
                }
                else if piece_type == 11{
                    attack_mask |= KING_MOVE_MASK[square];
                }

                temp_piece_bitboard ^= 1<<square;
            }
        }
    }

    return attack_mask;
}

// update board stuff
pub fn update_board_attack_mask(chess_board: &mut ChessBoard){
    // update it
    chess_board.attack_mask = get_board_attack_mask(chess_board, !chess_board.board_color);
}

pub fn update_board_check_mask(chess_board: &mut ChessBoard){
    // reset it 
    chess_board.check_mask = 0;
    chess_board.is_double_check = false;

    let king_bitboard: u64;
    let enemy_blockers: u64;
    let enemy_directional_pieces: u64;
    let piece_type_check_offset: usize;
    
    if chess_board.board_color{
        king_bitboard = chess_board.piece_bitboards[5];
        enemy_blockers = chess_board.black_piece_bitboard;
        enemy_directional_pieces = chess_board.piece_bitboards[7] | chess_board.piece_bitboards[9] | chess_board.piece_bitboards[10];
        piece_type_check_offset = 6;
    }
    else{
        king_bitboard = chess_board.piece_bitboards[11];
        enemy_blockers = chess_board.white_piece_bitboard;
        enemy_directional_pieces = chess_board.piece_bitboards[1] | chess_board.piece_bitboards[3] | chess_board.piece_bitboards[4];
        piece_type_check_offset = 0;
    }

    // there are no checks
    if chess_board.attack_mask & king_bitboard == 0{
        return;
    }

    let king_square:usize = king_bitboard.trailing_zeros() as usize;

    let king_check_direction_mask: u64 = get_queen_move_bitboard(king_square, chess_board.all_piece_bitboard);
    let mut check_piece_mask: u64 = king_check_direction_mask & enemy_directional_pieces;
    
    let mut check_num: u8 = 0;

    while check_piece_mask != 0{
        
        let enemy_checker_square : usize = check_piece_mask.trailing_zeros() as usize;
        let enemy_checker_type: u8 = chess_board.piece_array[enemy_checker_square as usize] - (piece_type_check_offset as u8) - 1;
        let direction: bool = get_direction(enemy_checker_square as u8, king_square as u8);
        
        if direction{
            // if the piece is a rook or a queen
            if enemy_checker_type == 3 || enemy_checker_type == 4{
                chess_board.check_mask |= king_check_direction_mask & get_rook_move_bitboard(enemy_checker_square, chess_board.all_piece_bitboard) & ROOK_MOVE_MASK[king_square];

                // update the attack mask so king doesnt move backwards
                chess_board.attack_mask |= ROOK_MOVE_MASK[enemy_checker_square];

                chess_board.check_mask |= 1 << enemy_checker_square;
                check_num += 1;
            }
        }
        else{
            // if the piece is a bishop or a queen
            if enemy_checker_type == 1 || enemy_checker_type == 4{
                chess_board.check_mask |= king_check_direction_mask & get_bishop_move_bitboard(enemy_checker_square, chess_board.all_piece_bitboard) & BISHOP_MOVE_MASK[king_square];
            
                // update the attack mask so king doesnt move backwards
                chess_board.attack_mask |= BISHOP_MOVE_MASK[enemy_checker_square];

                chess_board.check_mask |= 1 << enemy_checker_square;
                check_num += 1;
            }
        }

        check_piece_mask ^= 1 << enemy_checker_square;
    }
    
    let knight_check_mask : u64 = KNIGHT_MOVE_MASK[king_square] & chess_board.piece_bitboards[2 + piece_type_check_offset];
    
    // knight checks
    if knight_check_mask != 0{
        chess_board.check_mask |= knight_check_mask;
        check_num += 1;
    }

    // pawn checks
    let pawn_check_mask : u64;
    if chess_board.board_color{
        pawn_check_mask = WHITE_PAWN_ATTACK_MASK[king_square] & chess_board.piece_bitboards[6];
    }
    else{
        pawn_check_mask = BLACK_PAWN_ATTACK_MASK[king_square] & chess_board.piece_bitboards[0];
    }

    if pawn_check_mask != 0{
        chess_board.check_mask |= pawn_check_mask;
        check_num += 1;
    }

    if check_num >= 2{
        chess_board.is_double_check = true;
    }
}

pub fn update_board_pin_mask(chess_board: &mut ChessBoard){
    // reset it
    chess_board.pin_mask = 0;

    let king_bitboard: u64;
    let enemy_blockers: u64;
    let friendly_blockers: u64;

    let piece_type_pin_offset: usize;
    
    if chess_board.board_color{
        king_bitboard = chess_board.piece_bitboards[5];
        enemy_blockers = chess_board.black_piece_bitboard;
        friendly_blockers = chess_board.white_piece_bitboard;
        piece_type_pin_offset = 6;
    }
    else{
        king_bitboard = chess_board.piece_bitboards[11];
        enemy_blockers = chess_board.white_piece_bitboard;
        friendly_blockers = chess_board.black_piece_bitboard;
        piece_type_pin_offset = 0;
    }

    let king_square:usize = king_bitboard.trailing_zeros() as usize;

    let king_pin_direction_mask: u64 = get_queen_move_bitboard(king_square, enemy_blockers);
    let king_rook_check_direction_mask: u64;
    let king_bishop_check_direction_mask: u64;
    
    let mut possible_pinners: u64 = king_pin_direction_mask & enemy_blockers;
    
    // checks whether there are even any pinners
    if possible_pinners != 0{
        king_rook_check_direction_mask = get_rook_move_bitboard(king_square, chess_board.all_piece_bitboard);
        king_bishop_check_direction_mask = get_bishop_move_bitboard(king_square, chess_board.all_piece_bitboard);
        
        while possible_pinners != 0{
            let enemy_square: usize = possible_pinners.trailing_zeros() as usize;
            let enemy_piece_type: u8 = chess_board.piece_array[enemy_square] - (piece_type_pin_offset as u8) - 1;
            let direction: bool = get_direction(enemy_square as u8, king_square as u8);
                     
            possible_pinners ^= 1 << enemy_square;

            if direction{

                // the enemy piece(pinner) is not a rook or queen 
                if enemy_piece_type != 3 && enemy_piece_type != 4{
                    continue;
                }

                // get the friendly blockers that are pinned
                let pin_square_bitboard: u64 = get_rook_move_bitboard(enemy_square, chess_board.all_piece_bitboard) & king_rook_check_direction_mask & friendly_blockers;
    
                if pin_square_bitboard != 0{
                    let pin_square: usize = pin_square_bitboard.trailing_zeros() as usize;
                    let pin_piece_type: usize = (chess_board.piece_array[pin_square] as usize) + piece_type_pin_offset - 7;
    
                    // piece is a rook or queen
                    if pin_piece_type == 3 || pin_piece_type == 4{
                        chess_board.pin_mask |= get_rook_move_bitboard(pin_square, chess_board.all_piece_bitboard) & ROOK_MOVE_MASK[king_square];
                    }

                    // piece is a pawn
                    else if pin_piece_type == 0{

                        // vertical pin
                        if enemy_square / 8 != pin_square / 8{
                            if chess_board.board_color{
                                let pawn_infront_square: u64 = 1 << (pin_square - 8);
                                chess_board.pin_mask |= pawn_infront_square;
                                
                                // make sure there is nothing infront pawn
                                if pin_square - 8 >= 8 && pawn_infront_square & chess_board.all_piece_bitboard == 0{
                                    chess_board.pin_mask |= 1<< (pin_square - 16);
                                }
                            }
                            else{
                                let pawn_infront_square: u64 = 1 << (pin_square + 8);
                                chess_board.pin_mask |= pawn_infront_square;

                                // make sure there is nothing infront pawn
                                if pin_square + 16 <= 63 && pawn_infront_square & chess_board.all_piece_bitboard == 0{
                                    chess_board.pin_mask |= 1 << (pin_square + 16);
                                }
                            }
                        }                       
                    }

                    chess_board.pin_mask |= pin_square_bitboard;
                }
            }
            
            // bishop direction
            else{
                // exit if the enemy piece(pinner) is not a bishop or queen 
                if enemy_piece_type != 1 && enemy_piece_type != 4{
                    continue;
                }

                // get the friendly blockers that are pinned
                let pin_square_bitboard: u64 = get_bishop_move_bitboard(enemy_square, chess_board.all_piece_bitboard) & king_bishop_check_direction_mask & friendly_blockers;

                if pin_square_bitboard != 0{
                    let pin_square: usize = pin_square_bitboard.trailing_zeros() as usize;

                    // the weird syntax here is to flip the piece_type_pin_offset since we want friendly pieces
                    let pin_piece_type: usize = (chess_board.piece_array[pin_square] as usize) + piece_type_pin_offset - 7;
                    
                    // piece is a bishop or a queen
                    if pin_piece_type == 1 || pin_piece_type == 4{
                        chess_board.pin_mask |= get_bishop_move_bitboard(pin_square, chess_board.all_piece_bitboard) & BISHOP_MOVE_MASK[king_square];
                    }
                    
                    // piece is a pawn
                    // diagonal checks for pawns is literally only for enpassnat
                    // which I find pretty funny... what a waste
                    else if pin_piece_type == 0{
                        if chess_board.board_color{
                            // piece is on the left and king is under the piece
                            if (pin_square % 8 < king_square % 8) == (king_square > pin_square){
                                chess_board.pin_mask |= 1 << (pin_square - 9);
                            }
                            else{
                                chess_board.pin_mask |= 1 << (pin_square - 7);
                            }
                        }
                        else{
                            // piece is on the left and king is above the piece
                            if (pin_square % 8 < king_square % 8) == (king_square < pin_square){
                                chess_board.pin_mask |= 1 << (pin_square + 7);
                            }
                            else{
                                chess_board.pin_mask |= 1 << (pin_square + 9);
                            }
                        }
                    }
                    
                    chess_board.pin_mask |= pin_square_bitboard;
                }
            }
        }
        
    }
}

pub fn update_board(chess_board: &mut ChessBoard){
    if chess_board.is_updated{
        return;
    }

    if chess_board.attack_mask == 0{
        update_board_attack_mask(chess_board);
    }

    if chess_board.check_mask == !0{
        update_board_check_mask(chess_board);
    }
    
    update_board_pin_mask(chess_board);

    chess_board.is_updated = true;
}

pub fn get_capture_moves(chess_board: &mut ChessBoard, move_buffer: &mut MoveBuffer){
    update_board(chess_board);

    let piece_color_offset: usize;
    let opp_all_piece_bitboard: u64;

    if chess_board.board_color{
        piece_color_offset = 0;
        opp_all_piece_bitboard = chess_board.black_piece_bitboard;
    }
    else{
        piece_color_offset = 6;
        opp_all_piece_bitboard = chess_board.white_piece_bitboard;
    }

    if !chess_board.is_double_check{
        for piece_type in 0..6{
            let mut temp_piece_bitboard: u64 = chess_board.piece_bitboards[piece_type + piece_color_offset];

            while temp_piece_bitboard != 0{
                let square:u8 = temp_piece_bitboard.trailing_zeros() as u8;

                MOVE_FUNCTIONS_ARRAY[piece_type](chess_board, move_buffer, square, opp_all_piece_bitboard);

                temp_piece_bitboard ^= 1<<square;
            }
        }
    }

    let enpassant_square_x: usize = (chess_board.board_info >> 4) as usize;

    if enpassant_square_x != 0{
        
        let mut enpassant_piece_bitboard: u64;
        let enpassant_to_square: usize;
        let king_square: u8;
        
        if chess_board.board_color{
            // this is really ugly, but it avoids me making a new cache just for enpassant
            // gets the pawns that can enpassant
            enpassant_piece_bitboard = WHITE_PAWN_ATTACK_MASK[31+enpassant_square_x] & chess_board.piece_bitboards[0];
            enpassant_to_square = 15 + enpassant_square_x;
            king_square = chess_board.piece_bitboards[5].trailing_zeros() as u8;
        }
        else{
            enpassant_piece_bitboard = WHITE_PAWN_ATTACK_MASK[39+enpassant_square_x] & chess_board.piece_bitboards[6];
            enpassant_to_square = 39 + enpassant_square_x;
            king_square = chess_board.piece_bitboards[11].trailing_zeros() as u8;
        }

        while enpassant_piece_bitboard != 0{
            let passant_square: u8 = enpassant_piece_bitboard.trailing_zeros() as u8;
            enpassant_piece_bitboard ^= 1<<passant_square;

            // the pawn is pinned
            if chess_board.pin_mask & 1<<passant_square != 0{
                
                // piece is vertical to king and cant passant
                if get_direction(king_square, passant_square as u8){
                    continue;
                }

                // the target square is not allowed by pin mask
                if chess_board.pin_mask & (1<<enpassant_to_square) == 0{
                    continue;
                }
            }

            // king is in check
            if chess_board.check_mask != 0{
                if chess_board.board_color{
                    // cannot capture the pawn that is checking the king
                    if chess_board.check_mask & 1 << (enpassant_to_square + 8) == 0{
                        continue;
                    }
                }
                else{
                    if chess_board.check_mask & 1 << (enpassant_to_square - 8) == 0{
                        continue;
                    }
                } 
            }

            // Really annoying edge case where pinned doesnt cover
            // make sure there is only 1 en passant
            if enpassant_piece_bitboard == 0{
                if chess_board.board_color{
                    // on the en passant row
                    if king_square / 8 == 3{
                        let mut important_blockers = chess_board.all_piece_bitboard;
                        // get rid of passsant pawn and capturing pawn
                        important_blockers ^= 1 << (enpassant_to_square + 8);
                        important_blockers ^= 1 << (passant_square);

                        // check if there is a queen or rook
                        if get_rook_move_bitboard(king_square as usize, important_blockers) 
                        & (chess_board.piece_bitboards[9] | chess_board.piece_bitboards[10]) != 0{
                            continue;
                        }
                    }
                }
                else{
                    // on the en passnat row
                    if king_square / 8 == 4{
                        let mut important_blockers = chess_board.all_piece_bitboard;
                        // get rid of passsant pawn and capturing pawn
                        important_blockers ^= 1 << (enpassant_to_square - 8);
                        important_blockers ^= 1 << (passant_square);

                        // check if there is a queen or rook
                        if get_rook_move_bitboard(king_square as usize, important_blockers) 
                        & (chess_board.piece_bitboards[3] | chess_board.piece_bitboards[4]) != 0{
                            continue;
                        }
                    }
                }
            }
            

            move_buffer.add(get_move_code_special(passant_square, enpassant_to_square as u8, 3));
        }
    }
}

pub fn get_moves(chess_board: &mut ChessBoard, mut move_buffer: &mut MoveBuffer){
    update_board(chess_board);

    let piece_color_offset: usize;

    if chess_board.board_color{
        piece_color_offset = 0;
    }
    else{
        piece_color_offset = 6;
    }

    // standard movement

    if !chess_board.is_double_check{
        for piece_type in 0..6{
            let mut temp_piece_bitboard: u64 = chess_board.piece_bitboards[piece_type + piece_color_offset];

            while temp_piece_bitboard != 0{
                let square:u8 = temp_piece_bitboard.trailing_zeros() as u8;

                MOVE_FUNCTIONS_ARRAY[piece_type](chess_board, move_buffer, square, !0);

                temp_piece_bitboard ^= 1<<square;
            }
        }
    }
    else{
        // double check only consider king movement 

        let king_square:u8 = chess_board.piece_bitboards[5+piece_color_offset].trailing_zeros() as u8;

        MOVE_FUNCTIONS_ARRAY[5](chess_board, move_buffer, king_square, !0);

        // can break early
        return;
    }
    
    // handle double pawn moves
    let mut pawn_double_move_bitboards: u64 = chess_board.piece_bitboards[piece_color_offset];

    if chess_board.board_color{
        pawn_double_move_bitboards &= WHITE_PAWN_DOUBLE_MOVE_MASK;
    }
    else{
        pawn_double_move_bitboards &= BLACK_PAWN_DOUBLE_MOVE_MASK;
    }

    while pawn_double_move_bitboards != 0{
        let square:u8 = pawn_double_move_bitboards.trailing_zeros() as u8;

        add_pawn_double_move(chess_board, move_buffer, square);

        pawn_double_move_bitboards ^= 1<<square;
    }

    // handle castling
    // make sure king is not in check
    if chess_board.check_mask == 0{
        if chess_board.board_color{

            // castle left is possible
            if chess_board.board_info & 0x8 != 0  {
                // no attack squares - no pieces - not in check
                if (chess_board.all_piece_bitboard & WHITE_CASTLE_LEFT_BLOCKER_MASK == 0) && (chess_board.attack_mask & WHITE_CASTLE_LEFT_ATTACK_MASK == 0){
                    move_buffer.add(GET_MOVE_CODE_SPECIAL(60, 58, 9));
                } 
            }
            
            // castle right is possible
            if chess_board.board_info & 0x4 != 0{
                // no attack squares or blockers
                if (chess_board.all_piece_bitboard|chess_board.attack_mask) & WHITE_CASTLE_RIGHT_BLOCKER_MASK == 0{
                    move_buffer.add(GET_MOVE_CODE_SPECIAL(60, 62, 10));
                }
            }
        }
        else{
            // castle left is possible
            if chess_board.board_info & 0x2 != 0  {
                // no attack squares - no pieces - not in check
                if (chess_board.all_piece_bitboard & BLACK_CASTLE_LEFT_BLOCKER_MASK == 0) && (chess_board.attack_mask & BLACK_CASTLE_LEFT_ATTACK_MASK == 0){
                    move_buffer.add(GET_MOVE_CODE_SPECIAL(4, 2, 11));
                }
            }
            
            // castle right is possible
            if chess_board.board_info & 0x1 != 0{
                if (chess_board.all_piece_bitboard|chess_board.attack_mask) & BLACK_CASTLE_RIGHT_BLOCKER_MASK == 0{
                    move_buffer.add(GET_MOVE_CODE_SPECIAL(4, 6, 12));
                }
            }
        }
    }

    // handle enpassant
    let enpassant_square_x: usize = (chess_board.board_info >> 4) as usize;

    if enpassant_square_x != 0{
        
        let mut enpassant_piece_bitboard: u64;
        let enpassant_to_square: usize;
        let king_square: u8;
        
        if chess_board.board_color{
            // this is really ugly, but it avoids me making a new cache just for enpassant
            // gets the pawns that can enpassant
            enpassant_piece_bitboard = WHITE_PAWN_ATTACK_MASK[31+enpassant_square_x] & chess_board.piece_bitboards[0];
            enpassant_to_square = 15 + enpassant_square_x;
            king_square = chess_board.piece_bitboards[5].trailing_zeros() as u8;
        }
        else{
            enpassant_piece_bitboard = WHITE_PAWN_ATTACK_MASK[39+enpassant_square_x] & chess_board.piece_bitboards[6];
            enpassant_to_square = 39 + enpassant_square_x;
            king_square = chess_board.piece_bitboards[11].trailing_zeros() as u8;
        }

        while enpassant_piece_bitboard != 0{
            let passant_square: u8 = enpassant_piece_bitboard.trailing_zeros() as u8;
            enpassant_piece_bitboard ^= 1<<passant_square;

            // the pawn is pinned
            if chess_board.pin_mask & 1<<passant_square != 0{
                
                // piece is vertical to king and cant passant
                if get_direction(king_square, passant_square as u8){
                    continue;
                }

                // the target square is not allowed by pin mask
                if chess_board.pin_mask & (1<<enpassant_to_square) == 0{
                    continue;
                }
            }

            // king is in check
            if chess_board.check_mask != 0{
                if chess_board.board_color{
                    // cannot capture the pawn that is checking the king
                    if chess_board.check_mask & 1 << (enpassant_to_square + 8) == 0{
                        continue;
                    }
                }
                else{
                    if chess_board.check_mask & 1 << (enpassant_to_square - 8) == 0{
                        continue;
                    }
                } 
            }

            // Really annoying edge case where pinned doesnt cover
            // make sure there is only 1 en passant
            if enpassant_piece_bitboard == 0{
                if chess_board.board_color{
                    // on the en passant row
                    if king_square / 8 == 3{
                        let mut important_blockers = chess_board.all_piece_bitboard;
                        // get rid of passsant pawn and capturing pawn
                        important_blockers ^= 1 << (enpassant_to_square + 8);
                        important_blockers ^= 1 << (passant_square);

                        // check if there is a queen or rook
                        if get_rook_move_bitboard(king_square as usize, important_blockers) 
                        & (chess_board.piece_bitboards[9] | chess_board.piece_bitboards[10]) != 0{
                            continue;
                        }
                    }
                }
                else{
                    // on the en passnat row
                    if king_square / 8 == 4{
                        let mut important_blockers = chess_board.all_piece_bitboard;
                        // get rid of passsant pawn and capturing pawn
                        important_blockers ^= 1 << (enpassant_to_square - 8);
                        important_blockers ^= 1 << (passant_square);

                        // check if there is a queen or rook
                        if get_rook_move_bitboard(king_square as usize, important_blockers) 
                        & (chess_board.piece_bitboards[3] | chess_board.piece_bitboards[4]) != 0{
                            continue;
                        }
                    }
                }
            }
            

            move_buffer.add(get_move_code_special(passant_square, enpassant_to_square as u8, 3));
        }
    }
}