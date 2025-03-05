use std::io::{self, Write};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use std::ops::Neg;
use std::fs;


use crate::app_settings::*;
use crate::move_compute::*;
use crate::functions::*;
use crate::board::*;
use crate::evaluation::*;
use crate::game_board::*;
use crate::zobrist_hash::*;
use crate::transposition_table::*;
use crate::timer::Timer;


#[derive(Copy, Clone)]
pub struct MoveScorePair{
    pub mv: u16,
    pub score: i16,
    pub is_exact: bool,
}

impl MoveScorePair {
    fn new(mv: u16, score: i16, is_exact: bool) -> MoveScorePair {
        MoveScorePair { mv, score, is_exact }
    }
}

impl Neg for MoveScorePair {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            mv: self.mv,
            score: -self.score,
            is_exact: self.is_exact,
        }
    }
}

pub struct MoveWeightPair{
    pub mv: u16,
    pub weight: i16,
}

impl MoveWeightPair {
    fn new(mv: u16, weight: i16) -> MoveWeightPair {
        MoveWeightPair { mv, weight }
    }
}

fn get_move_weight(mv: u16, board: &ChessBoard) -> i16{
    let to_square: usize = ((mv >> 6) & MOVE_DECODER_MASK) as usize;
    let mut weight : i16 = 0;

    // is a piece capture
    if board.all_piece_bitboard & 1<<to_square != 0{
        weight += 1;

        let from_square: usize = (mv & MOVE_DECODER_MASK) as usize;

        let piece_captured: u8 = board.piece_array[to_square] % 6;
        let piece_moved: u8 = board.piece_array[from_square] % 6;

        if (piece_captured) > (piece_moved){
            weight += 1;
        }

        else{
            // depending on how large the difference is
            weight -= (piece_moved-piece_captured) as i16;
        }
    }

    // going to an attacked square
    if board.attack_mask & (1 << to_square) != 0{
        weight -= 1;
    }

    return weight;
}

fn sort_move_vec(move_vec_sorted: &mut Vec<MoveWeightPair>, move_vec: &Vec<u16>, chess_board: &ChessBoard){
    for mv in move_vec{
        move_vec_sorted.push(MoveWeightPair::new(*mv, get_move_weight(*mv, chess_board)));
    }

    // descending order since we want best weights first
    move_vec_sorted.sort_by(|a, b| b.weight.cmp(&a.weight));
}

// Encoding Scheme (not UCI for reasons)
// for more information -> main.py

// server - this program
// client - chess engine program

// server -> client
// 0 - end program
// 1 - send fen string and initialisation text
// 2 - wait for server
// 3 - inform move

// client -> server
// 4 - client ready confirmation
// 5 - supply move
// 6 - wait for client

pub fn chess_battle(){
    println!("CHESS BATTLE: VERSION {}", ENGINE_VERSION);

    let mut file_path_choice: String = String::new();

    print!("filepath (0/1) >>");
    io::stdout().flush().unwrap();
    
    io::stdin()
    .read_line(&mut file_path_choice)
    .expect("Failed to read line");

    file_path_choice = file_path_choice.trim().to_string();

    let file_path : String;

    if file_path_choice == "0"{
        file_path = "ChessBot1.txt".to_string();
    }
    else{
        file_path = "ChessBot2.txt".to_string();
    }

    let mut running: bool = true;
    
    // tells the bot when to move
    let mut move_now : bool = false;

    let mut game_board : GameChessBoard = create_empty_GameChessBoard(); 

    while running{
        let mut contents = String::new();

        // do it in a scope so file is closed
        {
            contents = fs::read_to_string(file_path.clone()).unwrap();
        }

        if contents == ""{
            continue
        }

        let command_char : char = contents.chars().nth(0).unwrap();

        let command : i32 = command_char.to_digit(10).unwrap() as i32;

        // close program
        if command == 0{
            {
                // tell it that it has no life plans
                fs::write(file_path.clone(), "4|NANANA");
            }

            running = false;
        }

        // initialisation text
        else if command == 1{
            
            let move_turn_char : char = contents.chars().nth(2).unwrap();

            let fen_string : &str = &contents[3..contents.len()];

            game_board = fen_to_GameChessBoard(fen_string);

            println!("GAME BOARD INITIALISED");
            print_game_board(&game_board);

            // to move
            if move_turn_char == '1'{
                {
                    // thinking
                    fs::write(file_path.clone(), "6|NANANA");
                }

                move_now = true;
            }
            else{
                {
                    // ready confirmation
                    fs::write(file_path.clone(), "4|NANANA");
                }
            }
        }

        // received opponents move
        else if command == 3{
            let move_code_string : &str = &contents[2..contents.len()];

            let move_code : u16 = move_code_string.parse().unwrap();

            println!("Received move: {}", get_move_string(move_code));

            game_make_move(&mut game_board, move_code);

            {
                // thinking
                fs::write(file_path.clone(), "6|NANANA");
            }

            move_now = true;
        }

        // should move now
        if move_now{
            let mvel_pair : MoveScorePair = get_best_move(&mut game_board, BATTLE_THINK_TIME as u32);

            println!("Move: {} {}", get_move_string(mvel_pair.mv), mvel_pair.score);            

            game_make_move(&mut game_board, mvel_pair.mv);

            // print_game_board(&game_board);
            
            // print_game_tree(&game_board);

            {
                // update the fen position lookup
                fs::write("FenPositionLookup.txt", format!("{}|{}", get_gamestate(&mut game_board), board_to_fen(&game_board.board)));

                // provide move
                fs::write(file_path.clone(), format!("5|{}",mvel_pair.mv));
            }

            move_now = false;
        }

        
    }
}

