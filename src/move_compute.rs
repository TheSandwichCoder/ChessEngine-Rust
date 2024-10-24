// Move Encoding:
// u16 
// special(4)  to(6)  from(6)
// 0000        000000 000000
//
// Special

// pawn stuff
// 0 0 1 0 -> pawn double
// 0 0 1 1 -> en passant

// pawn promotion
// 0 1 0 1 -> bishop - 5
// 0 1 1 0 -> knight - 6
// 0 1 1 1 -> rook - 7
// 1 0 0 0 -> queen - 8

// castling
// 1 0 0 1 -> white king castle left - 9
// 1 0 1 0 -> white king castle right - 10
// 1 0 1 1 -> black king castle left - 11
// 1 1 0 0 -> black king castle right - 12

use crate::functions::*;
use crate::magic_numbers::*;
use crate::board::*;
use rand::Rng;
use std::time::Instant;


const ROOK_MAGIC_NUMBER_SIZE: i32 = 14;
const BISHOP_MAGIC_NUMBER_SIZE: i32 = 11;

pub const ROOK_CACHE_ENTRY_SIZE: i32 = 1 << ROOK_MAGIC_NUMBER_SIZE;
const BISHOP_CACHE_ENTRY_SIZE: i32 = 1 << BISHOP_MAGIC_NUMBER_SIZE;

pub const ROOK_CACHE_SIZE: i32 = ROOK_CACHE_ENTRY_SIZE * 64;
const BISHOP_CACHE_SIZE: i32 = BISHOP_CACHE_ENTRY_SIZE * 64;

const ROOK_MAGIC_NUMBER_PUSH: i32 = 64 - ROOK_MAGIC_NUMBER_SIZE;
const BISHOP_MAGIC_NUMBER_PUSH: i32 = 64 - BISHOP_MAGIC_NUMBER_SIZE;

const RANDOM_BITBOARD_IDENTIFIER: u64 = 1298301209;

const HORIZONTAL_SLICE_BITBOARD : u64 = 0xFF00000000000000;
const VERTICLE_SLICE_BITBOARD : u64 = 0x8080808080808080;

const PROMOTION_BITBOARD_MASK: u64 = HORIZONTAL_SLICE_BITBOARD >> 56 | HORIZONTAL_SLICE_BITBOARD;

pub const WHITE_PAWN_DOUBLE_MOVE_MASK: u64 = HORIZONTAL_SLICE_BITBOARD >> 8;
pub const BLACK_PAWN_DOUBLE_MOVE_MASK: u64 = HORIZONTAL_SLICE_BITBOARD >> 48;

const EDGE_MASK: u64 = 0xFF818181818181FF;

pub const MOVE_DECODER_MASK:u16 = 0x3F;

// HEAVILY inspired by rust Pleco Engine
#[derive(Copy, Clone)]
pub struct SMagic {
    pub ptr: *const u64,
    pub mask: u64,
    pub magic: u64,
    pub shift: u32,
}

const fn GET_PAWN_ATTACK_MASK(color: bool) -> [u64; 64]{
    let mut attack_array : [u64; 64] = [0; 64];
    let mut counter : u8 = 0;

    while counter < 64{
        let y_pos : u8 = counter / 8;
        let x_pos : u8 = counter % 8;
        let mut attack_bitboard: u64 = 0;
        // skip the end rows    
        if (y_pos == 0 && color) || (y_pos == 7 && !color){
            counter += 1;
            continue;
        }

        // near the right side
        if x_pos != 7{
            // white
            if color{
                attack_bitboard |= 1 << (counter - 7);
            }
            else{
                attack_bitboard |= 1 << (counter + 9);
            }
        }
        
        if x_pos != 0{
            if color{
                attack_bitboard |= 1 << (counter - 9);
            }
            else{
                attack_bitboard |= 1 << (counter + 7);
            }
        }

        attack_array[counter as usize] = attack_bitboard;
        counter += 1;
    }

    return attack_array;
}

