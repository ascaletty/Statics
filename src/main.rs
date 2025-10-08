use bevy::render::mesh::{Mesh, RectangleMeshBuilder};
use std::io;

use bevy::color::palettes::css::GRAY;
use bevy::prelude::*;
use bevy_cursor::prelude::*;
use std::collections::HashMap;
const SNAP_TOLERANCE: f32 = 10.;
mod command;
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
        if last
            .position
            .unwrap()
            .distance(cursor.position().unwrap_or(Vec2::ZERO))
            > 0.
        {
            spawn_line_preview(commands, &mut meshes, &mut ids, materials);
        }
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
                    .nodes
                    .clone()
                    .iter()
                    .filter(|x| x.pos.distance(cursorloc) < SNAP_TOLERANCE)
                    .min_by_key(|x| x.pos.distance(cursorloc) < SNAP_TOLERANCE)
                {
                    println!("old_node id{}", old_node.id);
                    let nodecount = truss.nodes.len();
                    let last_node = Node {
                        pos: last.position.unwrap_or(old_node.pos),
                        id: nodecount - 1,
                    };

                    let memcount = truss.edges.len();
                    let member = Member {
                        start: last_node,
                        end: old_node.clone(),
                        id: memcount + 1,
                    };

                    commands.queue(member.clone());
                    truss.edges.push(member);

                    last.position = Some(old_node.pos);
                } else {
                    let nodecount = truss.nodes.len();

                    let memcount = truss.edges.len();
                    let mut node = Node {
                        pos: cursorloc,
                        id: nodecount,
                    };
                    commands.queue(node.clone());
                    truss.nodes.push(node.clone());

                    if last.position.is_some() {
                        let last_node = Node {
                            pos: last.position.unwrap(),
                            id: nodecount - 1,
                        };
                        let member = Member {
                            start: last_node,
                            end: node.clone(),
                            id: memcount + 1,
                        };
                        commands.queue(member.clone());
                        truss.edges.push(member);
                    }
                    last.position = Some(node.pos);
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
                let mut roll = Connection::Roller(Vec2::ZERO, 20);
                if let Some(old_node) = truss
                    .nodes
                    .iter()
                    .find(|x| x.pos.distance(cursorloc) < SNAP_TOLERANCE)
                {
                    roll = Connection::Roller(old_node.pos, connection_count);
                    commands.queue(roll.clone());
                } else {
                    roll = Connection::Roller(cursorloc, connection_count);

                    commands.queue(Connection::Roller(cursorloc, connection_count));
                }
                truss.connections.push(roll);
            }
            if keys.just_pressed(KeyCode::KeyP) {
                let connection_count = truss.connections.len();

                let mut pin = Connection::Pin(Vec2::ZERO, 20);
                if let Some(old_node) = truss
                    .nodes
                    .iter()
                    .find(|x| x.pos.distance(cursorloc) < SNAP_TOLERANCE)
                {
                    pin = Connection::Pin(old_node.pos, connection_count);
                    commands.queue(pin.clone());
                } else {
                    pin = Connection::Pin(cursorloc, connection_count);
                    commands.queue(pin.clone());
                }
                truss.connections.push(pin);
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
                    .find(|x| x.pos.distance(cursorloc) < SNAP_TOLERANCE)
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
    // if cursor_loc.x == 0. {
    //     cursor_loc.x = 1.0;
    // }
    // if cursor_loc.y == 0. {
    //     cursor_loc.y = 1.0;
    // }
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

#[cfg(test)]
mod tests {
    use std::f32::consts::PI;

    use super::*;
    use faer::Mat;
    use faer::prelude::*;
    fn zeros(mut matrix: Mat<f32>) -> Mat<f32> {
        for i in 0..matrix.nrows() {
            for j in 0..matrix.ncols() {
                if matrix[(i, j)].abs() < 1e-4 {
                    matrix[(i, j)] = 0.;
                }
            }
        }
        matrix
    }

    use bevy::math::Vec2;
    use std::fs;
    use std::path::Path;
    use truss::structs::*;

    #[derive(Debug, serde::Deserialize)]
    struct RawTruss {
        nodes: Vec<String>,
        members: Vec<String>,
        supports: serde_json::Map<String, serde_json::Value>,
        forces: Vec<String>,
        workspace: serde_json::Value,
        awnsers: Vec<f32>,
    }

    pub fn load_trusses_from_folder<P: AsRef<Path>>(folder: P) -> Vec<bool> {
        let mut vecbool = Vec::new();

        for entry in fs::read_dir(folder).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            let file_name = path.file_name().unwrap().to_str().unwrap();

            if file_name.starts_with("truss") {
                let data = fs::read_to_string(&path).unwrap();
                let raw: RawTruss = serde_json::from_str(&data).unwrap();

                // Parse nodes
                let nodes: Vec<Node> = raw
                    .nodes
                    .iter()
                    .enumerate()
                    .map(|(i, s)| {
                        let parts: Vec<f32> = s
                            .split(',')
                            .map(|x| x.trim().parse::<f32>().unwrap())
                            .collect();
                        Node {
                            id: i,
                            pos: Vec2::new(parts[0], parts[1]),
                        }
                    })
                    .collect();

                // Parse members into your Member struct
                let edges: Vec<Member> = raw
                    .members
                    .iter()
                    .enumerate()
                    .map(|(i, s)| {
                        let parts: Vec<usize> = s
                            .split(',')
                            .map(|x| x.trim().parse::<usize>().unwrap())
                            .collect();
                        let start = nodes[parts[0]].clone();
                        let end = nodes[parts[1]].clone();
                        Member { id: i, start, end }
                    })
                    .collect();

                // TODO add connections
                let connections: Vec<Connection> = raw
                    .supports
                    .iter()
                    .enumerate()
                    .map(|(i, (node, connection_type))| {
                        if connection_type.as_str().unwrap() == "P" {
                            let at_node = nodes
                                .iter()
                                .find(|x| x.id == node.parse::<usize>().unwrap())
                                .unwrap();
                            Connection::Pin(at_node.pos, i)
                        } else {
                            let at_node = nodes
                                .iter()
                                .find(|x| x.id == node.parse::<usize>().unwrap())
                                .unwrap();
                            Connection::Roller(at_node.pos, i)
                        }
                    })
                    .collect();
                let forces= raw.forces
                let mut truss = Truss {
                    nodes,
                    edges,
                    selected_node: None,
                    dragging: None,
                    connections: connections,
                    connectionmap: HashMap::new(),
                    membermap: HashMap::new(),
                    nodemap: HashMap::new(),
                };
                let expirimental = physics::calculate_member_stress(&mut truss);

                let awnser_mat = Mat::from_fn(raw.awnsers.len(), 1, |i, j| raw.awnsers[i]);
                if awnser_mat == expirimental {
                    vecbool.push(true);
                } else {
                    vecbool.push(false);
                }
            }
        }
        vecbool
    }
    #[derive(serde::Deserialize)]
    struct MatrixWrapper {
        matrix: Vec<Vec<f32>>,
    }

    fn test_triangle() -> Mat<f32> {
        let mut truss = Truss {
            edges: vec![],
            nodes: vec![],
            nodemap: HashMap::new(),
            connections: vec![],
            connectionmap: HashMap::new(),
            selected_node: None,
            dragging: None,
            membermap: HashMap::new(),
        };
        let one = Vec2::new(0., 0.);
        let two = Vec2::new(1., 0.);
        let three = Vec2::new(0., 1.);
        let force_end = Vec2::new(0., -1.);
        let n0 = Node { pos: one, id: 0 };
        let n1 = Node { pos: two, id: 1 };
        let n2 = Node { pos: three, id: 2 };
        let m0 = Member {
            start: n0.clone(),
            end: n1.clone(),
            id: 0,
        };
        let m1 = Member {
            start: n1.clone(),
            end: n2.clone(),
            id: 1,
        };
        let m2 = Member {
            start: n2.clone(),
            end: n0.clone(),
            id: 2,
        };

        let p1 = Connection::Pin(one, 1);
        let r1 = Connection::Roller(two, 2);
        let f1 = Connection::Force(Force {
            start: one,
            end: force_end,
            id: 3,
            magnitude: 2.,
        });
        truss
            .nodes
            .append(&mut vec![n0.clone(), n1.clone(), n2.clone()]);
        truss
            .edges
            .append(&mut vec![m0.clone(), m1.clone(), m2.clone()]);
        truss
            .connections
            .append(&mut vec![p1.clone(), r1.clone(), f1.clone()]);
        physics::calculate_member_stress(&mut truss)
    }

    fn test_complex_triangle() -> Mat<f32> {
        let mut truss = Truss {
            edges: vec![],
            nodes: vec![],
            nodemap: HashMap::new(),
            connections: vec![],
            connectionmap: HashMap::new(),
            selected_node: None,
            dragging: None,
            membermap: HashMap::new(),
        };
        let one = Vec2::new(0., 0.);
        let two = Vec2::new(-2., 3.);
        let three = Vec2::new(5., 0.);
        let force_end = Vec2::new(-400., -400.);
        let n0 = Node { pos: one, id: 0 };
        let n1 = Node { pos: two, id: 1 };
        let n2 = Node { pos: three, id: 2 };
        let m0 = Member {
            start: n0.clone(),
            end: n1.clone(),
            id: 0,
        };
        let m1 = Member {
            start: n1.clone(),
            end: n2.clone(),
            id: 1,
        };
        let m2 = Member {
            start: n2.clone(),
            end: n0.clone(),
            id: 2,
        };

        let p1 = Connection::Pin(one, 1);
        let r1 = Connection::Roller(three, 2);
        let f1 = Connection::Force(Force {
            start: two,
            end: force_end,
            id: 3,
            magnitude: 400.,
        });
        truss
            .nodes
            .append(&mut vec![n0.clone(), n1.clone(), n2.clone()]);
        truss
            .edges
            .append(&mut vec![m0.clone(), m1.clone(), m2.clone()]);
        truss
            .connections
            .append(&mut vec![p1.clone(), r1.clone(), f1.clone()]);
        physics::calculate_member_stress(&mut truss)
    }

    fn complex_triangle() -> Mat<f32> {
        mat![
            [-681.55347],
            [717.1116],
            [-659.1295],
            [281.07166],
            [567.0868],
            [-282.48407],
        ]
    }

    fn basic_triangle() -> Mat<f32> {
        mat![[0.], [0.], [0.], [0.], [2.], [0.],]
    }

    #[test]
    fn test_batch() {
        let vecbool = load_trusses_from_folder("test_trusses");
        for bools in vecbool {
            assert!(bools, "the  truss had an error");
        }
    }
    // fn forcing_matrix__match() {
    //     let true_output = basic_triangle();
    //     let output = test_triangle();
    //
    //     assert_eq!(output.forcing, true_output.forcing);
    // }
    //
    // #[test]
    // fn val_matrix_match() {
    //     let true_output = basic_triangle();
    //
    //     let output = test_triangle();
    //     assert_eq!(output.matrix, true_output.matrix);
    // }
    //
    #[test]
    fn awnsers_match() {
        let true_output = basic_triangle();

        let output = test_triangle();

        assert_eq!(zeros(output), zeros(true_output));
    }
    #[test]
    fn awnsers_match_complex() {
        let true_output = complex_triangle();
        let output = test_complex_triangle();

        assert_eq!(zeros(output), zeros(true_output));
    }
}
