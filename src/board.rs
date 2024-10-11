// BOARD ENCODING SCHEME
// 0        ...   0 0 0 0    0 0           0 0         
// move turn      enpassant  white castle  black castle

// enpassant : counting starts from  0 0 0 1

use crate::move_compute::*;

const PIECE_TYPE_STRING: &str = "PBNRQKpbnrqk/";

pub struct ChessBoard{
    piece_bitboards: [u64; 12],
    piece_array: [u16; 64],
    board_info:u32,

    pub white_piece_bitboard: u64,
    pub black_piece_bitboard: u64,

    pub check_mask: u64,
    pub pin_mask: u64,

    pub board_color: bool,

}

fn create_empty_board() -> ChessBoard{
    return ChessBoard{
        piece_bitboards:[0; 12],
        piece_array: [0; 64],
        board_info: 0,
        white_piece_bitboard: 0,
        black_piece_bitboard: 0,
        check_mask: 0,
        pin_mask: 0,
        board_color: false,
    }
}

fn get_piece_type_color(piece_type: &i32) -> bool{
    return *piece_type < 6;
}

fn add_piece_to_board(chess_board :&mut ChessBoard, piece_type: i32, piece_square: i32){
    let piece_bitboard: u64 = 1 << piece_square;
    chess_board.piece_bitboards[piece_type as usize] |= piece_bitboard;
    chess_board.piece_array[piece_square as usize] = (piece_type + 1) as u16;

    if get_piece_type_color(&piece_type){
        chess_board.white_piece_bitboard |= piece_bitboard;
    }
    else{
        chess_board.black_piece_bitboard |= piece_bitboard;
    }
}

pub fn print_board_info(chess_board: &ChessBoard){
    // print bitboard info
    let mut chess_board_array: [char; 64] = ['_'; 64];

    let mut piece_bitboard_together: u64 = 1<<64 - 1;

    if chess_board.board_info >> 31 == 1{
        println!("To Move: White \n");
    }
    else{
        println!("To Move: Black \n");

    }

    println!("White Castle King: {}", chess_board.board_info & 8 > 0);
    println!("White Castle Queen: {}", chess_board.board_info & 4 > 0);
    println!("Black Castle King: {}", chess_board.board_info & 2 > 0);
    println!("Black Castle Queen: {}", chess_board.board_info & 1 > 0);

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
        let piece_type: u16 = chess_board.piece_array[i];

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

    

    print!("\n\nPiece Bitboard Overlap: {}", piece_bitboard_together > 0)
}

pub fn fen_to_board(fen_string: &str) -> ChessBoard{
    let mut chess_board: ChessBoard = create_empty_board();

    let mut move_turn: u32 = 0;
    let mut castle_priv: u32 = 0;

    let mut counter : i32 = 0;
    let mut fen_string_part: i32 = 0;

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
                castle_priv |= 8;
            }
            else if p == 'Q'{
                castle_priv |= 4;
            }
            else if p == 'k'{
                castle_priv |= 2;
            }
            else if p == 'q'{
                castle_priv |= 1;
            }
        }

        // fen - en passant
        else if fen_string_part == 3{
            // uhhh i'll do this later. Thanks future me
        }

        // fen - fifty move rule
        else if fen_string_part == 4{
            // uhh yeah I'll also do this later. Thanks future me
        }
    }

    chess_board.board_info |= castle_priv;
    chess_board.board_info |= move_turn<<31;
    chess_board.board_color = move_turn == 1;
    
    return chess_board;
}

pub fn get_moves(chess_board: &ChessBoard, move_vec: &mut Vec<u16>){
    let color: u32 = chess_board.board_info >> 31;


    // white to move
    if color == 1{
        for piece_type in 0..6{
            let mut temp_piece_bitboard: u64 = chess_board.piece_bitboards[piece_type as usize];

            if piece_type != 3{
                continue;
            }

            while temp_piece_bitboard != 0{
                let piece_square : u16 = temp_piece_bitboard.trailing_zeros().try_into().unwrap();

                // add_rook_moves(move_vec, piece_square, chess_board.white_piece_bitboard, chess_board.black_piece_bitboard, 0, 0, 0);


                temp_piece_bitboard ^= 1<<piece_square;
            }
        }
        
    }

    else{

    }
}