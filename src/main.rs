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
use app_pieces::BoardPlugin;

mod board;
mod move_compute;
mod functions;
mod magic_numbers;
mod engine;
mod evaluation;

mod app_settings;
mod app_pieces;

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
    .add_plugins(BoardPlugin)
    .add_systems(Startup, setup)
    .run();

    // let mut chess_board_1 = fen_to_board("rnb2bnr/pppkppp1/4q3/3N3p/4B3/3P1N2/PPP2P1P/R1BQK2R/ w KQ - 0 1");

    // debug(&mut chess_board_1);
}


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

// fn update(mut )