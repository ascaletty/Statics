use bevy::asset::RenderAssetUsages;
use bevy::ecs::entity;
use bevy::render::mesh::{Indices, Mesh, RectangleMeshBuilder};
use bevy::state::commands;

use bevy::color::palettes::css::{CORAL, GRAY};
use bevy::math::{VectorSpace, bounding::*};
use bevy::{gizmos, prelude::*, sprite};
use bevy_cursor::prelude::*;
use std::collections::HashMap;
use std::process::id;

#[derive(Debug)]
enum Connection {
    Pin(Vec2),
    Roller(Vec2),
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
struct PreviewOn(bool);

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::Srgba(GRAY)))
        .insert_resource(Mode::Command)
        .insert_resource(Truss {
            nodes: vec![],
            edges: vec![],
            selected_node: None,
            dragging: None,
            connections: vec![],
        })
        .insert_resource(MembersIds {
            idmap: HashMap::new(),
        })
        .insert_resource(LastNode { position: None })
        .insert_resource(MemberCount { count: 0 })
        .add_plugins((DefaultPlugins, TrackCursorPlugin))
        .add_systems(Startup, setup_camera)
        .add_systems(Update, keyboard_input)
        .add_systems(Startup, spawn_line_preview)
        .add_systems(Update, update_line_preview)
        .run();
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
    ids: Res<MembersIds>,
    assets: ResMut<Assets<Mesh>>,
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
                let snap_tolerance = 10.0;
                if let Some(old_node) = truss
                    .nodes
                    .iter()
                    .find(|x| x.distance(cursorloc) < snap_tolerance)
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

                    print!("{}", count.count);
                    commands.queue(Node(cursorloc));
                    if last.position.is_some() {
                        commands.queue(Member {
                            start: last.position.unwrap(),
                            end: cursorloc,
                            id: count.count,
                        });
                    }

                    last.position = Some(node.0);
                }
            }
            if keys.just_pressed(KeyCode::Escape) {
                *mode = Mode::Command;
            }
            if keys.just_pressed(KeyCode::KeyQ) {
                // Left Ctrl was released
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
fn delete_components(
    mut commands: Commands,
    points: Query<Entity, With<Mesh2d>>,
    lines: Query<Entity, With<Sprite>>,
) {
    for entity in points.iter() {
        commands.entity(entity).remove::<Mesh2d>();
    }
    for entity in lines.iter() {
        commands.entity(entity).remove::<Sprite>();
    }
}
// fn update_line_preview(
//     cursor: Vec2,
//     ids: Res<MembersIds>,
//     mut assets: ResMut<Assets<Mesh>>,
//     mut commands: Commands,
// ) {
//     let mesh_handle = ids.idmap.get(&0).unwrap();
//     let mesh = assets.get_mut(mesh_handle).unwrap();
//     mesh.transform_by(Transform::from_xyz(cursor.x, cursor.y, 0.));
// }
fn spawn_line_preview(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut ids: ResMut<MembersIds>,
    mut materails: ResMut<Assets<ColorMaterial>>,
) {
    let mesh_handle = meshes.add(RectangleMeshBuilder::new(20.0, 1.0).build());
    let color_handle = materails.add(Color::WHITE);
    ids.idmap.insert(0, mesh_handle.clone());

    commands.spawn((
        Mesh2d(mesh_handle),
        MeshMaterial2d(color_handle),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
}

fn update_line_preview(
    cursor: ResMut<CursorLocation>, // to read cursor position
    ids: Res<MembersIds>,
    last: Res<LastNode>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    // Replace mesh data with a line from last_point to cursor
    let last_point = last.position.unwrap_or(Vec2::ZERO);
    let mut cursor_loc = cursor.world_position().unwrap_or(Vec2::ZERO);

    let length = last_point.distance(cursor_loc);
    print!("lenght{}", length);
    if cursor_loc.x == 0. {
        cursor_loc.x = 1.0;

        print!("cursor_loc x= 0")
    }
    if cursor_loc.y == 0. {
        cursor_loc.y = 1.0;
        print!("cursor_loc y= 0")
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
