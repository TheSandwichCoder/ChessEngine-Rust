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
    pub score_type: u8,
}

impl MoveScorePair {
    fn new(mv: u16, score: i16, score_type: u8) -> MoveScorePair {
        MoveScorePair { mv, score, score_type }
    }
}

impl Neg for MoveScorePair {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            mv: self.mv,
            score: -self.score,
            score_type: self.score_type,
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

            println!("evaluation: {} score: {} move line: {}", get_move_string(best_move.mv), best_move.score, get_TT_line_string(&get_move_line(game_board)));
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

            if game_board.transposition_table.contains(zob_hash, rep_count){
                let tt_entry = game_board.transposition_table.get(zob_hash, rep_count);

                tt_entry.print_entry();
            }
            else{
                println!("cannot find entry");
            }

            
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
        
        unsafe{node_counter = 0;}

        let t_start = Instant::now();
        
        if flag == 0{
            get_best_move_negamax(&mut game_board.board, &mut game_board.game_tree, &mut game_board.transposition_table, 5, 0, -INF, INF, &Timer::new(Duration::from_secs(10)), 0, false, &mut "".to_string());
        }
        else if flag == 1{
            unsafe{
                node_counter = perft(&mut game_board.board, 5);
            }
        }

        let time_taken = t_start.elapsed().as_millis();

        unsafe{
            println!("test: {} took: {}ms nds:{} nds/s: {}", fen_pos, time_taken, node_counter, node_counter as u128 /time_taken * 1000);
        }
    }
}

