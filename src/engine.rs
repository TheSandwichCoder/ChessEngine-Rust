use std::io::{self, Write};
use std::collections::HashMap;
use std::time::Duration;
use std::ops::Neg;
use std::fs;


use crate::app_settings::*;
use crate::move_compute::*;
use crate::functions::*;
use crate::board::*;
use crate::evaluation::*;
use crate::game_board::*;
use crate::zobrist_hash::*;
use crate::timer::Timer;


#[derive(Copy, Clone)]
pub struct MoveScorePair{
    pub mv: u16,
    pub score: i16,
}

impl MoveScorePair {
    fn new(mv: u16, score: i16) -> MoveScorePair {
        MoveScorePair { mv, score }
    }
}

impl Neg for MoveScorePair {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            mv: self.mv,
            score: -self.score,
        }
    }
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
            let mvel_pair : MoveScorePair = get_best_move(&mut game_board, BATTLE_THINK_TIME);

            println!("Move: {} {}", get_move_string(mvel_pair.mv), mvel_pair.score);            

            game_make_move(&mut game_board, mvel_pair.mv);

            // print_game_board(&game_board);
            
            // print_game_tree(&game_board);

            {
                // update the fen position lookup
                fs::write("FenPositionLookup.txt", format!("{}|{}", get_gamestate(&game_board), board_to_fen(&game_board.board)));

                // provide move
                fs::write(file_path.clone(), format!("5|{}",mvel_pair.mv));
            }

            move_now = false;
        }

        
    }
}

// gets the number of nodes given an iteration depth
pub fn perft(board: &ChessBoard, depth: u16) -> u32{
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

        node_num += perft(&sub_board, depth - 1);
    } 

    return node_num;
}

pub fn sub_perft(board: &ChessBoard, depth: u16){
    let mut move_vec: Vec<u16> = Vec::new();

    get_moves(board, &mut move_vec);

    let mut total_counter: u32 = 0;

    for mv in move_vec{
        let mut sub_board: ChessBoard = board.clone();

        make_move(&mut sub_board, mv);

        let move_num: u32 = perft(&sub_board, depth - 1);

        total_counter += move_num;
        println!("{}|{}",get_move_string(mv), move_num);
    } 

    println!("Total: {}", total_counter);
}

pub fn debug_zobrist_hash(board: &ChessBoard, depth:u16){
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

        debug_zobrist_hash(&sub_board, depth - 1);
    } 
}

pub fn debug(game_board: &mut GameChessBoard){
    let mut input_string: String = String::new();

    let mut debug_running: bool = true;

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
                game_board.board = fen_to_board(&DEFAULT_FEN);
            }
            else{
                game_board.board = fen_to_board(&input_string);
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

            // make_move(chess_board, mv);
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

            // make_move(chess_board, mv);
            game_make_move(game_board, mv);
        }

        else if input_string == "show moves"{
            let mut move_vec: Vec<u16> = Vec::new();

            get_moves(&game_board.board, &mut move_vec);

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

            sub_perft(&game_board.board, depth);
        }

        else if input_string == "debug z"{
            input_string.clear();
            print!("depth >>");
            io::stdout().flush().unwrap();
            
            io::stdin()
            .read_line(&mut input_string)
            .expect("Failed to read line");

            let depth: u16 = input_string.trim().parse().expect("cannot parse string to int");

            debug_zobrist_hash(&game_board.board, depth);
        }

        else if input_string == "best move"{
            input_string.clear();
            print!("depth >>");
            io::stdout().flush().unwrap();
            
            io::stdin()
            .read_line(&mut input_string)
            .expect("Failed to read line");

            let depth: u8 = input_string.trim().parse().expect("cannot parse string to int");

            // let best_move: MoveScorePair = get_best_move_depth_search(chess_board, depth);

            let best_move: MoveScorePair = get_best_move(game_board, depth);

            println!("evaluation: {} score: {}", get_move_string(best_move.mv), best_move.score);
        }
    }
}

static mut node_counter: u32 = 0;
const INF: i16 = 32767;

static mut CURR_SEARCH_DEPTH : u8 = DEFAULT_SEARCH_DEPTH;

