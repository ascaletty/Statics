use bevy::asset::{RenderAssetUsages, uuid};
use bevy::render::mesh::{Indices, Mesh, RectangleMeshBuilder};

use bevy::color::palettes::css::{CORAL, GRAY};
use bevy::math::{VectorSpace, bounding::*};
use bevy::state::commands;
use bevy::winit::cursor;
use bevy::{gizmos, prelude::*, sprite};
use bevy_cursor::prelude::*;
use std::collections::HashMap;
use std::process::id;
const SNAP_TOLERANCE: f32 = 10.;
#[derive(Debug)]
enum Connection {
    Pin(Vec2),
    Roller(Vec2),
}
impl Command for Connection {
    fn apply(self, world: &mut World) -> () {
        match self {
            Connection::Roller(pos) => {
                let mesh_handle = world.resource_scope(|_world, mut meshes: Mut<Assets<Mesh>>| {
                    let circ = Circle::new(10.0);
                    meshes.add(circ)
                });

                let color_material =
                    world.resource_scope(|_world, mut materials: Mut<Assets<ColorMaterial>>| {
                        let blue = Color::srgb(0.0, 1.0, 0.0);
                        materials.add(blue)
                    });

                world.spawn((
                    Mesh2d(mesh_handle.clone()),
                    MeshMaterial2d(color_material.clone()),
                    Transform::from_xyz(pos.x, pos.y, 0.),
                ));
            }
            Connection::Pin(pos) => {}
        }
    }
}

#[derive(Resource, Clone, Copy, Debug)]
enum Mode {
    Insert,
    Edit,
    Dimension,
    Command,
}

#[derive(Resource)]
struct Truss {
    nodes: Vec<Vec2>, // node entities in order
    edges: Vec<Member>,
    selected_node: Option<Node>,
    dragging: Option<Node>,
    connections: Vec<Connection>,
}

#[derive(Resource)]
struct MembersIds {
    idmap: HashMap<u32, Handle<Mesh>>,
}

#[derive(Resource)]
struct MemberCount {
    count: u32,
}
#[derive(Component, Clone, Debug)]
struct Member {
    start: Vec2,
    end: Vec2,
    id: u32,
}

impl Command for Member {
    fn apply(self, world: &mut World) {
        let length = self.start.distance(self.end);
        let diff = self.start - self.end;
        let mut theta = diff.x / diff.y;
        theta = theta.atan();
        let midpoint = (self.start + self.end) / 2.;
        let transform = Transform::from_xyz(midpoint.x, midpoint.y, 0.)
            .with_rotation(Quat::from_rotation_z(-theta));
        let mesh_handle = world.resource_scope(|_world, mut meshes: Mut<Assets<Mesh>>| {
            let rect = Rectangle::new(2., length);
            meshes.add(rect)
        });
        let color_material =
            world.resource_scope(|_world, mut materials: Mut<Assets<ColorMaterial>>| {
                let blue = Color::srgb(0.0, 0.0, 1.0);
                materials.add(blue)
            });
        world.resource_scope(|_world, mut member_ids: Mut<MembersIds>| {
            member_ids.idmap.insert(self.id, mesh_handle.clone())
        });
        println!(
            "Creating idmap with id {}, mesh_handle {:?}",
            self.id, mesh_handle
        );
        world.spawn((
            self,
            Mesh2d(mesh_handle.clone()),
            MeshMaterial2d(color_material.clone()),
            transform,
        ));
    }
}

#[derive(Component, Clone, Debug)]
struct Node(Vec2);
impl Command for Node {
    fn apply(self, world: &mut World) {
        let mesh_handle = world.resource_scope(|_world, mut meshes: Mut<Assets<Mesh>>| {
            let circle = Circle::new(5.);
            meshes.add(circle)
        });
        let color_material =
            world.resource_scope(|_world, mut materials: Mut<Assets<ColorMaterial>>| {
                let blue = Color::srgb(0.0, 0.0, 1.0);
                materials.add(blue)
            });
        let x = self.0.x;
        let y = self.0.y;
        world.spawn((
            self,
            Mesh2d(mesh_handle.clone()),
            MeshMaterial2d(color_material.clone()),
            Transform::from_xyz(x, y, 0.),
        ));
    }
}

#[derive(Resource)]
struct LastNode {
    position: Option<Vec2>,
}