const fn GET_KNIGHT_MOVE_MASK() -> [u64; 64]{
    let mut bitboard_array: [u64; 64] = [0; 64];

    let mut i: i32 = 0;
    let move_x: [i32; 8] = [-1, 1, 2,2, 1,-1,-2,-2];
    let move_y: [i32; 8] = [-2,-2,-1,1, 2, 2, 1,-1];

    while i < 64{
        let mut bitboard: u64 = 0;

        let x: i32 = i % 8;
        let y: i32 = i / 8;

        let mut temp_x:i32;
        let mut temp_y:i32;

        let mut j:usize = 0;

        while j < 8{
            temp_x = x + move_x[j];
            temp_y = y + move_y[j];

            if temp_x >= 0 && temp_y >= 0 && temp_x < 8 && temp_y < 8{
                bitboard |= 1 << (temp_y * 8 + temp_x);
            }

            j += 1;
        }

        bitboard_array[i as usize] = bitboard;
        i += 1;
    }

    return bitboard_array;
}

const fn GET_KING_MOVE_MASK() -> [u64; 64]{
    let mut bitboard_array: [u64; 64] = [0; 64];

    let mut i: i32 = 0;
    let move_x: [i32; 8] = [-1,0, 1, -1,1,-1,0,1];
    let move_y: [i32; 8] = [-1,-1,-1,0, 0, 1,1,1];

    while i < 64{
        let mut bitboard: u64 = 0;

        let x: i32 = i % 8;
        let y: i32 = i / 8;

        let mut temp_x:i32;
        let mut temp_y:i32;

        let mut j:usize = 0;

        while j < 8{
            temp_x = x + move_x[j];
            temp_y = y + move_y[j];

            if temp_x >= 0 && temp_y >= 0 && temp_x < 8 && temp_y < 8{
                bitboard |= 1 << (temp_y * 8 + temp_x);
            }

            j += 1;
        }

        bitboard_array[i as usize] = bitboard;
        i += 1;
    }

    return bitboard_array;
}

const fn GET_ROOK_MOVE_MASK() -> [u64; 64]{
    let mut bitboard_array: [u64; 64] = [0; 64];

    let mut i: i32 = 0;
    while i < 64{
        let x : i32 = i % 8;
        let y: i32 = i / 8;

        bitboard_array[(63-i) as usize] = ((HORIZONTAL_SLICE_BITBOARD >> (y * 8)) | (VERTICLE_SLICE_BITBOARD >> x)) & !(1<<(63-i));
        i += 1;
    }

    return bitboard_array;
}

const fn GET_BISHOP_MOVE_MASK() -> [u64; 64]{
    let mut bitboard_array: [u64; 64] = [0; 64];

    let mut i: i32 = 0;
    while i < 64{
        let mut bitboard: u64 = 0;

        let x: i32 = i % 8;
        let y: i32 = i / 8;

        let x_vec : [i32; 4] = [1, 1, -1, -1];
        let y_vec : [i32; 4] = [1, -1, 1, -1];

        let mut j: i32 = 0;
        while j < 4{
            let mut temp_x: i32 = x;
            let mut temp_y: i32 = y;

            while temp_x >= 0 && temp_y >= 0 && temp_x < 8 && temp_y < 8{
                bitboard |= 1 << (temp_y * 8 + temp_x);
                temp_x += x_vec[j as usize];
                temp_y += y_vec[j as usize];
            }
            j += 1;
        }
        
        // get rid of the piece pos
        bitboard &= !(1 << i);

        bitboard_array[i as usize] = bitboard;
        i += 1;
    }

    return bitboard_array;
}

const fn GET_QUEEN_MOVE_MASK() -> [u64; 64]{
    let rook_blocker_mask: [u64; 64] = GET_ROOK_MOVE_MASK();
    let bishop_blocker_mask: [u64; 64] = GET_BISHOP_MOVE_MASK();
    let mut queen_blockers_mask: [u64; 64] = [0; 64];

    let mut i : i32 = 0;

    while i < 64{
        queen_blockers_mask[i as usize] = rook_blocker_mask[i as usize] | bishop_blocker_mask[i as usize];
        i += 1;
    }

    return queen_blockers_mask;
}

const fn GET_ROOK_BLOCKER_MASK() -> [u64; 64]{
    let mut bitboard_array: [u64; 64] = [0; 64];

    let strict_hor_slice_bitboard = 0x7E00000000000000;
    let strict_vert_slice_bitboard = 0x80808080808000;

    let mut i: i32 = 0;
    while i < 64{
        let x : i32 = i % 8;
        let y: i32 = i / 8;

        // dont question this dark magic
        bitboard_array[(63-i) as usize] = ((strict_hor_slice_bitboard >> (y * 8)) | (strict_vert_slice_bitboard >> x)) & !(1<<(63-i));
        i += 1;
    }

    return bitboard_array;
}