// gets the number of nodes given an iteration depth
pub fn perft(board: &mut ChessBoard, depth: u16) -> u32{
    // base case
    if depth == 0{
        get_board_score(&board);
        return 1;
    }

    let mut move_vec: Vec<u16> = Vec::new();

    get_moves(board, &mut move_vec);

    let mut node_num: u32 = 0;
    
    for mv in move_vec{
        let mut sub_board: ChessBoard = board.clone();

        make_move(&mut sub_board, mv);

        node_num += perft(&mut sub_board, depth - 1);
    } 

    return node_num;
}

pub fn sub_perft(board: &mut ChessBoard, depth: u16){
    let mut move_vec: Vec<u16> = Vec::new();

    get_moves(board, &mut move_vec);

    let mut total_counter: u32 = 0;

    for mv in move_vec{
        let mut sub_board: ChessBoard = board.clone();

        make_move(&mut sub_board, mv);

        let move_num: u32 = perft(&mut sub_board, depth - 1);

        total_counter += move_num;
        println!("{}|{}",get_move_string(mv), move_num);
    } 

    println!("Total: {}", total_counter);
}

pub fn debug_zobrist_hash(board: &mut ChessBoard, depth:u16){
    // base case
    if depth == 0{
        return;
    }

    let mut move_vec: Vec<u16> = Vec::new();

    get_moves(board, &mut move_vec);

    let mut node_num: u32 = 0;
    
    for mv in move_vec{
        let mut sub_board: ChessBoard = board.clone();

        make_move(&mut sub_board, mv);

        if sub_board.zobrist_hash != get_full_zobrist_hash(&sub_board){
            println!("{} {}", sub_board.zobrist_hash, get_full_zobrist_hash(&sub_board));
            println!("prev fen:{} mv:{}", board_to_fen(&board), get_move_string(mv));
            return;
        }

        debug_zobrist_hash(&mut sub_board, depth - 1);
    } 
}

