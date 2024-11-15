use bevy::prelude::*;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::*;
use std::thread;
use bevy::window::PrimaryWindow;
use crate::board::*;
use crate::move_compute::get_move_code;
use crate::functions::*;
use crate::app_settings::*;
use crate::engine::*;
use crate::game_board::*;
use crate::evaluation::*;

pub struct BoardPlugin;

// I completely forgot my threading system so I'm going to leave this here for future me
// Has helped counter - 1

// gameloop - player move piece
// thread1 - engine move handling
// thread2 - engine move making

// player move piece -> sends new board copy to thread1
// thread1 sees new board copy -> does move search and finds move
// thread1 sends move to thread2 -> thread2 makes the move in global board

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, States)]
pub enum GameState {
    #[default]
    Playing,
    BlackMate,
    WhiteMate,
    Draw,
}

#[derive(Component, Clone)]
pub struct BoardParent{
    pub game_board: GameChessBoard,
    pub piece_selected_pos: IVec3,
    pub just_moved: bool,
}

#[derive(Component, Default)]
pub struct Piece_Follow{
    pub visibility: bool,
    pub pos: Vec3,
}

#[derive(Component, Default)]
pub struct Piece{
    pub pos: IVec3,
}

#[derive(Resource)]
struct ChessPieceAssets{
    textures: Vec<Handle<Image>>,
}

#[derive(Resource)]
struct ReceiveMoveTag(Mutex<Receiver<u16>>);

#[derive(Resource)]
struct SendBoardTag(Mutex<Sender<GameChessBoard>>);


// store the best move
struct EngineMove(u16);

impl Plugin for BoardPlugin{
    fn build(&self, app:&mut App){
        let (rx, tx) = engine_move_handling();

        app.insert_resource(ReceiveMoveTag(Mutex::new(rx)))
            .insert_resource(SendBoardTag(Mutex::new(tx)))
            .add_systems(Startup, load_chess_piece_images.before(spawn_board_parent).before(spawn_board_pieces).before(spawn_piece_follow))
            .add_systems(Startup,(spawn_board_parent.before(spawn_board_pieces), spawn_board_pieces, spawn_piece_follow))
            .add_systems(Update, (update_pieces_pos, update_piece_follow_pos, player_move_piece, update_board_position))
            .add_systems(Update, engine_move_piece)
            .add_systems(OnEnter(GameState::Playing), reset_board);
    }
}

fn engine_move_handling() -> (Receiver<u16>, Sender<GameChessBoard>) {
    let (tx, rx): (Sender<u16>, Receiver<u16>) = channel();

    let (tx2, rx2): (Sender<GameChessBoard>, Receiver<GameChessBoard>) = channel();

    thread::spawn(move || {
        loop {
            if let Ok(mut game_chess_board) = rx2.try_recv() {
                // let mv : MoveScorePair = get_best_move_depth_search(&game_chess_board.chess_board, DEFAULT_SEARCH_DEPTH);

                let mv: MoveScorePair = get_best_move(&mut game_chess_board, DEFAULT_SEARCH_DEPTH);

                print_move_command_debug(mv.mv);

                tx.send(mv.mv).unwrap();
            }
        }
    });

    (rx, tx2)
}



fn engine_move_piece(
    mut board_parent : Query<&mut BoardParent>,
    mv_rx: Res<ReceiveMoveTag>
){
    let mut board_parent = board_parent.single_mut();

    if let Ok(mv) = mv_rx.0.lock().unwrap().try_recv(){
        // make_move(&mut board_parent.game_board.board, mv);

        game_make_move(&mut board_parent.game_board, mv);

        board_parent.just_moved = true;
    }
}

fn load_chess_piece_images(mut commands: Commands, asset_server: Res<AssetServer>) {
    let chess_piece_images = vec![
        asset_server.load("white_pawn.png"),
        asset_server.load("white_bishop.png"),
        asset_server.load("white_knight.png"),
        asset_server.load("white_rook.png"),
        asset_server.load("white_queen.png"),
        asset_server.load("white_king.png"),
        asset_server.load("black_pawn.png"),
        asset_server.load("black_bishop.png"),
        asset_server.load("black_knight.png"),
        asset_server.load("black_rook.png"),
        asset_server.load("black_queen.png"),
        asset_server.load("black_king.png"),
    ];

    commands.insert_resource(ChessPieceAssets {
        textures: chess_piece_images,
    });
}

fn spawn_board_parent(mut commands:Commands){
    commands.spawn((
        SpatialBundle::default(), 
        BoardParent{
            game_board: fen_to_GameChessBoard(DEFAULT_FEN),
            piece_selected_pos: IVec3::new(-1,-1,0),
            just_moved: false,
        }, 
        Name::new("Board Parent")
    ));
}