const fn GET_BISHOP_BLOCKER_MASK() -> [u64; 64]{
    let mut bitboard_array: [u64; 64] = [0; 64];

    let mut i: i32 = 0;
    while i < 64{
        let mut bitboard: u64 = 0;

        let x: i32 = i % 8;
        let y: i32 = i / 8;

        let x_vec : [i32; 4] = [1, 1, -1, -1];
        let y_vec : [i32; 4] = [1, -1, 1, -1];

        let mut j: i32 = 0;
        while j < 4{
            let mut temp_x: i32 = x;
            let mut temp_y: i32 = y;

            while temp_x >= 0 && temp_y >= 0 && temp_x < 8 && temp_y < 8{
                bitboard |= 1 << (temp_y * 8 + temp_x);
                temp_x += x_vec[j as usize];
                temp_y += y_vec[j as usize];
            }
            j += 1;
        }
        
        // get rid of the piece pos
        bitboard &= !(1 << i);

        // get rid of the side pieces
        bitboard &= !(EDGE_MASK);

        bitboard_array[i as usize] = bitboard;
        i += 1;
    }

    return bitboard_array;
}

const fn GET_QUEEN_BLOCKER_MASK() -> [u64; 64]{
    let rook_blocker_mask: [u64; 64] = GET_ROOK_BLOCKER_MASK();
    let bishop_blocker_mask: [u64; 64] = GET_BISHOP_BLOCKER_MASK();
    let mut queen_blockers_mask: [u64; 64] = [0; 64];

    let mut i : i32 = 0;

    while i < 64{
        queen_blockers_mask[i as usize] = rook_blocker_mask[i as usize] | bishop_blocker_mask[i as usize];
        i += 1;
    }

    return queen_blockers_mask;
}



pub const fn get_blocker_combinations(bitboard: u64) -> [u64; 20000]{
    let mut temp_bitboard: u64 = bitboard;
    let mut bits_num: u64 = 0;

    while temp_bitboard != 0{
        let least_bit: u32 = temp_bitboard.trailing_zeros();
        temp_bitboard ^= 1 << least_bit;

        bits_num += 1;
    }

    let combination_num: u64 = 1 << bits_num;
    let mut blocker_combinations:[u64;20000] = [RANDOM_BITBOARD_IDENTIFIER; 20000];

    let mut i : u64 = 0;
    while i < combination_num{
        let mut temp_bitboard: u64 = bitboard;
        let mut new_bitboard: u64 = 0;
    
        let mut j: u64 = 0;
        while j < bits_num{
            // get the pos of the bit
            let least_bit: u32 = temp_bitboard.trailing_zeros();
            
            // get the val of the bit
            let bit_val: u64 = i >> j & 1;

            new_bitboard |= bit_val << least_bit;
            
            temp_bitboard ^= 1 << least_bit;
            j += 1;
        }
        blocker_combinations[i as usize] = new_bitboard;
        
        i += 1;
    }

    return blocker_combinations;
}

pub const fn get_rook_legal_moves(square: i32, blockers: u64) -> u64{
    let mut bitboard: u64 = 0;

    let x: i32 = square % 8;
    let y: i32 = square / 8;

    let x_vec : [i32; 4] = [1, 0, -1, 0];
    let y_vec : [i32; 4] = [0, 1, 0, -1];

    let mut j: i32 = 0;

    // goes through the 4 directions
    while j < 4{
        let mut temp_x: i32 = x;
        let mut temp_y: i32 = y;

        while temp_x >= 0 && temp_y >= 0 && temp_x < 8 && temp_y < 8{
            let square_bitboard: u64 = 1 << (temp_y * 8 + temp_x);
            bitboard |= square_bitboard;
            temp_x += x_vec[j as usize];
            temp_y += y_vec[j as usize];

            // there is a blocker
            if blockers & square_bitboard != 0{
                break;
            }
        }
        j += 1;
    }
    
    // get rid of the piece pos
    bitboard &= !(1 << square);

    return bitboard;
}