// 8/8/2K5/k7/8/8/1Q6/8/ w - - 0 1
pub fn debug(game_board: &mut GameChessBoard){
    let mut input_string: String = String::new();

    let mut debug_running: bool = true;

    println!("
COMMANDS
quit - quit
version - version
engine info - shows engine info
show - show curr board
show info - show curr board info
battle - initiate battles
fen - create curr board with fen
move - make normal move
move sequence - make a sequence of moves
move special - make special move (pawn double, ep, promotion, castling)
show moves - show possible moves
show perft - show perft
show eval - shows curr evaluation
debug z - debug zobrist
query tt - find entry in TT
best move - gets best move
bench -best - best move bench
bench -perft - perft bench
bench -single -perft - benches current board perft
    ");

    while debug_running{
        input_string.clear();

        print!(">>");
        io::stdout().flush().unwrap();
        
        io::stdin()
        .read_line(&mut input_string)
        .expect("Failed to read line");

        input_string = input_string.trim().to_string();

        if input_string == "quit"{
            debug_running = false;  
        }

        else if input_string == "version"{
            println!("VERSION {}", ENGINE_VERSION);
        }

        else if input_string == "engine info"{
            println!(
                "
ENGINE INFO:
DEFAULT THINK TIME: {}ms
DEFAULT FEN: {}
MAX SEARCH DEPTH: {}
QUISCENCE SEARCH DEPTH: {}
MAX SEARCH EXTENSION: {}
TRANSPOSITION TABLE SIZE: {}MB
VERSION: {}

BATTLE INFO:
BATTLE THINK TIME: {}ms
BATTLE MOVE LIMIT: {}
                ",
                DEFAULT_THINK_TIME,
                DEFAULT_FEN,
                MAX_SEARCH_DEPTH,
                QUIESCENCE_DEPTH_LIMIT,
                MAX_SEARCH_EXTENSION,
                TRANSPOSITION_TABLE_SIZE * 104 / 8 / (1024*1024),
                ENGINE_VERSION,

                BATTLE_THINK_TIME,
                MOVE_LIMIT_MAX

            )
        }

        else if input_string == "show"{
            print_game_board(&game_board);
        }

        else if input_string == "show info"{
            print_game_board_info(&game_board);
        }

        else if input_string == "battle"{
            chess_battle();

            // automatically exit after finished
            debug_running = false;
        }

        else if input_string == "fen"{
            input_string.clear();
            print!("fen >>");
            io::stdout().flush().unwrap();
            
            io::stdin()
            .read_line(&mut input_string)
            .expect("Failed to read line");

            input_string = input_string.trim().to_string();

            if input_string == "default"{
                *game_board = fen_to_GameChessBoard(&DEFAULT_FEN);
            }
            else{
                *game_board = fen_to_GameChessBoard(&input_string);
            }
        }

        else if input_string == "move"{
            input_string.clear();
            print!("move >>");
            io::stdout().flush().unwrap();
            
            io::stdin()
            .read_line(&mut input_string)
            .expect("Failed to read line");

            input_string = input_string.trim().to_string();
            let coord_1: u8 = coord_to_number(&input_string[0..2]);
            let coord_2: u8 = coord_to_number(&input_string[2..4]);

            let mv: u16 = get_move_code(coord_1, coord_2);

            game_make_move(game_board, mv)
        }

        else if input_string == "move special"{
            input_string.clear();
            print!("move >>");
            io::stdout().flush().unwrap();
            
            io::stdin()
            .read_line(&mut input_string)
            .expect("Failed to read line");

            input_string = input_string.trim().to_string();
            let coord_1: u8 = coord_to_number(&input_string[0..2]);
            let coord_2: u8 = coord_to_number(&input_string[2..4]);

            

            input_string.clear();
            print!("special >>");
            io::stdout().flush().unwrap();
            
            io::stdin()
            .read_line(&mut input_string)
            .expect("Failed to read line");
            input_string = input_string.trim().to_string();


            let mv: u16 = get_move_code_special(coord_1, coord_2, input_string.parse::<u8>().unwrap());

            game_make_move(game_board, mv);
        }

        else if input_string == "show moves"{
            let mut move_vec: Vec<u16> = Vec::new();

            get_moves(&mut game_board.board, &mut move_vec);

            print_moves(&move_vec);
        }

        else if input_string == "show perft"{
            input_string.clear();
            print!("depth >>");
            io::stdout().flush().unwrap();
            
            io::stdin()
            .read_line(&mut input_string)
            .expect("Failed to read line");

            let depth: u16 = input_string.trim().parse().expect("cannot parse string to int");

            sub_perft(&mut game_board.board, depth);
        }

        else if input_string == "show eval"{
            println!("Evaluation (rel): {}", get_board_score(&game_board.board));
        }

        else if input_string == "debug z"{
            input_string.clear();
            print!("depth >>");
            io::stdout().flush().unwrap();
            
            io::stdin()
            .read_line(&mut input_string)
            .expect("Failed to read line");

            let depth: u16 = input_string.trim().parse().expect("cannot parse string to int");

            debug_zobrist_hash(&mut game_board.board, depth);
        }

        else if input_string == "best move"{
            input_string.clear();
            print!("think time (ms) >>");
            io::stdout().flush().unwrap();
            
            io::stdin()
            .read_line(&mut input_string)
            .expect("Failed to read line");

            let think_time: u32 = input_string.trim().parse().expect("cannot parse string to int");

            let best_move: MoveScorePair = get_best_move(game_board, think_time);

            // println!("evaluation: {} score: {} move line: {}", get_move_string(best_move.mv), best_move.score, get_move_line_vec_string(&get_move_line(game_board)));
        }

        else if input_string == "move sequence"{
            input_string.clear();
            print!("move sequence >>");
            io::stdout().flush().unwrap();
            
            io::stdin()
            .read_line(&mut input_string)
            .expect("Failed to read line");

            let move_vec = split(input_string.trim());

            for mv_str in &move_vec{
                let coord_1: u8 = coord_to_number(&mv_str[0..2]);
                let coord_2: u8 = coord_to_number(&mv_str[2..4]);

                let mv: u16 = get_move_code(coord_1, coord_2);

                println!("mv: {}", get_move_string(mv));

                make_move(&mut game_board.board, mv);
                add_to_game_tree(&mut game_board.game_tree, game_board.board.zobrist_hash);

                
                print_game_board(&game_board);
            }
        }

        else if input_string == "query tt"{
            input_string.clear();
            print!("zobrist hash >>");
            io::stdout().flush().unwrap();
            
            io::stdin()
            .read_line(&mut input_string)
            .expect("Failed to read line");

            let zob_hash: u64;

            if input_string.trim() == "curr"{
                zob_hash = game_board.board.zobrist_hash;
            }
            else{
                zob_hash = input_string.trim().parse().expect("cannot parse string to int");
            }

            
            input_string.clear();

            print!("rep count >>");
            io::stdout().flush().unwrap();
            
            io::stdin()
            .read_line(&mut input_string)
            .expect("Failed to read line");

            let rep_count: u8 = input_string.trim().parse().expect("cannot parse string to int");

            let true_hash = zob_hash ^ REPETITION_COUNT_HASHES[rep_count as usize]; 

            let tt_entry = game_board.transposition_table.get(true_hash);

            tt_entry.print_entry();


            
        }

        else if input_string == "bench -best"{
            position_bench(0);
        }

        else if input_string == "bench -perft"{
            position_bench(1);
        }

        else if input_string == "bench -single -perft"{
            
            for perft_depth in 1..=5{
                let t_start = Instant::now();
        
                let node_num = perft(&mut game_board.board, perft_depth);

                let mut time_taken = t_start.elapsed().as_millis();

                
                let node_rate :u128; 
                if time_taken == 0{
                    node_rate = 0;
                }
                else{
                    node_rate = node_num as u128 /time_taken * 1000;
                }
                
                println!("depth: {} took: {}ms nds:{} nds/s: {}", perft_depth, time_taken, node_num, node_rate);

            }
            
        }
    }
}

pub fn position_bench(flag: u8){
    let mut fen_pos_array: Vec<String> = vec![
        "r2qkb1r/pb3ppp/1pn1pn2/2p5/2pP4/PP3NP1/4PPBP/RNBQ1RK1 w kq - 0 9".to_string(),
        "r1b2rk1/1pqp1ppp/p3pn2/P7/1b2P3/1NNPBP2/1P4PP/R2QK2R w KQ - 1 13".to_string(),
        "r2q1rk1/pp2bppp/2n2n2/3pN3/3P4/7P/PP2NPP1/R1BQ1RK1 w - - 1 13".to_string(),
        "r1b1k2r/p1qp1ppp/2p1pn2/2b5/4P3/6P1/PPP2PBP/RNBQ1RK1 w kq - 0 9".to_string(),
        "1r2r1k1/pbqn1pp1/1ppbpn1p/3p4/2PP4/1PN1PNPP/PBQ2PBK/R2R4 w - - 1 15".to_string(),
        "rnb2rk1/pp3ppp/2p1pq2/8/1bpP4/2N1PN2/PP2BPPP/R2QK2R w KQ - 0 9".to_string(),
        "rn2k2r/pbp2ppp/1p2p2q/4P1n1/2pP4/2PB1PP1/PB1Q3P/R3K1NR w KQkq - 0 13".to_string(),
        "rnbqr3/bp3ppk/p1pp1n1p/4p3/1PP5/P1NP1NP1/3BPPBP/R1Q2RK1 w - - 3 12".to_string(),
        "r2q1rk1/pb1nbppp/1p2pn2/8/3p4/2NBPN2/PP1BQPPP/2R1R1K1 w - - 0 13".to_string(),
        "r1bq1rk1/pp1nbppp/5n2/2Ppp3/1P3B2/P1N1P2P/5PP1/R2QKBNR w KQ - 0 11".to_string(),
        "r1bqr1k1/ppp2pp1/1b1p1n1p/4p3/1PBnP3/P1NP1N1P/2P2PP1/R1BQR1K1 w - - 3 11".to_string(),
        "r2qkb1r/p2nnp2/bp2p1p1/2ppP2p/3P1P1P/2P2NP1/PP4B1/RNBQR2K w kq - 3 13".to_string(),
        "r1bq1rk1/pp1nbpp1/2n1p2p/3p4/3P3B/3BPN2/PP1N1PPP/R2Q1RK1 w - - 5 11".to_string(),
        "r2q1rk1/1p1nbppp/p1p5/2Pp1b2/1P1P1B2/2R2N1P/P3BPP1/3Q1RK1 w - - 1 16".to_string(),
        "r3k1nr/pp3ppp/n3b3/1N2p3/2p1P3/8/PP1K1PPP/R2N1B1R w kq - 1 11".to_string(),
        "r2q1rk1/1bpn1pbp/pp1ppnp1/8/3PPP2/2N2N2/PPP1BBPP/R2QR1K1 w - - 0 11".to_string(),
        "2kr1b1r/pbq2pp1/1pn1pn2/2p4p/4PPP1/2P3NP/PP1NQ1B1/R1B2RK1 w - - 0 15".to_string(),
        "r1b1r1k1/ppp1qppp/4p1n1/7n/2BP4/2N2NQ1/P1P2PPP/1R2R1K1 w - - 0 17".to_string(),
        "rn1k1b1r/pp4pp/4B3/2pP4/Q1P5/N4b1P/Pq2p1P1/R3R1K1 w - - 0 17".to_string(),
        "rn2kb1r/pp1qpppp/6b1/3pB3/2BP4/5PN1/PPP3PP/R2QK2R w KQkq - 0 13".to_string(),
        "r1b1k2r/2qn1ppp/2p5/p1b1pP2/1pN1P1n1/3B1N2/PPPBQ2P/R3K2R w KQkq - 0 15".to_string(),
    ];  

    for fen_pos in fen_pos_array{
        let mut game_board : GameChessBoard = fen_to_GameChessBoard(&fen_pos);
        
        let mut node_counter = 0;

        let t_start = Instant::now();
        
        if flag == 0{
            get_best_move_negamax(&mut game_board.board, &mut game_board.game_tree, &mut game_board.transposition_table, 5, 0, 0, -INF, INF, &Timer::new(Duration::from_secs(10)), &mut node_counter);
        }
        else if flag == 1{
            
            node_counter = perft(&mut game_board.board, 5);
            
        }

        let time_taken = t_start.elapsed().as_millis();

        
        println!("test: {} took: {}ms nds:{} nds/s: {}", fen_pos, time_taken, node_counter, node_counter as u128 /time_taken * 1000);
        
    }
}

pub fn get_move_line(game_chess_board: &GameChessBoard) -> Vec<u16>{
    let mut chess_board_copy = game_chess_board.board.clone();

    let mut move_line_vec : Vec<u16> = Vec::new();

    let chess_board_repetition: u8 = get_position_counter(&game_chess_board.game_tree, chess_board_copy.zobrist_hash);

    let true_hash = chess_board_copy.zobrist_hash ^ REPETITION_COUNT_HASHES[chess_board_repetition as usize];

    let starting_tt = game_chess_board.transposition_table.get(true_hash);

    make_move(&mut chess_board_copy, starting_tt.best_move);

    move_line_vec.push(starting_tt.best_move);

    for i in 0..starting_tt.depth()-1{
        let chess_board_repetition: u8 = get_position_counter(&game_chess_board.game_tree, chess_board_copy.zobrist_hash);

        if game_chess_board.transposition_table.contains(chess_board_copy.zobrist_hash){
            let tt_entry = game_chess_board.transposition_table.get(chess_board_copy.zobrist_hash);

            make_move(&mut chess_board_copy, tt_entry.best_move);
            move_line_vec.push(tt_entry.best_move);

        }

        // couldnt find an entry (which should be strange but whatever)
        else{
            break;
        }
    }

    return move_line_vec;
}

pub fn debug_print(s: &str, ply: u8){
    // Open a file with append option
    let mut data_file = fs::OpenOptions::new()
        .append(true)
        .open("debug.txt")
        .expect("cannot open file");

    for i in 0..ply{
        data_file
        .write(b"\t")
        .expect("write failed");
    }

    
    
    // Write to a file
    data_file
        .write(format!("{}\n", s).as_bytes())
        .expect("write failed");
}

pub fn debug_log_str(log_str: &mut String, logs: &str, ply: u8){
    for i in 0..ply{
        log_str.push('>');
    }

    log_str.push_str(logs);
    log_str.push('\n')
}

pub fn add_log(logs: &str){
    let mut data_file = fs::OpenOptions::new()
        .append(true)
        .open("debug.txt")
        .expect("cannot open file");
    
    // Write to a file
    data_file
        .write(logs.as_bytes())
        .expect("write failed");
}

pub fn debug_log(logs: &str,  ply: u8){
    // Open a file with append option
    let mut data_file = fs::OpenOptions::new()
        .append(true)
        .open("debug.txt")
        .expect("cannot open file");

    for i in 0..ply{
        data_file
        .write(b">")
        .expect("write failed");
    }
    
    
    
    // Write to a file
    data_file
        .write(format!("{}\n", logs).as_bytes())
        .expect("write failed");
}

pub const CHECKMATE_SCORE: i16 = 9990;
pub const STATIC_MOVE_PRUNING_MARGIN : i16 = 150;

pub fn discredit_score(score: i16) -> i16{
    // probably a checkmate
    if score < -CHECKMATE_SCORE || score > CHECKMATE_SCORE{
        if score > 0{
            return score - 1;
        }
        else{
            return score + 1;
        }
    }

    return score;
}

pub fn get_search_extention(board: &ChessBoard) -> bool{
    // if the enemy is in check
    if board.check_mask != 0{
        return true;
    }
    
    false
}

const INF: i16 = 32767;

pub fn get_best_move(game_chess_board: &mut GameChessBoard, time_alloc: u32) -> MoveScorePair{
    let best_move = iterative_deepening(&mut game_chess_board.board, &mut game_chess_board.game_tree, &mut game_chess_board.transposition_table, time_alloc);

    // println!("{}", get_move_line_vec_string(&get_move_line(game_chess_board)));
    return best_move;
}

pub const SCORE_EXACT_TYPE: bool = true;
pub const SCORE_NOT_EXACT_TYPE: bool = false;

pub const EVAL_NODE_TYPE: u8 = 1;
pub const CUT_NODE_TYPE : u8 = 2;
pub const TT_NODE_TYPE : u8 = 3;

// testing fens:
// 2Q2nk1/p4p1p/1p2rnp1/3p4/3P3q/BP6/P2N4/2K2R b - - - 

// heavily inspired by pleco engine... again
pub fn iterative_deepening(chess_board: &mut ChessBoard, game_tree: &mut HashMap<u64, u8>, transposition_table: &mut TranspositionTable, time_alloc: u32) -> MoveScorePair{
    let timer: Timer = Timer::new(Duration::from_millis(time_alloc as u64));

    let depth: u8 = 7;
    let mut best_mvel = MoveScorePair::new(0, -INF, SCORE_EXACT_TYPE);

    let mut alpha : i16 = -INF;
    let mut beta : i16 = INF;

    let mut curr_depth = 1;

    let mut move_vec_unsorted: Vec<u16> = Vec::new();

    let mut node_counter = 0;

    get_moves(chess_board, &mut move_vec_unsorted);

    let mut move_vec_sorted: Vec<MoveWeightPair> = Vec::new();

    sort_move_vec(&mut move_vec_sorted, &move_vec_unsorted, chess_board);

    while curr_depth < MAX_SEARCH_DEPTH{
        node_counter = 0;

        // debug_print(&"{", curr_depth);
        if timer.time_out(){
            break;
        }
        // Search Starts here
        let mut best_mvel_search_pair : MoveScorePair = MoveScorePair::new(0, -INF, SCORE_EXACT_TYPE);

        for mut mv_weight_pair in &mut move_vec_sorted{
            let mv = mv_weight_pair.mv;

            
            let mut sub_board: ChessBoard = chess_board.clone();
            let mvel_pair: MoveScorePair;

            make_move(&mut sub_board, mv);

            add_to_game_tree(game_tree, sub_board.zobrist_hash);

            mvel_pair = -get_best_move_negamax(&mut sub_board, game_tree, transposition_table, curr_depth - 1, 1, 0, -beta, -alpha, &timer, &mut node_counter);

            remove_from_game_tree(game_tree, sub_board.zobrist_hash);

            if timer.time_out(){
                break;
            }

            mv_weight_pair.weight = mvel_pair.score;

            if mvel_pair.score > best_mvel_search_pair.score{
                best_mvel_search_pair.score = mvel_pair.score;
                best_mvel_search_pair.mv = mv;

                // update the best line
    
                if mvel_pair.score > alpha{
                    alpha = mvel_pair.score;
                }
            }
        }
        // Move Search Ends here


        // doesnt add anything to the move vec and just sorts the changed values
        sort_move_vec(&mut move_vec_sorted, &Vec::new(), chess_board);

        if best_mvel_search_pair.score > beta{
            println!("RESTART SEARCH - BETA");
            beta = INF;
        }
        else if best_mvel_search_pair.score < alpha{
            println!("RESTART SEARCH - ALPHA");
            alpha = -INF;
        }
        else{
            // move was not null
            if best_mvel_search_pair.mv != 0{
                
                best_mvel = best_mvel_search_pair;

                println!("DEPTH SEARCHED TO {} a:{} b:{} nodes:{} best move: {} eval: {}",curr_depth, alpha, beta, node_counter, get_move_string(best_mvel.mv), best_mvel.score);

                alpha = -INF;
                beta = INF;

                // alpha = best_mvel.score - 35;
                // beta = best_mvel.score + 35;

                curr_depth += 1;
            }
        }
    }

    let chess_board_repetition: u8 = get_position_counter(game_tree, chess_board.zobrist_hash);

    let true_hash = chess_board.zobrist_hash ^ REPETITION_COUNT_HASHES[chess_board_repetition as usize];

    transposition_table.add(true_hash, discredit_score(best_mvel.score), curr_depth, EXACT_BOUND, best_mvel.mv);

    println!("{}%", transposition_table.capacity() * 100.0);

    return best_mvel;
}

// stolen I mean borrowed from the blunder engine
const FUTILITY_MARGINS: [i16; 9] = [
	0,
	100, // depth 1
	160, // depth 2
	220, // depth 3
	280, // depth 4
	340, // depth 5
	400, // depth 6
	460, // depth 7
	520, // depth 8
];


pub fn get_best_move_negamax(chess_board: &mut ChessBoard, game_tree: &mut HashMap<u64, u8>, transposition_table: &mut TranspositionTable, mut depth: u8, ply: u8, mut search_extention_counter: u8, mut alpha: i16, mut beta: i16, timer: &Timer, node_counter: &mut u32) -> MoveScorePair{

    // check every 2048 nodes if our time runs out
    // heavily inspired by the blunder engine
    if (*node_counter & 2047) == 0{
        if timer.time_out(){
            return MoveScorePair::new(0, -INF, SCORE_NOT_EXACT_TYPE);
        }
    }

    *node_counter += 1;
    
    let chess_board_repetition: u8 = get_position_counter(game_tree, chess_board.zobrist_hash) - 1;
    
    let true_hash = chess_board.zobrist_hash ^ REPETITION_COUNT_HASHES[chess_board_repetition as usize];
    

    let tt_entry = transposition_table.get(true_hash);

    // extra check to make sure we have a valid collision
    if tt_entry.hash == true_hash{
        // larger / equal search
        if tt_entry.depth() >= depth{
            let entry_type = tt_entry.entry_type();

            if entry_type == EXACT_BOUND{
                return MoveScorePair::new(0, tt_entry.score, SCORE_EXACT_TYPE);
            }

            else if entry_type == LOWER_BOUND && tt_entry.score >= beta{
                return MoveScorePair::new(0, tt_entry.score, SCORE_NOT_EXACT_TYPE);
            }

            else if entry_type == UPPER_BOUND && tt_entry.score <= alpha{
                return MoveScorePair::new(0, tt_entry.score, SCORE_NOT_EXACT_TYPE);
            }
        }
    }  
    
    
    if depth == 0{
        let qsearch_score = quiescence_search(chess_board, alpha, beta, QUIESCENCE_DEPTH_LIMIT);

        return qsearch_score;
    }
    
    
    let mut best_mvel_pair : MoveScorePair = MoveScorePair::new(0, -INF, SCORE_EXACT_TYPE);

    // upper bound
    let mut tt_entry_type: u8 = UPPER_BOUND;

    let mut move_vec_unsorted: Vec<u16> = Vec::new();

    get_moves(chess_board, &mut move_vec_unsorted);

    // let isPVNode = beta-alpha != 1;


    // extend search
    if search_extention_counter < MAX_SEARCH_EXTENSION{
        if get_search_extention(chess_board){
            search_extention_counter += 1;
            depth += 1;
        }
    }

    // Static move prunign
    // if !in_check && beta.abs() < CHECKMATE_SCORE && !isPVNode{
    //     let static_score = get_board_score(chess_board);

    //     let score_margin = STATIC_MOVE_PRUNING_MARGIN * depth as i16;

    //     if static_score - score_margin >= beta{
    //         return MoveScorePair::new(0, static_score - score_margin, SCORE_NOT_EXACT_TYPE);;
    //     }
    // }

    // razoring

    // let in_check = chess_board.check_mask != 0;
    
    // if depth <= 2 && !in_check && !isPVNode{
    //     let static_score = get_board_score(chess_board);

    //     if static_score + FUTILITY_MARGINS[depth as usize] * 3 < alpha{
    //         let score = quiescence_search(chess_board, alpha, beta, QUIESCENCE_DEPTH_LIMIT);

    //         if score.score < alpha {
    //             // println!("razor worked");
	// 			return score;
	// 		}
    //     }
    // }

    // no legal moves
    if move_vec_unsorted.len() == 0{
        // stalemate
        if chess_board.check_mask == 0{
            best_mvel_pair.score = 0;
        }
        
        // checkmate
        else{
            // shift the checkmate so closer checkmates are preffered
            best_mvel_pair.score = -10000 + ply as i16;
            
        }

        return best_mvel_pair
    }

    let mut move_vec_sorted: Vec<MoveWeightPair> = Vec::new();
    sort_move_vec(&mut move_vec_sorted, &move_vec_unsorted, chess_board);

    for mv_weight_pair in move_vec_sorted{
        let mv = mv_weight_pair.mv;
        
        let mut sub_board: ChessBoard = chess_board.clone();
        let mut mvel_pair: MoveScorePair = MoveScorePair::new(0, 0, SCORE_EXACT_TYPE);

        make_move(&mut sub_board, mv);

        let counter : u8 = add_to_game_tree(game_tree, sub_board.zobrist_hash);

        // position repetition check
        if counter >= 3{
            mvel_pair = MoveScorePair::new(0, 0, SCORE_EXACT_TYPE);
        }

        else{
            mvel_pair = -get_best_move_negamax(&mut sub_board, game_tree, transposition_table, depth - 1, ply + 1, search_extention_counter, -beta, -alpha, timer, node_counter);   
        }

        if mvel_pair.score >= beta{
            remove_from_game_tree(game_tree, sub_board.zobrist_hash);

            let ply_usize = ply as usize;

            mvel_pair.is_exact = SCORE_NOT_EXACT_TYPE;
            transposition_table.add(true_hash, discredit_score(mvel_pair.score), depth, LOWER_BOUND, mvel_pair.mv);

            return mvel_pair;
        }

        
        if mvel_pair.score > best_mvel_pair.score{
            let ply_usize = ply as usize;

            best_mvel_pair.score = mvel_pair.score;
            best_mvel_pair.mv = mv;
            best_mvel_pair.is_exact = mvel_pair.is_exact;

            if best_mvel_pair.score > alpha{
                alpha = mvel_pair.score;

                tt_entry_type = EXACT_BOUND;
            }
        }

        remove_from_game_tree(game_tree, sub_board.zobrist_hash);
    }

    if best_mvel_pair.is_exact{
        transposition_table.add(true_hash, discredit_score(best_mvel_pair.score), depth, tt_entry_type, best_mvel_pair.mv);
    }


    return best_mvel_pair;
}

pub fn quiescence_search(chess_board: &mut ChessBoard, mut alpha: i16, mut beta: i16, depth: u8) -> MoveScorePair{  
    let stand_pat = get_board_score(chess_board);

    if stand_pat >= beta{
        return MoveScorePair::new(0, beta, SCORE_EXACT_TYPE);
    }
        
    if alpha < stand_pat{
        alpha = stand_pat;
    }
        
    
    let mut best_mvel_pair : MoveScorePair = MoveScorePair::new(0, -INF, SCORE_EXACT_TYPE);

    // upper bound
    let mut move_vec_unsorted: Vec<u16> = Vec::new();

    get_capture_moves(chess_board, &mut move_vec_unsorted);

    // no legal moves
    if move_vec_unsorted.len() == 0 || depth == 0{
        return MoveScorePair::new(0, get_board_score(chess_board), SCORE_EXACT_TYPE);
    }

    let mut move_vec_sorted: Vec<MoveWeightPair> = Vec::new();
    sort_move_vec(&mut move_vec_sorted, &move_vec_unsorted, chess_board);

    for mv_weight_pair in move_vec_sorted{
        let mv = mv_weight_pair.mv;

        let mut sub_board: ChessBoard = chess_board.clone();
        let mut mvel_pair: MoveScorePair = MoveScorePair::new(0, 0, SCORE_EXACT_TYPE);

        make_move(&mut sub_board, mv);
        
        mvel_pair = -quiescence_search(&mut sub_board, -beta, -alpha, depth - 1);                

        if mvel_pair.score >= beta{
            return mvel_pair;
        }

        if mvel_pair.score > best_mvel_pair.score{
            best_mvel_pair.score = mvel_pair.score;
            best_mvel_pair.mv = mv;

            if best_mvel_pair.score > alpha{
                alpha = mvel_pair.score;
            }
        }
    }

    return best_mvel_pair;
}