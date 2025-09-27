use bevy::ecs::entity;
use bevy::render::mesh::{Indices, Mesh};
use bevy::state::commands;

use bevy::color::palettes::css::GRAY;
use bevy::math::{VectorSpace, bounding::*};
use bevy::{prelude::*, sprite};
use bevy_cursor::prelude::*;

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
#[derive(Component, Clone, Debug)]
struct Member {
    start: Vec2,
    end: Vec2,
}
impl Command for Member {
    fn apply(self, world: &mut World) {
        let length = self.start.distance(self.end);
        let diff = self.start - self.end;
        let theta = diff.y.atan2(diff.x);
        let midpoint = (self.start + self.end) / 2.;
        let transform = Transform::from_xyz(midpoint.x, midpoint.y, 0.)
            .with_rotation(Quat::from_rotation_z(theta));
        world.spawn((
            Sprite {
                color: Color::WHITE,
                custom_size: Some(Vec2::new(length, 2.)),
                ..Default::default()
            },
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

#[derive(Resource)]
struct LinePreview {
    start: Vec2,
    end: Vec2,
}

impl Command for LinePreview {
    fn apply(self, world: &mut World) {
        let length = self.start.distance(self.end);
        let diff = self.start - self.end;
        let theta = diff.y.atan2(diff.x);
        let midpoint = (self.start + self.end) / 2.;
        let transform = Transform::from_xyz(midpoint.x, midpoint.y, 0.)
            .with_rotation(Quat::from_rotation_z(theta));
        world.spawn((
            Sprite {
                color: Color::WHITE,
                custom_size: Some(Vec2::new(length, 2.)),
                ..Default::default()
            },
            transform,
        ));
    }
}

// #[derive(Component)]
// struct Edge {
//     position: Vec2,
// } // marker for edge entities

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
        .insert_resource(LinePreview {
            start: Vec2::new(0., 0.),
            end: Vec2::new(0., 0.),
        })
        .insert_resource(PreviewOn(true))
        .insert_resource(LastNode { position: None })
        .add_plugins((DefaultPlugins, TrackCursorPlugin))
        .add_systems(Startup, setup_camera)
        .add_systems(Update, keyboard_input)
        // .add_systems(Update, update_line_preview)
        .add_systems(Update, spawn_line_preview)
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
    preview: ResMut<LinePreview>,
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
            let cursorloc = cursor.world_position().unwrap();
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
                    });

                    last.position = Some(*old_node);
                } else {
                    truss.nodes.push(cursorloc);
                    commands.queue(Node(cursorloc));
                    if last.position.is_some() {
                        commands.queue(Member {
                            start: last.position.unwrap(),
                            end: cursorloc,
                        });
                    }

                    last.position = Some(node.0);
                }
            }
            if keys.just_pressed(KeyCode::Escape) {
                *mode = Mode::Command;
            }
            if keys.just_pressed(KeyCode::KeyQ) {
                update_line_preview(cursorloc, last.position.unwrap(), preview);
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
// fn add_roller(
//     mut commands: Commands,
//     cursor: Vec2,
//     mut meshes: ResMut<Assets<Mesh>>,
//     mut truss: ResMut<Truss>,
//
//     mut materials: ResMut<Assets<ColorMaterial>>,
// ) {
//     truss.connections.push(Connection::roller(cursor));
//     let circle = meshes.add(Circle::new(5.));
//     commands.spawn((
//         Mesh2d(circle),
//         MeshMaterial2d(materials.add(Color::srgb(0., 256., 0.))),
//         Transform::from_xyz(cursor.x, cursor.y, 0.),
//     ));
// }
fn update_line_preview(cursor: Vec2, last: Vec2, mut preview: ResMut<LinePreview>) {
    preview.start = last;
    preview.end = cursor;
}

fn spawn_line_preview(
    mut commands: Commands,
    cursor: Res<CursorLocation>,
    last: Res<LastNode>,
    mut previewon: ResMut<PreviewOn>,
) {
    commands.queue(LinePreview {
        start: last.position.unwrap_or(Vec2::ZERO),
        end: cursor.world_position().unwrap_or(Vec2::new(100., 100.)),
    });
}