pub const fn get_bishop_legal_moves(square: i32, blockers: u64) -> u64{
    let mut bitboard: u64 = 0;

    let x: i32 = square % 8;
    let y: i32 = square / 8;

    let x_vec : [i32; 4] = [1, 1, -1, -1];
    let y_vec : [i32; 4] = [1, -1, 1, -1];

    let mut j: i32 = 0;

    // goes through the 4 directions
    while j < 4{
        let mut temp_x: i32 = x;
        let mut temp_y: i32 = y;

        while temp_x >= 0 && temp_y >= 0 && temp_x < 8 && temp_y < 8{
            let square_bitboard: u64 = 1 << (temp_y * 8 + temp_x);
            bitboard |= square_bitboard;
            temp_x += x_vec[j as usize];
            temp_y += y_vec[j as usize];

            // there is a blocker
            if blockers & square_bitboard != 0{
                break;
            }
        }
        j += 1;
    }
    
    // get rid of the piece pos
    bitboard &= !(1 << square);

    return bitboard;
}

// these are the masks to identify the important blockers
pub const ROOK_BLOCKERS_MASK: [u64; 64] = GET_ROOK_BLOCKER_MASK();
pub const BISHOP_BLOCKERS_MASK: [u64; 64] = GET_BISHOP_BLOCKER_MASK();
pub const QUEEN_BLOCKERS_MASK: [u64; 64] = GET_QUEEN_BLOCKER_MASK();



pub const fn INITIALISE_ROOK_MOVE_CACHE() -> [u64; ROOK_MOVE_CACHE_SIZE]{

    let mut rook_move_cache: [u64; ROOK_MOVE_CACHE_SIZE as usize] = [0; ROOK_MOVE_CACHE_SIZE as usize];

    let mut square: usize = 0;
    let mut square_index:usize = 0;
    while square < 64{
        let rook_blocker_mask: u64 = ROOK_BLOCKERS_MASK[square];
        let rook_magic_number_push: u8 = ROOK_RAW_SHIFTS[square];
        let rook_blocker_combinations: [u64; 20000] = get_blocker_combinations(rook_blocker_mask);

        let mut rook_blocker_counter: i32 = 0;

        // goes through all the blocker combinations in initialises them
        while rook_blocker_combinations[rook_blocker_counter as usize] != RANDOM_BITBOARD_IDENTIFIER{
            let rook_blocker_combination: u64 = rook_blocker_combinations[rook_blocker_counter as usize];
            let rook_legal_move: u64 = get_rook_legal_moves(square as i32, rook_blocker_combination);

            let local_cache_index: u64 =  (rook_blocker_combination.wrapping_mul(ROOK_RAW_MAGICS[square])) >> rook_magic_number_push;

            let move_cache_index: usize = square_index + (local_cache_index as usize);

            // initialise it
            rook_move_cache[move_cache_index] = rook_legal_move;

            rook_blocker_counter += 1;
        }

        square_index += 1 << (64 - rook_magic_number_push);

        square += 1;
    }

    return rook_move_cache;
}

pub const fn INITIALISE_BISHOP_MOVE_CACHE() -> [u64; BISHOP_MOVE_CACHE_SIZE]{

    let mut bishop_move_cache: [u64; BISHOP_MOVE_CACHE_SIZE as usize] = [0; BISHOP_MOVE_CACHE_SIZE as usize];

    let mut square: usize = 0;
    let mut square_index:usize = 0;
    while square < 64{
        let bishop_blocker_mask: u64 = BISHOP_BLOCKERS_MASK[square];
        let bishop_magic_number_push: u8 = BISHOP_RAW_SHIFTS[square];
        let bishop_blocker_combinations: [u64; 20000] = get_blocker_combinations(bishop_blocker_mask);

        let mut bishop_blocker_counter: i32 = 0;

        // goes through all the blocker combinations in initialises them
        while bishop_blocker_combinations[bishop_blocker_counter as usize] != RANDOM_BITBOARD_IDENTIFIER{
            let bishop_blocker_combination: u64 = bishop_blocker_combinations[bishop_blocker_counter as usize];
            let bishop_legal_move: u64 = get_bishop_legal_moves(square as i32, bishop_blocker_combination);

            let local_cache_index: u64 =  (bishop_blocker_combination.wrapping_mul(BISHOP_RAW_MAGICS[square])) >> bishop_magic_number_push;

            let move_cache_index: usize = square_index + (local_cache_index as usize);

            // initialise it
            bishop_move_cache[move_cache_index] = bishop_legal_move;

            bishop_blocker_counter += 1;
        }

        square_index += 1 << (64 - bishop_magic_number_push);

        square += 1;
    }

    return bishop_move_cache;
}

