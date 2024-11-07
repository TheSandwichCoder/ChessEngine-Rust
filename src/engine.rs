use std::io::{self, Write};
use std::collections::HashMap;

use crate::move_compute::*;
use crate::functions::*;
use crate::board::*;
use crate::evaluation::*;
use crate::app_settings::SEARCH_DEPTH;
use crate::game_board::*;

#[derive(Copy, Clone)]
pub struct MoveScorePair{
    pub mv: u16,
    pub score: i16,
}

pub fn create_empty_MoveScorePair() -> MoveScorePair{
    return MoveScorePair{
        mv: 0,
        score: 0,
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

        else if input_string == "fen"{
            input_string.clear();
            print!("fen >>");
            io::stdout().flush().unwrap();
            
            io::stdin()
            .read_line(&mut input_string)
            .expect("Failed to read line");

            input_string = input_string.trim().to_string();
            
            game_board.board = fen_to_board(&input_string);
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

pub fn get_best_move(game_chess_board: &mut GameChessBoard, depth: u8) -> MoveScorePair{
    return get_best_move_depth_search(&game_chess_board.board, &mut game_chess_board.game_tree, depth);
}

pub fn get_best_move_depth_search(chess_board: &ChessBoard, game_tree: &mut HashMap<u64, u8>, depth: u8) -> MoveScorePair{
    let mut move_vec: Vec<u16> = Vec::new();

    get_moves(chess_board, &mut move_vec);

    let mut best_mvel_pair: MoveScorePair = MoveScorePair{
        mv: 0,
        score: 0,
    };

    if chess_board.board_color{
        best_mvel_pair.score = -10000;
    }
    else{
        best_mvel_pair.score = 10000;
    }

    // no legal moves
    if move_vec.len() == 0{
        // stalemate
        if chess_board.check_mask == 0{
            best_mvel_pair.score = 0;
        }
        
        // checkmate
        return best_mvel_pair;
    }

    for mv in move_vec{
        let mut sub_board: ChessBoard = chess_board.clone();

        make_move(&mut sub_board, mv);

        let mvel_pair: MoveScorePair;

        if depth == 1{
            mvel_pair = MoveScorePair{
                mv: mv,
                score: get_board_score(&sub_board),
            };
            unsafe{
                node_counter += 1;
            };
        }

        else{
            mvel_pair = get_best_move_depth_search(&sub_board, game_tree, depth - 1);

            if depth == SEARCH_DEPTH{
                unsafe{
                    println!("move searched: {} {} {}", get_move_string(mv), mvel_pair.score, node_counter);
                    node_counter = 0;
                };
            }
        }

        if (mvel_pair.score > best_mvel_pair.score) == chess_board.board_color{
            best_mvel_pair.score = mvel_pair.score;
            best_mvel_pair.mv = mv;
        }
    }

    return best_mvel_pair;
}
