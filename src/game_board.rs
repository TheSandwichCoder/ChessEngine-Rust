use crate::board::*;
use std::collections::HashMap;

// a game board is used to hold the transposition table and move tree
// Game Boards should not be used recursively
// references to the transposition table and move tree should be passed instead

pub struct GameChessBoard{
    pub board: ChessBoard,
    pub game_tree: HashMap<u64, u8>,
}

impl Clone for GameChessBoard {
    fn clone(&self) -> GameChessBoard {
        let mut game_board = GameChessBoard{
            board: self.board.clone(),
            game_tree: self.game_tree.clone(),
        };

        // for (key, val) in self.game_tree.into_iter() {
        //     game_board.game_tree.insert(key, val);
        // }
        
        return game_board;
    }
}

pub fn fen_to_GameChessBoard(s: &str) -> GameChessBoard{
    return GameChessBoard{
        board: fen_to_board(s),
        game_tree: HashMap::new(),
    }
}



// this is just normal move making but with game tree stuff
pub fn game_make_move(chess_board: &mut GameChessBoard, mv: u16){
    make_move(&mut chess_board.board, mv);
    
    // do this later you idiot
    chess_board.game_tree.insert(1, 1);
}

pub fn print_game_board(game_board: &GameChessBoard){
    print_board(&game_board.board);
}

pub fn print_game_board_info(game_board: &GameChessBoard){
    print_board_info(&game_board.board);

    println!("\n Zobrist Hash Table");
    for (hash, counter) in &game_board.game_tree {
        println!("{}:{}", hash, counter);
    }
}