use std::time::Instant;
use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};

use board::*;
use move_compute::*;
use functions::*;
use magic_numbers::*;
use engine::*;
use evaluation::*;
use app_settings::*;
use app_pieces::*;

mod board;
mod move_compute;
mod functions;
mod magic_numbers;
mod engine;
mod evaluation;

mod app_settings;
mod app_pieces;

#[derive(Component)]
struct replayText;

fn main() {
    App::new()
    .add_plugins((
        DefaultPlugins
        .set(ImagePlugin::default_nearest())
        .set(WindowPlugin {
            primary_window: Some(Window{
                title: "Rust Chess".into(),
                resolution: (SCREENSIZE.x, SCREENSIZE.y).into(),
                resizable:false,
                enabled_buttons: bevy::window::EnabledButtons {
                    maximize: false,
                    ..Default::default()
                },
                ..default()
            }),
            ..default()
        })
        .build(),
    ))
    .init_state::<GameState>()
    .add_plugins(BoardPlugin)
    .add_systems(Startup, setup)
    .add_systems(OnExit(GameState::Playing), show_game_over_screen_system)
    .add_systems(OnEnter(GameState::Playing), remove_game_over_screen_system)

    .add_systems(Update, play_again_inputs)
    .run();

    // let mut chess_board_1 = fen_to_board("r1b1Rbk1/p7/2nr3q/2pp3B/1p4P1/2BP3P/PPP2P2/4R1K1/ b - - 0 1");

    // debug(&mut chess_board_1);
}
// move
// d6e6
// e8e6


fn setup(
    mut commands: Commands, 
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
){
    let camera = Camera2dBundle::default();
    commands.spawn(camera);

    let shape = Mesh2dHandle(meshes.add(Rectangle::new(SQUARE_SIZE, SQUARE_SIZE)));

    let color1 = Color::rgb(0.913,0.455,0.318);
    let color2 = Color::rgb(1.0,1.0,1.0);

    for x in 0..8{
        for y in 0..8{
            let true_x = x - 4;
            let true_y = y - 4;

            let pos: Vec3 = Vec3::new(true_x as f32, true_y as f32, 0.0) * SQUARE_SIZE + PIECE_EDGE_OFFSET;

            if x % 2 == y % 2{
                commands.spawn(
                MaterialMesh2dBundle{
                    mesh: shape.clone(),
                    material: materials.add(color2),
                    transform: Transform{
                        translation: pos,
                        ..default()
                    },
                    ..default()
                });
            }
            else{
                commands.spawn(
                    MaterialMesh2dBundle{
                        mesh: shape.clone(),
                        material: materials.add(color1),
                        transform: Transform{
                            translation: pos,
                            ..default()
                        },
                        ..default()
                    });
            }
        }
    }
    
}

fn show_game_over_screen_system(
    mut commands: Commands,
    game_state: Res<State<GameState>>
) {
    let mut game_over_string = String::new();

    match game_state.get() {
        GameState::BlackMate => {
            game_over_string = "You just got pawned".to_string();
        },
        GameState::WhiteMate => {
            game_over_string = "Wow you are so cool".to_string();
        },
        GameState::Draw => {
            game_over_string = "Imagine not winning".to_string();
        },
        _ => {
            game_over_string = "uh oh something bad happened".to_string();
        }
    }

    commands.spawn((
        // Create a TextBundle that has a Text with a single section.
        TextBundle::from_section(
            // Accepts a `String` or any type that converts into a `String`, such as `&str`
            game_over_string.to_owned(),
            TextStyle {
                font_size: 40.0,
                color: Color::rgb(0.0,0.0,0.0),
                ..default()
            },
        ) // Set the justification of the Text
        .with_text_justify(JustifyText::Center)
        // Set the style of the TextBundle itself.
        .with_style(Style {
            position_type: PositionType::Absolute,
            bottom: Val::Px(HALF_SCREENSIZE.x),
            left: Val::Px(HALF_SCREENSIZE.x/2.0),
            ..default()
        }),
        replayText
    ));
}

fn remove_game_over_screen_system(
    mut commands: Commands,
    game_state: Res<State<GameState>>,
    replay_text: Query<Entity, With<replayText>>
){
    if replay_text.iter().count() > 0{
        let replay_text = replay_text.single();
        commands.entity(replay_text).despawn();
    }    
}

fn play_again_inputs(
    mouseInput: Res<ButtonInput<MouseButton>>,
    keyboardInput: Res<ButtonInput<KeyCode>>,
    current_state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>
){

    match current_state.get() {
        GameState::Playing =>{},
        _ =>{
            if keyboardInput.just_pressed(KeyCode::Enter) || mouseInput.just_pressed(MouseButton::Left){
                next_state.set(GameState:: Playing);
            }
        },
    }

}