#[allow(long_running_const_eval)]
pub const ROOK_MOVE_CACHE : [u64 ; ROOK_MOVE_CACHE_SIZE] = INITIALISE_ROOK_MOVE_CACHE();
pub const ROOK_SMAGICS : [SMagic; 64] = INITIALISE_ROOK_MAGICS();

pub const BISHOP_MOVE_CACHE : [u64; BISHOP_MOVE_CACHE_SIZE] = INITIALISE_BISHOP_MOVE_CACHE();
pub const BISHOP_SMAGICS : [SMagic; 64] = INITIALISE_BISHOP_MAGICS();

pub const fn INITIALISE_ROOK_MAGICS() -> [SMagic; 64]{
    let mut index : usize = 0;

    let mut move_ptr : usize = 0;

    let mut rook_magics : [SMagic; 64] = [SMagic{
        ptr:ROOK_MOVE_CACHE.as_ptr(), 
        mask:0, 
        magic:0, 
        shift:0
    };64];

    while index < 64{
        rook_magics[index] = SMagic{
            ptr: unsafe{ROOK_MOVE_CACHE.as_ptr().add(move_ptr)},
            mask: ROOK_BLOCKERS_MASK[index],
            magic: ROOK_RAW_MAGICS[index],
            shift: ROOK_RAW_SHIFTS[index] as u32,
        };
        move_ptr += 1 << (64-ROOK_RAW_SHIFTS[index]);
        index += 1;
    }

    return rook_magics;
}

pub const fn INITIALISE_BISHOP_MAGICS() -> [SMagic; 64]{
    let mut index : usize = 0;

    let mut move_ptr : usize = 0;

    let mut bishop_magics : [SMagic; 64] = [SMagic{
        ptr:BISHOP_MOVE_CACHE.as_ptr(), 
        mask:0, 
        magic:0, 
        shift:0
    };64];

    while index < 64{
        bishop_magics[index] = SMagic{
            ptr: unsafe{BISHOP_MOVE_CACHE.as_ptr().add(move_ptr)},
            mask: BISHOP_BLOCKERS_MASK[index],
            magic: BISHOP_RAW_MAGICS[index],
            shift: BISHOP_RAW_SHIFTS[index] as u32,
        };
        move_ptr += 1 << (64-BISHOP_RAW_SHIFTS[index]);
        index += 1;
    }

    return bishop_magics;
}

// masks for movement / direction
pub const WHITE_PAWN_ATTACK_MASK: [u64; 64] = GET_PAWN_ATTACK_MASK(true);
pub const BLACK_PAWN_ATTACK_MASK: [u64; 64] = GET_PAWN_ATTACK_MASK(false);

pub const KNIGHT_MOVE_MASK: [u64; 64] = GET_KNIGHT_MOVE_MASK();
pub const KING_MOVE_MASK: [u64; 64] = GET_KING_MOVE_MASK();
pub const ROOK_MOVE_MASK: [u64; 64] = GET_ROOK_MOVE_MASK();
pub const BISHOP_MOVE_MASK: [u64; 64] = GET_BISHOP_MOVE_MASK();
pub const QUEEN_MOVE_MASK: [u64; 64] = GET_QUEEN_MOVE_MASK();

pub fn get_Magic_Number(blocker_combinations: [u64; 20000]) -> (u64,u32){ 
    
    let mut best_magic_number:u64 = 0;
    let mut best_shift_size:u32 = 0;

    for shift_size in 49..64{
        let numberSize: u32 = 64 - shift_size;

        for _attempt_number in 0..1000000{
            let mut success = true;
            let random_magic = rand::thread_rng().gen_range(0..(!0));

            let mut temp_array= vec![false; 1<<numberSize as usize];

            let mut index = 0;
            while blocker_combinations[index as usize] != RANDOM_BITBOARD_IDENTIFIER{

                let bitboard_index = (blocker_combinations[index].wrapping_mul(random_magic)) >> shift_size;  

                // blocker collision
                if temp_array[bitboard_index as usize] == true{
                    success = false;
                    break;
                }

                temp_array[bitboard_index as usize] = true;


                index += 1;
            }

            if success{
                
                best_magic_number = random_magic;
                best_shift_size = shift_size;
                break;
            }
        }

        // could not find magic number
        if best_shift_size != shift_size{
            break;
        }

    }
    println!("0x{:X}|{}", best_magic_number, best_shift_size);

    return (best_magic_number, best_shift_size);
}

