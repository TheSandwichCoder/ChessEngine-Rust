use crate::board::*;
use crate::transposition_table::TranspositionTable;
use std::collections::HashMap;

// a game board is used to hold the transposition table and move tree
// Game Boards should not be used recursively
// references to the transposition table and move tree should be passed instead

pub struct GameChessBoard{
    pub board: ChessBoard,
    pub game_tree: HashMap<u64, u8>,
    pub transposition_table: TranspositionTable,
    pub move_limit: u16,
}

impl Clone for GameChessBoard {
    fn clone(&self) -> GameChessBoard {
        let mut game_board = GameChessBoard{
            board: self.board.clone(),
            game_tree: self.game_tree.clone(),
            transposition_table: self.transposition_table.clone(),
            move_limit: 0,
        };
        
        return game_board;
    }
}

pub fn create_empty_GameChessBoard() -> GameChessBoard{
    return GameChessBoard{
        board: create_empty_board(),
        game_tree: HashMap::new(),
        transposition_table: TranspositionTable::new(),
        move_limit: 0,
    }
}

pub fn fen_to_GameChessBoard(s: &str) -> GameChessBoard{
    let mut game_board = GameChessBoard{
        board: fen_to_board(s),
        game_tree: HashMap::new(),
        transposition_table: TranspositionTable::new(),
        move_limit: 0,
    };

    add_to_game_tree(&mut game_board.game_tree, game_board.board.zobrist_hash);

    return game_board;
}



// this is just normal move making but with game tree stuff
pub fn game_make_move(chess_board: &mut GameChessBoard, mv: u16){
    make_move(&mut chess_board.board, mv);
    
    add_to_game_tree(&mut chess_board.game_tree, chess_board.board.zobrist_hash);

    // should drain the TT
    if chess_board.transposition_table.exceed_size(){
        chess_board.transposition_table.drain();
    }


    chess_board.move_limit += 1;
}

pub fn get_position_counter(game_tree: &HashMap<u64, u8>, hash: u64) -> u8{
    if !game_tree.contains_key(&hash) {
        return 0;
    }

    return *game_tree.get(&hash).unwrap();
}

pub fn add_to_game_tree(game_tree: &mut HashMap<u64, u8>, hash: u64) -> u8{
    // add hash if it is not already in game tree and increment if it is
    return *game_tree.entry(hash).and_modify(|counter| *counter += 1).or_insert(1);
}

pub fn remove_from_game_tree(game_tree: &mut HashMap<u64, u8>, hash: u64){
    let counter = game_tree.get_mut(&hash).unwrap();
    
    // decrement counter
    *counter -= 1;

    // remove instance if no longer present
    if *counter == 0{
        game_tree.remove(&hash);
    }
}

pub fn print_game_board(game_board: &GameChessBoard){
    print_board(&game_board.board);
}

pub fn print_game_board_info(game_board: &GameChessBoard){
    print_board_info(&game_board.board);

    print_game_tree(game_board);
}

pub fn print_game_tree(game_board: &GameChessBoard){
    println!("\n Zobrist Hash Table");
    if game_board.game_tree.len() == 0{
        println!("[EMPTY]");
    }

    for (hash, counter) in &game_board.game_tree {
        println!("{}:{}", hash, counter);
    }
}