fn spawn_piece_follow(
    mut commands: Commands,
    chess_piece_assets: Res<ChessPieceAssets>,
){
    commands.spawn((
        SpriteBundle{
            texture: chess_piece_assets.textures[0].clone(),
            transform: Transform {
                scale: Vec3::new(SCALE_FACTOR, SCALE_FACTOR, SCALE_FACTOR),  // Resize the image by scaling it
                translation: Vec3::new(0.0,0.0,-1.0),
                ..default()
            },
            ..default()
        },
        Piece_Follow{
            visibility: false,
            pos: Vec3::new(0.0,0.0,-1.0),
        },
        Name::new("Piece Follow")
    ));
}

fn board_pos_to_global_pos(vec: IVec3) -> Vec3{
    return (vec - IVec3::new(4,4,-1)).as_vec3() * SQUARE_SIZE + PIECE_EDGE_OFFSET;
}

fn global_pos_to_board_pos(vec: Vec3) -> IVec3{
    return ((vec + HALF_SCREENSIZE) / SQUARE_SIZE).as_ivec3();
}

fn square_num_to_board_pos(square: u8) -> IVec3{
    let x_pos = (square % 8) as i32;
    let y_pos = (square / 8) as i32;

    // the 7 - y is to flip the board so white is on the bottom
    return IVec3::new(x_pos, 7-y_pos, 1);
}

fn board_pos_to_square_num(board_pos: IVec3) -> u8{
    return ((7-board_pos.y) * 8 + board_pos.x) as u8;
}

fn mouse_pos_to_global_pos(mouse_pos: Vec2) -> Vec3{
    return Vec3::new(mouse_pos.x - SCREENSIZE.x/2.0, -mouse_pos.y+SCREENSIZE.y/2.0, 0.0);
}


fn spawn_board_pieces(
    mut commands: Commands,
    mut parent: Query<(Entity, &mut BoardParent)>,
    chess_piece_assets: Res<ChessPieceAssets>,
){
    let (parent, mut parentStruct) = parent.single_mut();

    for i in 0..12{
        let mut temp_bitboard : u64 = parentStruct.game_board.board.piece_bitboards[i];

        while temp_bitboard != 0{
            let square: u8 = temp_bitboard.trailing_zeros() as u8;

            commands.entity(parent).with_children(|commands|{
                commands.spawn((
                    SpriteBundle{
                        texture: chess_piece_assets.textures[i].clone(),
                        transform: Transform {
                            scale: Vec3::new(SCALE_FACTOR, SCALE_FACTOR, SCALE_FACTOR),  // Resize the image by scaling it
                            translation: Vec3::new(0.0,0.0,0.0),
                            ..default()
                        },
                        ..default()
                    },
                    Piece{
                        pos: square_num_to_board_pos(square),
                    },
                    Name::new("Piece"),
                ));
            });

            temp_bitboard ^= 1 << square;
        }
    }
}

// I'm sorry for I have sinned
fn update_board_position(
    mut commands: Commands,
    pieces: Query<Entity, With<Piece>>,
    mut board_parent: Query<(Entity, &mut BoardParent)>,
    chess_piece_assets: Res<ChessPieceAssets>,
    current_state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>
){
    let (board_parent, mut parent_struct) = board_parent.single_mut();


    if parent_struct.just_moved{
        // remove all the current pieces
        for piece in &pieces{
            commands.entity(board_parent).remove_children(&[piece]);
            commands.entity(piece).despawn();
        }


        for i in 0..12{
            let mut temp_bitboard : u64 = parent_struct.game_board.board.piece_bitboards[i];

            while temp_bitboard != 0{
                let square: u8 = temp_bitboard.trailing_zeros() as u8;

                commands.entity(board_parent).with_children(|commands|{
                    commands.spawn((
                        SpriteBundle{
                            texture: chess_piece_assets.textures[i].clone(),
                            transform: Transform {
                                scale: Vec3::new(SCALE_FACTOR, SCALE_FACTOR, SCALE_FACTOR),  // Resize the image by scaling it
                                translation: Vec3::new(0.0,0.0,0.0),
                                ..default()
                            },
                            ..default()
                        },
                        Piece{
                            pos: square_num_to_board_pos(square),
                        },
                        Name::new("Piece"),
                    ));
                });

                temp_bitboard ^= 1 << square;
            }
        }

        let game_board_state = get_gamestate(&parent_struct.game_board);
        if game_board_state == 1{
            next_state.set(GameState::WhiteMate);
        }
        else if game_board_state == 2{
            next_state.set(GameState::BlackMate);
        }
        else if game_board_state == 3{
            next_state.set(GameState::Draw);
        }
        

        parent_struct.just_moved = false;
    }

}