// piece movement functions

pub fn get_move_code(from_square: u8, to_square: u8) -> u16{
    return (from_square as u16) | ((to_square as u16) << 6);
}

pub fn get_move_code_special(from_square: u8, to_square: u8, special:u8) -> u16{
    return (from_square as u16) | ((to_square as u16) << 6) | ((special as u16) << 12);
}

pub const fn GET_MOVE_CODE_SPECIAL(from_square: u8, to_square: u8, special:u8) -> u16{
    return (from_square as u16) | ((to_square as u16) << 6) | ((special as u16) << 12);
}

pub fn is_move_normal(mv: u16) -> bool{
    return mv < 4096;
}

// Horizontal: true  Diagonal: false
pub fn get_direction(square1: u8, square2: u8) -> bool{
    return (square1 % 8 == square2 % 8) || (square1/8 == square2/8);
}

fn add_moves(
    move_vector: &mut Vec<u16>,
    from_square: u8,
    mut move_bitboard: u64,
){
  while move_bitboard != 0{
    let to_square : u8 = move_bitboard.trailing_zeros().try_into().unwrap();

    move_vector.push(get_move_code(from_square, to_square));

    move_bitboard ^= 1<<to_square;
  }
}

// rook and bishop move bitboard gen
pub fn get_rook_move_bitboard(
    square: usize,
    mut blockers: u64,
) -> u64{
    let magic_entry: &SMagic = unsafe { ROOK_SMAGICS.get_unchecked(square) };
    blockers &= magic_entry.mask;
    blockers = blockers.wrapping_mul(magic_entry.magic);
    blockers = blockers.wrapping_shr(magic_entry.shift);
    
    return unsafe { *(magic_entry.ptr).add(blockers as usize) };
}

pub fn get_bishop_move_bitboard(
    square: usize,
    mut blockers: u64,
) -> u64{
    let magic_entry: &SMagic = unsafe { BISHOP_SMAGICS.get_unchecked(square) };
    blockers &= magic_entry.mask;
    blockers = blockers.wrapping_mul(magic_entry.magic);
    blockers = blockers.wrapping_shr(magic_entry.shift);
    
    return unsafe { *(magic_entry.ptr).add(blockers as usize) };
}

pub fn get_queen_move_bitboard(
    square: usize,
    blockers: u64,
) -> u64{

    return get_bishop_move_bitboard(square, blockers) | get_rook_move_bitboard(square, blockers);
}

// Sliding Pieces

pub fn add_rook_moves(
    chess_board: &ChessBoard,
    move_vector: &mut Vec<u16>,
    square: u8,
){
    let blockers: u64 = chess_board.white_piece_bitboard | chess_board.black_piece_bitboard;

    let king_square: u8;
    let friendly_blockers: u64;
    if chess_board.board_color{
        king_square = chess_board.piece_bitboards[5].trailing_zeros() as u8;
        friendly_blockers = chess_board.white_piece_bitboard;
    }
    else{
        king_square = chess_board.piece_bitboards[11].trailing_zeros() as u8;
        friendly_blockers = chess_board.black_piece_bitboard;
    }

    let mut rook_move_bitboard: u64 = get_rook_move_bitboard(square as usize, blockers) & !friendly_blockers;

    // pin check
    if (chess_board.pin_mask & 1<<square)!= 0{
        rook_move_bitboard &= chess_board.pin_mask & ROOK_MOVE_MASK[king_square as usize];
    }

    // "check" check
    if chess_board.check_mask != 0{
        rook_move_bitboard &= chess_board.check_mask;
    }

    add_moves(move_vector, square, rook_move_bitboard);
}

