use bevy::{asset::ErasedAssetLoader, prelude::*};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Connection {
    Pin(Vec2, usize),
    Roller(Vec2, usize),
    Force(Force),
}

impl Command for Connection {
    fn apply(self, world: &mut World) {
        match self {
            Connection::Roller(pos, id) => {
                let mesh_handle = world.resource_scope(|_world, mut meshes: Mut<Assets<Mesh>>| {
                    let circ = Circle::new(10.0);
                    meshes.add(circ)
                });

                let color_material =
                    world.resource_scope(|_world, mut materials: Mut<Assets<ColorMaterial>>| {
                        let blue = Color::srgb(0.0, 1.0, 0.0);
                        materials.add(blue)
                    });
                world.resource_scope(|_world, mut member_ids: Mut<Truss>| {
                    member_ids.connectionmap.insert(id, mesh_handle.clone())
                });

                world.spawn((
                    Mesh2d(mesh_handle.clone()),
                    MeshMaterial2d(color_material.clone()),
                    Transform::from_xyz(pos.x, pos.y, 0.),
                ));
            }
            Connection::Pin(pos, id) => {
                let mesh_handle = world.resource_scope(|_world, mut meshes: Mut<Assets<Mesh>>| {
                    let rect = Rectangle::new(10.0, 10.0);
                    meshes.add(rect)
                });

                let color_material =
                    world.resource_scope(|_world, mut materials: Mut<Assets<ColorMaterial>>| {
                        let blue = Color::srgb(1.0, 0.0, 0.0);
                        materials.add(blue)
                    });
                world.resource_scope(|_world, mut member_ids: Mut<Truss>| {
                    member_ids.connectionmap.insert(id, mesh_handle.clone())
                });

                world.spawn((
                    Mesh2d(mesh_handle.clone()),
                    MeshMaterial2d(color_material.clone()),
                    Transform::from_xyz(pos.x, pos.y, 0.),
                ));
            }
            Connection::Force(Force {
                start,
                end,
                magnitude: mag,
                id,
            }) => {
                let length = start.distance(end);
                let diff = start - end;
                let mut theta = diff.x / diff.y;
                theta = theta.atan();
                let midpoint = (start + end) / 2.;
                let transform = Transform::from_xyz(midpoint.x, midpoint.y, 0.)
                    .with_rotation(Quat::from_rotation_z(-theta));
                let mesh_handle = world.resource_scope(|_world, mut meshes: Mut<Assets<Mesh>>| {
                    let tri = Triangle2d::new(
                        end + Vec2::new(-2., 0.),
                        end + Vec2::new(2., 0.),
                        end + Vec2::new(0., 1.),
                    );
                    meshes.add(tri)
                });
                let line_handle = world.resource_scope(|_world, mut meshes: Mut<Assets<Mesh>>| {
                    let rect = Rectangle::new(2., length);
                    meshes.add(rect)
                });

                let color_material =
                    world.resource_scope(|_world, mut materials: Mut<Assets<ColorMaterial>>| {
                        let blue = Color::srgb(1.0, 0.0, 0.0);
                        materials.add(blue)
                    });
                // world.resource_scope(|_world, mut member_ids: Mut<Truss>| {
                //     member_ids.connectionmap.insert(id, mesh_handle.clone())
                // });

                world.spawn((
                    Mesh2d(line_handle.clone()),
                    MeshMaterial2d(color_material.clone()),
                    transform,
                ));
            }
        }
    }
}

#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Insert,
    Edit,
    Dimension,
    Command,
    InsertText,
}
#[derive(Resource, Debug, Clone)]
pub struct Force {
    pub start: Vec2,
    pub end: Vec2,
    pub magnitude: f32,
    pub id: usize,
}

#[derive(Resource, Clone)]
pub struct Truss {
    pub nodes: Vec<Node>, // node entities in order
    pub edges: Vec<Member>,
    pub preview: Option<AssetId<Mesh>>,
    pub selected_node: Option<Node>,
    pub dragging: Option<Node>,
    pub connections: Vec<Connection>,
    pub membermap: HashMap<usize, Handle<Mesh>>,
    pub nodemap: HashMap<usize, Handle<Mesh>>,
    pub connectionmap: HashMap<usize, Handle<Mesh>>,
}

#[derive(Component, Clone, Debug)]
pub struct Member {
    pub start: Node,
    pub end: Node,
    pub id: usize,
}

impl Command for Member {
    fn apply(self, world: &mut World) {
        let length = self.start.pos.distance(self.end.pos);
        let diff = self.start.pos - self.end.pos;
        let mut theta = diff.x / diff.y;
        theta = theta.atan();
        let midpoint = (self.start.pos + self.end.pos) / 2.;
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
        world.resource_scope(|_world, mut member_ids: Mut<Truss>| {
            member_ids.membermap.insert(self.id, mesh_handle.clone())
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
pub struct Node {
    pub pos: Vec2,
    pub id: usize,
}
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
        world.resource_scope(|_world, mut member_ids: Mut<Truss>| {
            member_ids.nodemap.insert(self.id, mesh_handle.clone())
        });
        let x = self.pos.x;
        let y = self.pos.y;
        world.spawn((
            self,
            Mesh2d(mesh_handle.clone()),
            MeshMaterial2d(color_material.clone()),
            Transform::from_xyz(x, y, 0.),
        ));
    }
}

#[derive(Resource)]
pub struct LastNode {
    pub position: Option<Vec2>,
}

#[derive(Resource)]
pub struct TextBuffer(pub String);