#[derive(Resource)]
struct NodeCount(u32);
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
        })
        .insert_resource(NodeCount(0))
        .insert_resource(MembersIds {
            idmap: HashMap::new(),
        })
        .insert_resource(LastNode { position: None })
        .insert_resource(MemberCount { count: 0 })
        .add_plugins((DefaultPlugins, TrackCursorPlugin))
        .add_systems(Startup, setup_camera)
        .add_systems(Update, keyboard_input)
        .add_systems(Update, preview_on)
        .add_systems(Startup, grid)
        .add_systems(Update, zoom)
        .run();
}
fn preview_on(
    truss: Res<Truss>,
    mode: Res<Mode>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut ids: ResMut<MembersIds>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    cursor: ResMut<CursorLocation>,
    last: Res<LastNode>,
) {
    let insert = match mode.into_inner() {
        Mode::Insert => true,
        _ => false,
    };
    let previewspawned = ids.idmap.contains_key(&0);
    let last_exist = last.position.is_some();
    if !previewspawned && last_exist {
        spawn_line_preview(commands, &mut meshes, &mut ids, materials);
    }

    if truss.nodes.len() > 0 && insert && previewspawned && last_exist {
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
    mut count: ResMut<MemberCount>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut ids: ResMut<MembersIds>,
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
        }
        Mode::Insert => {
            let cursorloc = cursor.world_position().unwrap_or(Vec2::ZERO);
            if keys.just_pressed(KeyCode::Space) {
                let node = Node(Vec2::new(cursorloc.x, cursorloc.y));
                if let Some(old_node) = truss
                    .nodes
                    .iter()
                    .find(|x| x.distance(cursorloc) < SNAP_TOLERANCE)
                {
                    commands.queue(Member {
                        start: last.position.unwrap_or(*old_node),
                        end: *old_node,
                        id: count.count,
                    });

                    last.position = Some(*old_node);
                } else {
                    truss.nodes.push(cursorloc);

                    count.count += 1;

                    commands.queue(Node(cursorloc));
                    if last.position.is_some() {
                        let member = Member {
                            start: last.position.unwrap(),
                            end: cursorloc,
                            id: count.count,
                        };
                        truss.edges.push(member.clone());
                        commands.queue(member);
                    }

                    last.position = Some(node.0);
                }
            }
            if keys.just_pressed(KeyCode::Escape) {
                *mode = Mode::Command;
                meshes.remove(ids.idmap.get(&0).unwrap().id());
                ids.idmap.remove(&0);
            }
            if keys.just_pressed(KeyCode::KeyR) {
                if let Some(old_node) = truss
                    .nodes
                    .iter()
                    .find(|x| x.distance(cursorloc) < SNAP_TOLERANCE)
                {
                    commands.queue(Connection::Roller(*old_node));
                } else {
                    commands.queue(Connection::Roller(cursorloc));
                }
                truss.connections.push(Connection::Roller(cursorloc));
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
    ids: &mut ResMut<MembersIds>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mesh_handle = meshes.add(RectangleMeshBuilder::new(0.0, 0.0).build());
    let color_handle = materials.add(Color::WHITE);
    ids.idmap.insert(0, mesh_handle.clone());

    commands.spawn((
        Mesh2d(mesh_handle),
        MeshMaterial2d(color_handle),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
}

fn update_line_preview(
    cursor: ResMut<CursorLocation>, // to read cursor position
    ids: ResMut<MembersIds>,
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

    let mesh_handle = ids.idmap.get(&0).unwrap();
    let mesh = meshes.get_mut(mesh_handle).unwrap();
    *mesh = RectangleMeshBuilder::new(2., length)
        .build()
        .transformed_by(transform);
}
use bevy::render::mesh::PrimitiveTopology;

fn grid(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut mesh = Mesh::new(PrimitiveTopology::LineList, RenderAssetUsages::default());

    let mut positions = Vec::new();
    let mut indices = Vec::new();

    let step = 50.0;
    let half = 500.0;

    let mut i = 0;
    for x in (-10..=10).map(|i| i as f32 * step) {
        positions.push([x, -half, 0.0]);
        positions.push([x, half, 0.0]);
        indices.push(i);
        indices.push(i + 1);
        i += 2;
    }

    for y in (-10..=10).map(|i| i as f32 * step) {
        positions.push([-half, y, 0.0]);
        positions.push([half, y, 0.0]);
        indices.push(i);
        indices.push(i + 1);
        i += 2;
    }

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);

    commands.spawn((
        Mesh2d(meshes.add(mesh)),
        MeshMaterial2d(materials.add(Color::WHITE)),
    ));
}
fn zoom(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    camera_query: Single<&mut Projection, With<Camera>>,
) {
    let mut cam = camera_query.into_inner();
    if keys.just_pressed(KeyCode::KeyJ) {
        match *cam {
            Projection::Orthographic(ref mut ortho) => ortho.scale -= 1.,
            _ => panic!("help i cant find the right cam"),
        }
    }
    if keys.just_pressed(KeyCode::KeyK) {
        match *cam {
            Projection::Orthographic(ref mut ortho) => ortho.scale += 1.,
            _ => panic!("help i cant find the right cam"),
        }
    }
}