pub fn add_bishop_moves(
    chess_board: &ChessBoard,
    move_vector: &mut Vec<u16>,
    square: u8,
){
    let blockers: u64 = chess_board.white_piece_bitboard | chess_board.black_piece_bitboard;

    let friendly_blockers : u64;
    let king_square: u8;
    if chess_board.board_color{
        friendly_blockers = chess_board.white_piece_bitboard;
        king_square = chess_board.piece_bitboards[5].trailing_zeros() as u8;
    }
    else{
        friendly_blockers = chess_board.black_piece_bitboard;
        king_square = chess_board.piece_bitboards[11].trailing_zeros() as u8;
    }

    let mut bishop_move_bitboard: u64 = get_bishop_move_bitboard(square as usize, blockers) & !friendly_blockers;


    // pin check
    if (chess_board.pin_mask & 1<<square)!= 0{
        bishop_move_bitboard &= chess_board.pin_mask & BISHOP_MOVE_MASK[king_square as usize];
    }

    // "check" check
    if chess_board.check_mask != 0{
        bishop_move_bitboard &= chess_board.check_mask;
    }

    add_moves(move_vector, square, bishop_move_bitboard);
}

pub fn add_queen_moves(
    chess_board: &ChessBoard,
    move_vector: &mut Vec<u16>, 
    square: u8, 
){
    let blockers : u64 = chess_board.white_piece_bitboard | chess_board.black_piece_bitboard;
    
    let friendly_blockers: u64;
    let king_square: u8;

    // get friendly blockers
    if chess_board.board_color{
        friendly_blockers = chess_board.white_piece_bitboard;
        king_square = chess_board.piece_bitboards[5].trailing_zeros() as u8;
    }
    else{
        friendly_blockers = chess_board.black_piece_bitboard;
        king_square = chess_board.piece_bitboards[11].trailing_zeros() as u8;
    } 

    let mut queen_move_bitboard: u64 = (get_bishop_move_bitboard(square as usize, blockers) | get_rook_move_bitboard(square as usize, blockers)) & !friendly_blockers;

    // pin check
    if (chess_board.pin_mask & 1<<square)!= 0{
        if get_direction(king_square, square){
            queen_move_bitboard &= chess_board.pin_mask & ROOK_MOVE_MASK[king_square as usize];
        }
        else{
            queen_move_bitboard &= chess_board.pin_mask & BISHOP_MOVE_MASK[king_square as usize];
        }

    }

    // "check" check
    if chess_board.check_mask != 0{
        queen_move_bitboard &= chess_board.check_mask;
    }

    add_moves(move_vector, square, queen_move_bitboard);
}

// Non-Sliding Pieces

pub fn add_knight_moves(
    chess_board: &ChessBoard,
    move_vector: &mut Vec<u16>, 
    square: u8, 
){
    // the knight is not pinned
    if (chess_board.pin_mask & 1<<square) == 0{
        let friendly_blockers: u64;

        // get friendly blockers
        if chess_board.board_color{
            friendly_blockers = chess_board.white_piece_bitboard;
        }
        else{
            friendly_blockers = chess_board.black_piece_bitboard;
        } 

        let mut move_bitboard:u64 = KNIGHT_MOVE_MASK[square as usize] & !friendly_blockers; 

        // "check" check
        if chess_board.check_mask != 0{
            move_bitboard &= chess_board.check_mask;
        }

        add_moves(move_vector, square, move_bitboard);
    }
}

pub fn add_king_moves(
    chess_board: &ChessBoard,
    move_vector: &mut Vec<u16>, 
    square: u8, 
){
    let friendly_blockers: u64;
    if chess_board.board_color{
        friendly_blockers = chess_board.white_piece_bitboard;
    }
    else{
        friendly_blockers = chess_board.black_piece_bitboard;
    }

    let move_bitboard:u64 = KING_MOVE_MASK[square as usize] & !chess_board.attack_mask & !friendly_blockers;
    
    add_moves(move_vector, square, move_bitboard);
}

fn add_pawn_promotion_moves(from_square: u8, to_square: u8, move_vector: &mut Vec<u16>){
    move_vector.push(get_move_code_special(from_square, to_square, 5));
    move_vector.push(get_move_code_special(from_square, to_square, 6));
    move_vector.push(get_move_code_special(from_square, to_square, 7));
    move_vector.push(get_move_code_special(from_square, to_square, 8));
}

fn add_moves_pawn_promotion(
    move_vector: &mut Vec<u16>,
    from_square: u8,
    mut move_bitboard: u64,
){
  while move_bitboard != 0{
    let to_square : u8 = move_bitboard.trailing_zeros().try_into().unwrap();

    add_pawn_promotion_moves(from_square, to_square, move_vector);

    move_bitboard ^= 1<<to_square;
  }
}

