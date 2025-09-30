use bevy::render::mesh::{Mesh, RectangleMeshBuilder};
use std::io;

use bevy::color::palettes::css::GRAY;
use bevy::prelude::*;
use bevy_cursor::prelude::*;
use std::collections::HashMap;
const SNAP_TOLERANCE: f32 = 10.;
mod physics;
use truss::structs::Node;
use truss::structs::*;
fn main() {
    App::new()
        .insert_resource(ClearColor(Color::Srgba(GRAY)))
        .insert_resource(Mode::Insert)
        .insert_resource(Truss {
            nodes: vec![],
            edges: vec![],
            selected_node: None,
            dragging: None,
            connections: vec![],
            membermap: HashMap::new(),
            nodemap: HashMap::new(),
            connectionmap: HashMap::new(),
        })
        .insert_resource(LastNode { position: None })
        .add_plugins((DefaultPlugins, TrackCursorPlugin))
        .add_systems(Startup, setup_camera)
        .add_systems(Update, keyboard_input)
        .add_systems(Update, preview_on)
        .run();
}
fn preview_on(
    mode: Res<Mode>,
    commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut ids: ResMut<Truss>,
    materials: ResMut<Assets<ColorMaterial>>,
    cursor: ResMut<CursorLocation>,
    last: Res<LastNode>,
) {
    let insert = matches!(*mode, Mode::Insert);
    let previewspawned = ids.membermap.contains_key(&0);
    let last_exist = last.position.is_some();
    if !previewspawned && last_exist {
        spawn_line_preview(commands, &mut meshes, &mut ids, materials);
    }
    if insert && previewspawned && last_exist {
        update_line_preview(cursor, ids, last, meshes);
    }
}
fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn keyboard_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut mode: ResMut<Mode>,
    mut last: ResMut<LastNode>,
    cursor: Res<CursorLocation>,
    mut truss: ResMut<Truss>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    match *mode {
        Mode::Command => {
            if keys.just_pressed(KeyCode::KeyI) {
                *mode = Mode::Insert;
                last.position = None;
            }
            if keys.just_pressed(KeyCode::KeyD) {
                *mode = Mode::Dimension;
            }
            if keys.just_pressed(KeyCode::KeyR) {
                physics::calculate_member_stress(truss.into_inner());
            }
        }
        Mode::Insert => {
            let cursorloc = cursor.world_position().unwrap_or(Vec2::ZERO);
            if keys.just_pressed(KeyCode::Space) {
                if let Some(old_node) = truss
                    .clone()
                    .nodes
                    .iter()
                    .find(|x| x.0.distance(cursorloc) < SNAP_TOLERANCE)
                {
                    let memcount = truss.edges.len();
                    let member = Member {
                        start: last.position.unwrap_or(old_node.0),
                        end: old_node.0,
                        id: memcount + 1,
                    };

                    commands.queue(member.clone());
                    truss.edges.push(member);

                    last.position = Some(old_node.0);
                } else {
                    let nodecount = truss.nodes.len();

                    let memcount = truss.edges.len();
                    let node = Node(cursorloc, nodecount + 1);
                    commands.queue(node.clone());
                    truss.nodes.push(node.clone());
                    if last.position.is_some() {
                        let member = Member {
                            start: last.position.unwrap(),
                            end: cursorloc,
                            id: memcount + 1,
                        };
                        commands.queue(member.clone());
                        truss.edges.push(member);
                    }
                    last.position = Some(node.0);
                }
            }
            if keys.just_pressed(KeyCode::Escape) {
                *mode = Mode::Command;
                let memcount = truss.edges.len();
                meshes.remove(truss.membermap.get(&0).unwrap().id());
                truss.membermap.remove(&0);
                if last.position.is_none() {
                    truss.membermap.remove(&memcount);
                }
            }
            if keys.just_pressed(KeyCode::KeyR) {
                let connection_count = truss.connections.len();
                if let Some(old_node) = truss
                    .nodes
                    .iter()
                    .find(|x| x.0.distance(cursorloc) < SNAP_TOLERANCE)
                {
                    commands.queue(Connection::Roller(old_node.0, connection_count));
                } else {
                    commands.queue(Connection::Roller(cursorloc, connection_count));
                }
                truss
                    .connections
                    .push(Connection::Roller(cursorloc, connection_count));
            }
            if keys.just_pressed(KeyCode::KeyP) {
                let connection_count = truss.connections.len();
                if let Some(old_node) = truss
                    .nodes
                    .iter()
                    .find(|x| x.0.distance(cursorloc) < SNAP_TOLERANCE)
                {
                    commands.queue(Connection::Pin(old_node.0, connection_count));
                } else {
                    commands.queue(Connection::Pin(cursorloc, connection_count));
                }
                truss
                    .connections
                    .push(Connection::Pin(cursorloc, connection_count));
            }
            if keys.just_pressed(KeyCode::KeyF) {
                let connection_count = truss.connections.len();
                println!("enter magnitude");
                let mut mag = String::new();
                io::stdin()
                    .read_line(&mut mag)
                    .expect("failed to read message");

                let force = Force {
                    magnitude: mag.trim().parse().unwrap_or(0.),
                    start: last.position.unwrap(),
                    end: cursorloc,
                    id: connection_count,
                };

                if let Some(old_node) = truss
                    .nodes
                    .iter()
                    .find(|x| x.0.distance(cursorloc) < SNAP_TOLERANCE)
                {
                    commands.queue(Connection::Force(force.clone()));
                } else {
                    commands.queue(Connection::Force(force.clone()));
                }
                truss.connections.push(Connection::Force(force));
            }
            // we can check multiple at once with `.any_*`
            if keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]) {
                // Either the left or right shift are being held down
            }
            if keys.any_just_pressed([KeyCode::Delete, KeyCode::Backspace]) {
                // Either delete or backspace was just pressed
            }
        }
        Mode::Edit => {}
        Mode::Dimension => {}
    }
}