pub fn get_move_line(game_chess_board: &GameChessBoard) -> Vec<u16>{
    let mut chess_board_copy = game_chess_board.board.clone();

    let mut move_line_vec : Vec<u16> = Vec::new();

    let chess_board_repetition: u8 = get_position_counter(&game_chess_board.game_tree, chess_board_copy.zobrist_hash);

    let starting_tt: &TTEntry = game_chess_board.transposition_table.get(chess_board_copy.zobrist_hash, chess_board_repetition);

    make_move(&mut chess_board_copy, starting_tt.best_move);

    move_line_vec.push(starting_tt.best_move);

    for i in 0..starting_tt.depth()-1{
        let chess_board_repetition: u8 = get_position_counter(&game_chess_board.game_tree, chess_board_copy.zobrist_hash);

        if game_chess_board.transposition_table.contains(chess_board_copy.zobrist_hash, 0){
            let tt_entry: &TTEntry = game_chess_board.transposition_table.get(chess_board_copy.zobrist_hash, 0);

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

pub fn debug_print(s: &str, depth: u8){
    // Open a file with append option
    let mut data_file = fs::OpenOptions::new()
        .append(true)
        .open("debug.txt")
        .expect("cannot open file");

    unsafe{
        for i in 0..(CURR_SEARCH_DEPTH-depth){
            data_file
            .write(b"\t")
            .expect("write failed");
        }
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

pub fn debug_log(logs: &str,  depth: u8){
    // Open a file with append option
    let mut data_file = fs::OpenOptions::new()
        .append(true)
        .open("debug.txt")
        .expect("cannot open file");

    unsafe{
        for i in 0..(CURR_SEARCH_DEPTH-depth){
            data_file
            .write(b">")
            .expect("write failed");
        }
    }
    
    
    // Write to a file
    data_file
        .write(format!("{}\n", logs).as_bytes())
        .expect("write failed");
}

pub fn discredit_score(score: i16) -> i16{
    // probably a checkmate
    if score < -9990 || score > 9990{
        if score > 0{
            return score - 1;
        }
        else{
            return score + 1;
        }
    }

    return score;
}

pub fn get_search_extention(board: &ChessBoard, mv_weight_pair: MoveWeightPair) -> u8{
    // capture extension
    if mv_weight_pair.weight == 2{
        return 1;
    }

    return 0;

}

static mut node_counter: u32 = 0;
const INF: i16 = 32767;

static mut CURR_SEARCH_DEPTH : u8 = 0;

pub fn get_best_move(game_chess_board: &mut GameChessBoard, time_alloc: u32) -> MoveScorePair{
    // unsafe{CURR_SEARCH_DEPTH = depth;}

    return iterative_deepening(&mut game_chess_board.board, &mut game_chess_board.game_tree, &mut game_chess_board.transposition_table, time_alloc);
}

pub const BASE_NODE_TYPE: u8 = 0;

pub const EVAL_NODE_TYPE: u8 = 1;
pub const CUT_NODE_TYPE : u8 = 2;
pub const TT_NODE_TYPE : u8 = 3;

// testing fens:
// 2Q2nk1/p4p1p/1p2rnp1/3p4/3P3q/BP6/P2N4/2K2R b - - - 

// heavily inspired by pleco engine... again
pub fn iterative_deepening(chess_board: &mut ChessBoard, game_tree: &mut HashMap<u64, u8>, transposition_table: &mut TranspositionTable, time_alloc: u32) -> MoveScorePair{
    let timer: Timer = Timer::new(Duration::from_millis(time_alloc as u64));

    let depth: u8 = 7;
    let mut best_mvel = MoveScorePair::new(0, -INF, BASE_NODE_TYPE);

    let mut alpha : i16 = -INF;
    let mut beta : i16 = INF;

    let mut curr_depth = 1;

    let mut move_vec_unsorted: Vec<u16> = Vec::new();

    get_moves(chess_board, &mut move_vec_unsorted);

    let mut move_vec_sorted: Vec<MoveWeightPair> = Vec::new();

    sort_move_vec(&mut move_vec_sorted, &move_vec_unsorted, chess_board);

    // if transposition_table.exceed_size(){
    //     transposition_table.drain();
    // }    
    // transposition_table.clear();

    // let mut move_line_array:[u16; 32] = [0; 32];

    let mut log_string : String = "".to_string();

    while curr_depth < MAX_SEARCH_DEPTH{

        unsafe{CURR_SEARCH_DEPTH = curr_depth;}

        // debug_print(&"{", curr_depth);
        debug_log_str(&mut log_string, &"{", 0);

        if timer.time_out(){
            break;
        }
        // Search Starts here
        let mut best_mvel_search_pair : MoveScorePair = MoveScorePair::new(0, -INF, BASE_NODE_TYPE);

        // for mv_weight_pair in &move_vec_sorted{
        //     print!("{}:{},", get_move_string(mv_weight_pair.mv), mv_weight_pair.weight);
        // }
        // println!("");

        for mut mv_weight_pair in &mut move_vec_sorted{
            let mv = mv_weight_pair.mv;

            
            let mut sub_board: ChessBoard = chess_board.clone();
            let mvel_pair: MoveScorePair;

            make_move(&mut sub_board, mv);

            add_to_game_tree(game_tree, sub_board.zobrist_hash);

            // mvel_pair = -get_best_move_negamax(&mut sub_board, game_tree, transposition_table, curr_depth - 1, 1, -beta, -alpha, &timer, &mut move_line_array);
            mvel_pair = -get_best_move_negamax(&mut sub_board, game_tree, transposition_table, curr_depth - 1, 1, -INF, INF, &timer, mv_weight_pair.mv, false, &mut log_string);

            remove_from_game_tree(game_tree, sub_board.zobrist_hash);
            // println!("{} {} a:{}",get_move_string(mv), mvel_pair.score, alpha);

            
            // debug_print(&format!("TOP LEVEL {} s:{} d:{}", get_move_string(mv), mvel_pair.score, depth), 0);

            if timer.time_out(){
                break;
            }

            mv_weight_pair.weight = mvel_pair.score;

            println!("score: {}", mvel_pair.score);

            if mvel_pair.score > best_mvel_search_pair.score{
                best_mvel_search_pair.score = mvel_pair.score;
                best_mvel_search_pair.mv = mv;

                // update the best line
    
                if mvel_pair.score > alpha{
                    alpha = mvel_pair.score;
                }
            }
        }
        
        debug_log_str(&mut log_string, &"}", 0);
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
            // move was null
            if best_mvel_search_pair.mv != 0{
                
                best_mvel = best_mvel_search_pair;
                unsafe{
                    println!("DEPTH SEARCHED TO {} a:{} b:{} nodes:{} best move: {} eval: {}",curr_depth, alpha, beta, node_counter, get_move_string(best_mvel.mv), best_mvel.score);
                    node_counter = 0;
                }
                alpha = -INF;
                beta = INF;

                // alpha = best_mvel.score - 35;
                // beta = best_mvel.score + 35;

                curr_depth += 1;
            }
        }
    }

    add_log(&log_string);

    let chess_board_repetition: u8 = get_position_counter(game_tree, chess_board.zobrist_hash);
    transposition_table.add(chess_board.zobrist_hash, chess_board_repetition, discredit_score(best_mvel.score), curr_depth, EXACT_BOUND, best_mvel.mv);

    println!("{}%", transposition_table.capacity() * 100.0);

    return best_mvel;
}

pub const MOVE_LINE_END : u16 = 0xF000 | 1;
pub const BETA_CUTOFF: u16 = 0xF000 | 2;

pub fn get_best_move_negamax(chess_board: &mut ChessBoard, game_tree: &mut HashMap<u64, u8>, transposition_table: &mut TranspositionTable, depth: u8, ply: u8, mut alpha: i16, mut beta: i16, timer: &Timer, recent_move: u16, log_info: bool, log_string: &mut String) -> MoveScorePair{
    let chess_board_repetition: u8 = get_position_counter(game_tree, chess_board.zobrist_hash) - 1;
    
    // if chess_board_repetition == 0{
    //     println!("{}", chess_board_repetition);
    //     print_board(chess_board);
    // }

    if transposition_table.contains(chess_board.zobrist_hash, chess_board_repetition){
        let tt_entry: &mut TTEntry = transposition_table.get_mut(chess_board.zobrist_hash, chess_board_repetition);
        
        // larger / equal search
        // change here
        if tt_entry.depth() >= depth{
            // if tt_entry.depth() == 0{
            //     println!("tt:{} score:{}",tt_entry.score, get_board_score(chess_board));
            // }
            
            
            let entry_type = tt_entry.entry_type();

            if tt_entry.visited < 255{
                tt_entry.visited += 1;
            }

            if entry_type == EXACT_BOUND{
                // make a TT entry

                // move_line_array[ply as usize] = tt_entry.depth() as u16 | 0xF000;

                // print_board(chess_board);
                // println!("depth: {}", tt_entry.depth());
                // println!("{}",get_TT_line_string(&tt_entry.move_line));
                // add_to_move_line(move_line_array, ply as usize, &tt_entry.move_line);

                if log_info{
                    debug_log_str(log_string, &format!("({},{},{},{})", TT_NODE_TYPE, tt_entry.score, get_move_string(recent_move),chess_board.zobrist_hash), ply);
                }

                return MoveScorePair::new(0, tt_entry.score, BASE_NODE_TYPE);
            }

            // else if entry_type == LOWER_BOUND && tt_entry.score <= alpha{
            //     move_line_array[ply as usize] = MOVE_LINE_TT;
            //     return MoveScorePair::new(0, tt_entry.score);
            // }

            // else if entry_type == UPPER_BOUND && tt_entry.score >= beta{
            //     move_line_array[ply as usize] = MOVE_LINE_TT;
            //     return MoveScorePair::new(0, tt_entry.score);
            // }
            
            // // debug_print(&format!("TT LOOKUP fen{} s:{} tt-d:{}", board_to_fen(&sub_board), tt_entry.score, tt_entry.depth()), depth);            
        }
    }
    
    unsafe{
        // if depth == 0 || ply >= CURR_SEARCH_DEPTH + MAX_SEARCH_EXTENSION{

        if depth == 0{
            // unsafe{node_counter += 1;}
            
            // let board_score = get_board_score(chess_board);

            // return MoveScorePair::new(0, board_score);
            // move_line_array[ply as usize] = MOVE_LINE_END;
            let qsearch_score = quiescence_search(chess_board, alpha, beta, QUIESCENCE_DEPTH_LIMIT);

            if log_info{
                debug_log_str(log_string, &format!("({},{},{},{})", EVAL_NODE_TYPE, qsearch_score.score, get_move_string(recent_move), chess_board.zobrist_hash), ply);
            }

            return qsearch_score;
        }
    }
    
    // debug_print(&format!("--NEW SEARCH d:{}--", depth), depth);

    // we subtract one since it was added in the previous iteration
    let mut best_mvel_pair : MoveScorePair = MoveScorePair::new(0, -INF, BASE_NODE_TYPE);
    // let mut new_move_line_array: [u16; 32] = move_line_array.clone();
    // let mut new_move_line_array = [0; 32];

    // upper bound
    let mut tt_entry_type: u8 = UPPER_BOUND;

    let mut move_vec_unsorted: Vec<u16> = Vec::new();

    get_moves(chess_board, &mut move_vec_unsorted);

    // no legal moves
    if move_vec_unsorted.len() == 0{
        // stalemate
        if chess_board.check_mask == 0{
            best_mvel_pair.score = 0;
        }
        
        // checkmate
        else{
            // shift the checkmate so closer checkmates are preffered
            unsafe{
                best_mvel_pair.score = -10000 + ((CURR_SEARCH_DEPTH - depth) as i16);
            }
        }

        // println!("atleast something else");

        return best_mvel_pair
    }

    let mut move_vec_sorted: Vec<MoveWeightPair> = Vec::new();
    sort_move_vec(&mut move_vec_sorted, &move_vec_unsorted, chess_board);

    // println!("sorted mv vec {}", move_vec_sorted.len());

    for mv_weight_pair in move_vec_sorted{
        let mv = mv_weight_pair.mv;

        if timer.time_out(){
            return best_mvel_pair;
        }
        let mut sub_board: ChessBoard = chess_board.clone();
        let mut mvel_pair: MoveScorePair = MoveScorePair::new(0, 0, BASE_NODE_TYPE);

        // let search_extention = get_search_extention(chess_board, mv_weight_pair);

        make_move(&mut sub_board, mv);

        
        // debug_print(&format!("MV START mv{} depth{}", get_move_string(mv), depth), depth);

        let counter : u8 = add_to_game_tree(game_tree, sub_board.zobrist_hash);

        // position repetition check
        if counter >= 3{
            mvel_pair = MoveScorePair::new(0, 0, 0);
        }

        else{
            mvel_pair = -get_best_move_negamax(&mut sub_board, game_tree, transposition_table, depth - 1, ply + 1, -beta, -alpha, timer, mv_weight_pair.mv, log_info, log_string);   
        }

        
        // debug_print(&format!("MV END mv{} s:{} d:{}", get_move_string(mv), mvel_pair.score, depth), depth);
        // println!("mvel pair score: {}", mvel_pair.score);

        if mvel_pair.score >= beta{
            
            // debug_print(&"BETA CUTOFF", depth);
            
            
            remove_from_game_tree(game_tree, sub_board.zobrist_hash);

            let ply_usize = ply as usize;
            
            // new_move_line_array[ply_usize] = BETA_CUTOFF;
            // move_line_array[ply_usize..].copy_from_slice(&new_move_line_array[ply_usize..]);

            if log_info{
                debug_log_str(log_string, &format!("({},{},{},{})", CUT_NODE_TYPE, mvel_pair.score, get_move_string(recent_move), sub_board.zobrist_hash), ply);
            }

            mvel_pair.score_type = CUT_NODE_TYPE;
            // transposition_table.add(chess_board.zobrist_hash, chess_board_repetition, discredit_score(mvel_pair.score), depth, LOWER_BOUND, get_move_line_slice(move_line_array, ply as usize));
            return mvel_pair;
        }

        
        if mvel_pair.score > best_mvel_pair.score{
            // println!("here {}", depth);
            // debug_print(&format!("NEW BEST d:{}", depth), depth);

            let ply_usize = ply as usize;
            
            // update the move line 
            // new_move_line_array[ply_usize] = mv;
            // move_line_array[ply_usize..].copy_from_slice(&new_move_line_array[ply_usize..]);
            // *move_line_array = new_move_line_array;

            best_mvel_pair.score = mvel_pair.score;
            best_mvel_pair.mv = mv;
            best_mvel_pair.score_type = mvel_pair.score_type;
            // tt_entry_type = EXACT_BOUND;

            if best_mvel_pair.score > alpha{
                alpha = mvel_pair.score;

                tt_entry_type = EXACT_BOUND;
            }
        }

        remove_from_game_tree(game_tree, sub_board.zobrist_hash);
    }

    // debug_print(&format!("TT WRITE fen{} s:{} d:{}", board_to_fen(&chess_board), best_mvel_pair.score, depth), depth);
    // debug_print(&format!("RETURN BEST mv{} s:{} d:{}", get_move_string(best_mvel_pair.mv), best_mvel_pair.score, depth), depth);
    
    if best_mvel_pair.score_type != CUT_NODE_TYPE{
        transposition_table.add(chess_board.zobrist_hash, chess_board_repetition, discredit_score(best_mvel_pair.score), depth, tt_entry_type, best_mvel_pair.mv);
    }
    
    if log_info{
        debug_log_str(log_string, &format!("({},{},{},{})", BASE_NODE_TYPE, best_mvel_pair.score, get_move_string(recent_move), chess_board.zobrist_hash), ply);
    }

    return best_mvel_pair;
}

pub fn quiescence_search(chess_board: &mut ChessBoard, mut alpha: i16, mut beta: i16, depth: u8) -> MoveScorePair{  
    let stand_pat = get_board_score(chess_board);

    if stand_pat >= beta{
        return MoveScorePair::new(0, beta, BASE_NODE_TYPE);
    }
        
    if alpha < stand_pat{
        alpha = stand_pat;
    }
        
    
    let mut best_mvel_pair : MoveScorePair = MoveScorePair::new(0, -INF, BASE_NODE_TYPE);

    // upper bound
    let mut move_vec_unsorted: Vec<u16> = Vec::new();

    get_capture_moves(chess_board, &mut move_vec_unsorted);

    // no legal moves
    if move_vec_unsorted.len() == 0 || depth == 0{
        unsafe{node_counter += 1;}
        return MoveScorePair::new(0, get_board_score(chess_board), BASE_NODE_TYPE);
    }

    let mut move_vec_sorted: Vec<MoveWeightPair> = Vec::new();
    sort_move_vec(&mut move_vec_sorted, &move_vec_unsorted, chess_board);

    for mv_weight_pair in move_vec_sorted{
        let mv = mv_weight_pair.mv;

        let mut sub_board: ChessBoard = chess_board.clone();
        let mut mvel_pair: MoveScorePair = MoveScorePair::new(0, 0, BASE_NODE_TYPE);

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