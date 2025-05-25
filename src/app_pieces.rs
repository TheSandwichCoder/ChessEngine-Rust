use bevy::prelude::*;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::*;
use std::{thread, time};
use std::io::{self, Write};
use bevy::window::PrimaryWindow;
use crate::board::*;
use crate::move_compute::get_move_code;
use crate::functions::*;
use crate::app_settings::*;
use crate::engine::*;
use crate::game_board::*;
use crate::evaluation::*;
use crate::move_compute::*;

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
    pub requested_move: bool,
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

#[derive(Resource, Clone)]
pub struct GameSettings{
    pub engine_color: bool,
    pub starting_pos : String,
}

impl GameSettings{
    fn new() -> Self{
        GameSettings{
            engine_color: false,
            starting_pos: DEFAULT_FEN.to_string(),
        }
    }
}

#[derive(Resource)]
struct ChessPieceAssets{
    textures: Vec<Handle<Image>>,
}

#[derive(Resource)]
struct ReceiveMoveTag(Mutex<Receiver<u16>>);

#[derive(Resource)]
struct SendBoardTag(Mutex<Sender<GameChessBoard>>);

#[derive(Resource)]
struct ReceiveMoveTag2(Mutex<Receiver<String>>);


// store the best move
struct EngineMove(u16);

impl Plugin for BoardPlugin{
    fn build(&self, app:&mut App){
        let (rx, tx) = engine_move_handling();
        let rx2 = engine_update_settings();

        app.insert_resource(ReceiveMoveTag(Mutex::new(rx)))
            .insert_resource(SendBoardTag(Mutex::new(tx)))
            .insert_resource(ReceiveMoveTag2(Mutex::new(rx2)))
            .insert_resource(create_game_settings())

            .add_systems(Startup, load_chess_piece_images.before(spawn_board_parent).before(spawn_board_pieces).before(spawn_piece_follow))
            .add_systems(Startup,(spawn_board_parent.before(spawn_board_pieces), spawn_board_pieces, spawn_piece_follow))
            .add_systems(Update, (update_pieces_pos, update_piece_follow_pos, player_move_piece, update_board_position, update_board_move))
            .add_systems(Update, (engine_move_piece, update_game_settings))
            .add_systems(OnEnter(GameState::Playing), reset_board);
    }
}

fn engine_move_handling() -> (Receiver<u16>, Sender<GameChessBoard>) {
    let (tx, rx): (Sender<u16>, Receiver<u16>) = channel();

    let (tx2, rx2): (Sender<GameChessBoard>, Receiver<GameChessBoard>) = channel();

    thread::spawn(move || {
        loop {
            thread::sleep(time::Duration::from_millis(50));
            if let Ok(mut game_chess_board) = rx2.try_recv() {

                // if we changed when the bot is making a mv, we want to

                let mv: MoveScorePair = get_best_move(&mut game_chess_board, DEFAULT_THINK_TIME as u32);

                print_move_command_debug(mv.mv);

                tx.send(mv.mv).unwrap();
            }
        }
    });

    (rx, tx2)
}