pub fn get_best_move(game_chess_board: &mut GameChessBoard, time_alloc: u8) -> MoveScorePair{
    // unsafe{CURR_SEARCH_DEPTH = depth;}

    return iterative_deepening(&game_chess_board.board, &mut game_chess_board.game_tree, time_alloc);
    // return get_best_move_negamax(&game_chess_board.board, &mut game_chess_board.game_tree, depth, -INF, INF);
}

// heavily inspired by pleco engine... again
pub fn iterative_deepening(chess_board: &ChessBoard, game_tree: &mut HashMap<u64, u8>, time_alloc: u8) -> MoveScorePair{
    let timer: Timer = Timer::new(Duration::new(time_alloc as u64, 0));

    let depth: u8 = 7;
    let mut best_mvel = MoveScorePair::new(0, -INF);

    let mut alpha : i16 = -INF;
    let mut beta : i16 = INF;

    unsafe{CURR_SEARCH_DEPTH = depth;}

    let mut curr_depth = 2;

    while curr_depth < MAX_SEARCH_DEPTH{
        if timer.time_out(){
            println!("TIME OUT");
            break;
        }

        let mvel : MoveScorePair = get_best_move_negamax(chess_board, game_tree, curr_depth, alpha, beta, &timer);

        if mvel.score >= beta{
            println!("RESTART SEARCH - BETA");
            beta = INF;
        }
        else if mvel.score <= alpha{
            println!("RESTART SEARCH - ALPHA");
            alpha = -INF;
        }
        else{
            // move was null
            if mvel.mv != 0{
                alpha = mvel.score - 100;
                beta = mvel.score + 100;
                unsafe{
                    println!("DEPTH SEARCHED TO {} a:{} b:{} nodes:{} best move: {}",curr_depth, alpha, beta, node_counter, get_move_string(best_mvel.mv));
                    node_counter = 0;
                }
                
                best_mvel = mvel;
            }
            
            curr_depth += 1;
        }
    
    }

    return best_mvel;
}

pub fn get_best_move_negamax(chess_board: &ChessBoard, game_tree: &mut HashMap<u64, u8>, depth: u8, mut alpha: i16, mut beta: i16, timer: &Timer) -> MoveScorePair{
    if depth == 0{
        unsafe{node_counter += 1;}

        return MoveScorePair::new(0, get_board_score(chess_board));
    }

    let mut best_mvel_pair : MoveScorePair = MoveScorePair::new(0, alpha);

    let mut move_vec: Vec<u16> = Vec::new();

    get_moves(chess_board, &mut move_vec);

    // no legal moves
    if move_vec.len() == 0{
        // stalemate
        if chess_board.check_mask == 0{
            best_mvel_pair.score = 0;
        }
        
        // checkmate
        else{
            // shift the checkmate so closer checkmates are preffered
            best_mvel_pair.score = -10000 - (depth as i16);
        }

        return best_mvel_pair
    }

    for mv in move_vec{
        if timer.time_out(){
            return MoveScorePair::new(0, -INF);
        }
        let mut sub_board: ChessBoard = chess_board.clone();
        let mvel_pair: MoveScorePair;

        make_move(&mut sub_board, mv);

        let counter : u8 = add_to_game_tree(game_tree, sub_board.zobrist_hash);

        // position repetition check
        if counter >= 3{
            mvel_pair = MoveScorePair::new(0, 0);
        }

        else{
            mvel_pair = -get_best_move_negamax(&sub_board, game_tree, depth - 1, -beta, -alpha, timer);

            // unsafe{
            //     if depth == CURR_SEARCH_DEPTH{
            //         println!("move searched: {} {} {}", get_move_string(mv), mvel_pair.score, node_counter);
            //         node_counter = 0;
            //     }
            // }
        }

        if mvel_pair.score >= beta{
            remove_from_game_tree(game_tree, sub_board.zobrist_hash);
            return mvel_pair;
        }

        if mvel_pair.score > best_mvel_pair.score{
            best_mvel_pair.score = mvel_pair.score;
            best_mvel_pair.mv = mv;

            if mvel_pair.score > alpha{
                alpha = mvel_pair.score;
            }
        }

        remove_from_game_tree(game_tree, sub_board.zobrist_hash);
    }

    return best_mvel_pair;
}