fn update_pieces_pos(
    mut pieces: Query<(&mut Transform, &Piece)>,
    board: Query<&BoardParent>
){
    let board = board.single();
    for (mut transform, piece) in &mut pieces{
        

        transform.translation = board_pos_to_global_pos(piece.pos);

        if piece.pos.xy() == board.piece_selected_pos.xy(){
            transform.translation.z = -1.0;
        }
        else{
            transform.translation.z = 1.0;
        }
    } 
}

fn reset_board(
    mut board: Query<&mut BoardParent>
){
    // I hate threading
    if board.iter().count() > 0{
        let mut board = board.single_mut();

        board.game_board = fen_to_GameChessBoard(DEFAULT_FEN);
        board.piece_selected_pos = IVec3::new(-1,-1,0);
        board.just_moved = true;
    }
}

fn update_piece_follow_pos(
    mut piece_follow: Query<(&mut Transform, &mut Piece_Follow)>,
    q_windows: Query<&Window, With<PrimaryWindow>>,    
){
    let mouse_position = q_windows.single().cursor_position();

    if !mouse_position.is_some(){
        return;
    }

    let (mut transform,mut piece_follow) = piece_follow.single_mut();

    let mouse_pos_3d = mouse_pos_to_global_pos(mouse_position.unwrap());

    piece_follow.pos = mouse_pos_3d;
    if piece_follow.visibility{
        piece_follow.pos.z = 2.0;
    }
    else{
        piece_follow.pos.z = -1.0;
    }
    
    transform.translation = piece_follow.pos;
}

fn player_move_piece(
    mut commands: Commands,
    mut board_parent: Query<&mut BoardParent>,
    mut piece_follow: Query<(&mut Handle<Image>, &mut Piece_Follow)>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    mouseInput: Res<ButtonInput<MouseButton>>,
    chess_piece_assets: Res<ChessPieceAssets>,
    board_tx: Res<SendBoardTag>,
){
    let (mut piece_follow_texture, mut piece_follow) = piece_follow.single_mut();
    let mut board_parent = board_parent.single_mut();

    let game_board : &GameChessBoard = &board_parent.game_board;

    let mouse_position = q_windows.single().cursor_position();

    if !mouse_position.is_some(){
        return;
    }

    if mouseInput.just_pressed(MouseButton::Left) {
        let board_selected_pos = global_pos_to_board_pos(mouse_pos_to_global_pos(mouse_position.unwrap()));

        // selecting a piece
        if board_parent.piece_selected_pos.x == -1{
            let piece_square: usize = board_pos_to_square_num(board_selected_pos) as usize;
            let selected_piece_type = game_board.board.piece_array[piece_square] as usize;

            // selected an empty square or a black piece or its blacks turn
            if selected_piece_type == 0 || selected_piece_type > 6 || !game_board.board.board_color{
                return;
            }

            board_parent.piece_selected_pos = board_selected_pos;
            piece_follow.visibility = true;

            *piece_follow_texture = chess_piece_assets.textures[selected_piece_type-1].clone();
            
        }

        // making a move
        else{
            let mut move_vec: Vec<u16> = Vec::new();

            get_moves(&game_board.board, &mut move_vec);

            let mut is_legal : bool = false;
            
            let mut move_code : u16 = get_move_code(board_pos_to_square_num(board_parent.piece_selected_pos), board_pos_to_square_num(board_selected_pos));

            for mv in move_vec{
                if mv & 0xFFF == move_code{
                    // promotion - forced to be queen promotion cause Im lazy
                    if mv >> 12 >= 5 && mv >> 12 <= 8{
                        move_code |= 0x8000;
                    }
                    else{
                        move_code = mv;
                    }
                    
                    is_legal = true;
                    break;
                }
            }             

            board_parent.piece_selected_pos = IVec3::new(-1, -1, 0);
            piece_follow.visibility = false;

            if !is_legal{
                return;
            }

            
            // make_move(&mut board_parent.board, move_code);

            game_make_move(&mut board_parent.game_board, move_code);

            board_parent.just_moved = true;

            print_move_command_debug(move_code);
            println!("{}", board_to_fen(&board_parent.game_board.board));

            let game_board_state = get_gamestate(&board_parent.game_board);

            // the game ended
            if game_board_state != 0{
                return;
            }

            // ask the move thread to make a move
            board_tx.0.lock().unwrap().send(board_parent.game_board.clone()).unwrap();
        }
    }
}