// cmd indices
// 1 - reset
// 2 - flip
// 3 - fen
// 4 - time
// 9 - quit
fn engine_update_settings() -> Receiver<String>{
    let (tx, rx): (Sender<String>, Receiver<String>) = channel();

    thread::spawn(move || {
        loop {
            thread::sleep(time::Duration::from_millis(50));
            println!("
[DO NOT GIVE COMMAND WHEN BOT IS THINKING]

COMMANDS
reset - resets board
flip - flips board
fen - takes fen
time - changes time given to the bot

quit - quit
            ");

            let mut send_string = "0NANANA".to_string();

            let mut input_string = String::new();
            io::stdin().read_line(&mut input_string).expect("Failed to read line");
            input_string = input_string.trim().to_string();

            if input_string == "reset"{
                send_string = "1whyuhere".to_string();
            }
            else if input_string == "flip"{
                send_string = "2whyuhere".to_string();
            }
            else if input_string == "fen"{
                input_string.clear();
                print!("fen >>");
                io::stdout().flush().unwrap();
                
                io::stdin().read_line(&mut input_string).expect("Failed to read line");

                input_string = input_string.trim().to_string();
                
                send_string = format!("3{}",input_string);
            }
            else if input_string == "time"{
                input_string.clear();
                print!("new think time (ms) >>");
                io::stdout().flush().unwrap();
            
                io::stdin().read_line(&mut input_string).expect("Failed to read line");

                send_string = format!("4{}", input_string.trim());
            }
            else if input_string == "quit"{
                send_string = "9whyuhere".to_string();
            }
            
            tx.send(send_string).unwrap();
        }
    });

    rx
}



fn engine_move_piece(
    mut board_parent : Query<&mut BoardParent>,
    mut game_settings : ResMut<GameSettings>,
    mv_rx: Res<ReceiveMoveTag>
){
    let mut board_parent = board_parent.single_mut();

    if let Ok(mv) = mv_rx.0.lock().unwrap().try_recv(){
        game_make_move(&mut board_parent.game_board, mv);
        board_parent.just_moved = true;
    }
}

fn update_game_settings(
    mut board: Query<&mut BoardParent>,
    mut game_settings : ResMut<GameSettings>,
    mut writer: EventWriter<AppExit>,
    cmd_rx: Res<ReceiveMoveTag2>,
){
    if let Ok(cmd_str) = cmd_rx.0.lock().unwrap().try_recv(){
        let mut board = board.single_mut();

        let len = cmd_str.len();

        let cmd_type = cmd_str.chars().nth(0).unwrap() as u32 - '0'  as u32;
        let cmd_info = &cmd_str[1..len];

        // reset
        if cmd_type == 1{
            board.game_board = fen_to_GameChessBoard(DEFAULT_FEN);
            board.piece_selected_pos = IVec3::new(-1,-1,0);
            board.just_moved = true;
            board.requested_move = false;
            board.game_board.board.board_color = true;
            game_settings.engine_color = false;
            
        }
        else if cmd_type == 2{
            game_settings.engine_color = !game_settings.engine_color;
            board.just_moved = true;
            board.requested_move = game_settings.engine_color == board.game_board.board.board_color;
            board.piece_selected_pos = IVec3::new(-1,-1,0);


        }

        if cmd_type == 9{
            writer.send(AppExit::Success);
        }
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

fn create_game_settings() -> GameSettings{
    GameSettings::new()
}

fn spawn_board_parent(mut commands:Commands){
    commands.spawn((
        SpatialBundle::default(), 
        BoardParent{
            game_board: fen_to_GameChessBoard(DEFAULT_FEN),
            piece_selected_pos: IVec3::new(-1,-1,0),
            just_moved: false,
            requested_move: false,
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

fn square_num_to_board_pos(square: u8, board_color: bool) -> IVec3{
    let x_pos = (square % 8) as i32;
    let y_pos = (square / 8) as i32;

    // the 7 - y is to flip the board so white is on the bottom
    if board_color{
        return IVec3::new(x_pos, 7-y_pos, 1);
    }
    else{
        return IVec3::new(x_pos, y_pos, 1);;
    }
    
}

fn board_pos_to_square_num(board_pos: IVec3, board_color: bool) -> u8{
    if board_color{
        return ((7-board_pos.y) * 8 + board_pos.x) as u8;
    }
    else{
        return (board_pos.y * 8 + board_pos.x) as u8; 
    }
}

fn mouse_pos_to_global_pos(mouse_pos: Vec2) -> Vec3{
    return Vec3::new(mouse_pos.x - SCREENSIZE.x/2.0, -mouse_pos.y+SCREENSIZE.y/2.0, 0.0);
}


fn spawn_board_pieces(
    mut commands: Commands,
    mut parent: Query<(Entity, &mut BoardParent)>,
    chess_piece_assets: Res<ChessPieceAssets>,
    game_settings: Res<GameSettings>,
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
                        pos: square_num_to_board_pos(square, !game_settings.engine_color),
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
    mut board_parent: Query<(Entity, &mut BoardParent)>,
    mut next_state: ResMut<NextState<GameState>>,
    pieces: Query<Entity, With<Piece>>,
    chess_piece_assets: Res<ChessPieceAssets>,
    current_state: Res<State<GameState>>,
    game_settings: Res<GameSettings>,
    
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
                            pos: square_num_to_board_pos(square, !game_settings.engine_color),
                        },
                        Name::new("Piece"),
                    ));
                });

                temp_bitboard ^= 1 << square;
            }
        }

        let game_board_state = get_gamestate(&mut parent_struct.game_board);
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

fn is_piece_same_color(piece_type: usize, color: bool) -> bool{
    if color{
        return piece_type <= 6;
    }
    else{
        return piece_type > 6;
    }
}

fn player_move_piece(
    mut commands: Commands,
    mut board_parent: Query<&mut BoardParent>,
    mut piece_follow: Query<(&mut Handle<Image>, &mut Piece_Follow)>,
    game_settings: Res<GameSettings>,
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
            let piece_square: usize = board_pos_to_square_num(board_selected_pos, !game_settings.engine_color) as usize;
            let selected_piece_type = game_board.board.piece_array[piece_square] as usize;

            // selected an empty square or a black piece or its blacks turn
            if selected_piece_type == 0 || !is_piece_same_color(selected_piece_type, !game_settings.engine_color) || !(game_board.board.board_color == !game_settings.engine_color){
                return;
            }

            board_parent.piece_selected_pos = board_selected_pos;
            piece_follow.visibility = true;

            *piece_follow_texture = chess_piece_assets.textures[selected_piece_type-1].clone();
            
        }

        // making a move
        else{
            let mut move_buffer = MoveBuffer::new();

            get_moves(&mut board_parent.game_board.board, &mut move_buffer);

            let mut is_legal : bool = false;
            
            let mut move_code : u16 = get_move_code(board_pos_to_square_num(board_parent.piece_selected_pos, !game_settings.engine_color), board_pos_to_square_num(board_selected_pos, !game_settings.engine_color));

            for mv_i in 0..move_buffer.index{
                let mv = move_buffer.mv_arr[mv_i];

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

            let game_board_state = get_gamestate(&mut board_parent.game_board);

            // the game ended
            if game_board_state != 0{
                return;
            }

            // ask the move thread to make a move
            board_parent.requested_move = true;
        }
    }
}

fn update_board_move(
    mut board_parent: Query<&mut BoardParent>,
    board_tx: Res<SendBoardTag>,
){
    let mut board_parent = board_parent.single_mut();
    
    if board_parent.requested_move{
        board_parent.requested_move = false;
        board_tx.0.lock().unwrap().send(board_parent.game_board.clone()).unwrap();
    } 
}