// this assumes that it is possible for double move
pub fn add_pawn_double_move(
    chess_board: &ChessBoard,
    move_vector: &mut Vec<u16>, 
    square: u8,
){
    let mut move_bitboard = 0;
    if chess_board.board_color{
        // no blockers in the way
        if ((1 << (square-16)) | (1<<(square-8))) & chess_board.all_piece_bitboard == 0{
            move_bitboard |= 1 << (square-16);
        }
    }
    else{
        // no blockers in the way
        if ((1 << (square+16)) | (1<<(square+8))) & chess_board.all_piece_bitboard == 0{
            move_bitboard |= 1 << (square+16);
        }
    }

    if move_bitboard == 0{
        return;
    }

    if 1 << square & chess_board.pin_mask != 0{
        move_bitboard &= chess_board.pin_mask;
    }
    
    // check masking
    if chess_board.check_mask != 0{
        move_bitboard &= chess_board.check_mask;
    }

    if move_bitboard != 0{
        if chess_board.board_color{
            move_vector.push(get_move_code_special(square, square-16, 2));
        }
        else{
            move_vector.push(get_move_code_special(square, square+16, 2));
        }
    }
}

pub fn add_pawn_moves(
    chess_board: &ChessBoard,
    move_vector: &mut Vec<u16>, 
    square: u8, 
){
    let mut move_bitboard: u64 = 0;

    // white
    if chess_board.board_color{

        // // pawn double move row
        // if square >= 48 && square <= 55{
        //     // nothing blocking it from moving double
        //     if 1 << (square - 16) & chess_board.all_piece_bitboard == 0{
        //         move_bitboard |= 1 << (square - 16);
        //     }
        // }

        // nothing blocking it from moving forward
        if 1 << (square - 8) & chess_board.all_piece_bitboard == 0{
            // we temp ignore promotions since we are limited by the bitboard
            move_bitboard |= 1 << (square - 8);
            
        }

        // can capture piece on the right
        if (square % 8 < 7) && 1 << (square - 7) & chess_board.black_piece_bitboard != 0{
            move_bitboard |= 1 << (square - 7);
        }

        // can capture piece on the left
        if (square % 8 > 0) && 1 << (square - 9) & chess_board.black_piece_bitboard != 0{
            move_bitboard |= 1 << (square - 9);
        }
    }

    // black
    else{
        // // pawn double move row
        // if square >= 8 && square <= 15{
        //     // nothing blocking it from moving double
        //     if 1 << (square + 16) & chess_board.all_piece_bitboard == 0{
        //         move_bitboard |= 1 << (square+ 16);
        //     }
        // }

        // nothing blocking it from moving forward
        if 1 << (square + 8) & chess_board.all_piece_bitboard == 0{
            move_bitboard |= 1 << (square + 8);
        }

        // can capture piece on the left
        if (square % 8 > 0) && 1 << (square + 7) & chess_board.white_piece_bitboard != 0{
            move_bitboard |= 1 << (square + 7);
        }

        // can capture piece on the right
        if (square % 8 < 7) && 1 << (square + 9) & chess_board.white_piece_bitboard != 0{
            move_bitboard |= 1 << (square + 9);
        }
    }

    // pin masking
    if 1 << square & chess_board.pin_mask != 0{
        move_bitboard &= chess_board.pin_mask;

        let mut king_square = chess_board.piece_bitboards[5].trailing_zeros() as u8;
        if !chess_board.board_color{
            king_square = chess_board.piece_bitboards[11].trailing_zeros() as u8;
        }

        // this additional mask is to prevent clashing
        // bishop direction
        if !get_direction(square, king_square){
            move_bitboard &= BISHOP_MOVE_MASK[king_square as usize];
        }
    }
    
    // check masking
    if chess_board.check_mask != 0{
        move_bitboard &= chess_board.check_mask;
    }

    // if one of the moves is a promotion, we can assumne all are... somehow
    if move_bitboard & PROMOTION_BITBOARD_MASK != 0{
        add_moves_pawn_promotion(move_vector, square, move_bitboard);
    }
    else{
        add_moves(move_vector, square, move_bitboard);
    }
}