fn spawn_line_preview(
    mut commands: Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    ids: &mut ResMut<Truss>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mesh_handle = meshes.add(RectangleMeshBuilder::new(0.0, 0.0).build());
    let color_handle = materials.add(Color::WHITE);
    ids.membermap.insert(0, mesh_handle.clone());

    commands.spawn((
        Mesh2d(mesh_handle),
        MeshMaterial2d(color_handle),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
}

fn update_line_preview(
    cursor: ResMut<CursorLocation>, // to read cursor position
    ids: ResMut<Truss>,
    last: Res<LastNode>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let last_point = last.position.unwrap_or(Vec2::ZERO);
    let mut cursor_loc = cursor.world_position().unwrap_or(Vec2::ZERO);

    let length = last_point.distance(cursor_loc);
    if cursor_loc.x == 0. {
        cursor_loc.x = 1.0;
    }
    if cursor_loc.y == 0. {
        cursor_loc.y = 1.0;
    }
    let diff = last_point - cursor_loc;
    let mut theta = diff.x / diff.y;
    theta = theta.atan();
    let midpoint = (last_point + cursor_loc) / 2.;
    let transform = Transform::from_xyz(midpoint.x, midpoint.y, 0.)
        .with_rotation(Quat::from_rotation_z(-theta));

    let mesh_handle = ids.membermap.get(&0).unwrap();
    let mesh = meshes.get_mut(mesh_handle).unwrap();
    *mesh = RectangleMeshBuilder::new(2., length)
        .build()
        .transformed_by(transform);
}

// fn zoom(
//     commands: Commands,
//     keys: Res<ButtonInput<KeyCode>>,
//     camera_query: Single<&mut Projection, With<Camera>>,
// ) {
//     let mut cam = camera_query.into_inner();
//     if keys.just_pressed(KeyCode::KeyJ) {
//         match *cam {
//             Projection::Orthographic(ref mut ortho) => ortho.scale -= 1.,
//             _ => panic!("help i cant find the right cam"),
//         }
//     }
//     if keys.just_pressed(KeyCode::KeyK) {
//         match *cam {
//             Projection::Orthographic(ref mut ortho) => ortho.scale += 1.,
//             _ => panic!("help i cant find the right cam"),
//         }
//     }
//     if keys.just_pressed(KeyCode::KeyL) {
//         match *cam {
//             Projection::Orthographic(ref mut ortho) => ortho.viewport_origin + 0.1,
//             _ => panic!("help i cant find the right cam"),
//         };
//     }
// }
