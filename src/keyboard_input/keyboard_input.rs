use crate::physics_dir::physics;
use crate::structs_dir::structs::*;
use bevy::ecs::event::EventReader;
use bevy::input::ButtonState;
use bevy::input::keyboard;
use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::{AssetId, Assets, ButtonInput, Commands, KeyCode, Mesh, Res, ResMut, Vec2};
use bevy::render::mesh::MeshVertexAttributeId;
use bevy::state::commands;
use bevy_cursor::prelude::*;

const SNAP_TOLERANCE: f32 = 10.;

use bevy::input::keyboard::Key;
use bevy::prelude::Resource;
use faer::traits::pulp::core_arch::x86;
pub fn handle_text_input(
    mut evr_kbd: EventReader<KeyboardInput>,
    mut mode: ResMut<Mode>,
    last: Res<LastNode>,
    mut truss: ResMut<Truss>,
    cursor: Res<CursorLocation>,
    mut commands: Commands,
) {
    let mut buf = String::new();
    let cursorloc = cursor.world_position().unwrap();
    let connection_count = truss.connections.len();
    for ev in evr_kbd.read() {
        if ev.state == ButtonState::Released {
            continue;
        }
        match &ev.logical_key {
            Key::Enter => {
                *mode = Mode::Command;
                match buf.parse() {
                    Ok(input) => {
                        let force = Force {
                            magnitude: input,
                            start: last.position.unwrap(),
                            end: cursorloc,
                            id: connection_count,
                        };
                        if let Some(_old_node) = truss
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
                    Err(err) => {
                        buf.clear();
                        println!("Please only intger values of magnitudes");
                    }
                }
            }

            Key::Backspace => {
                buf.pop();
            }
            Key::Character(ch) => {
                if ch.chars().any(|c| !c.is_control()) {
                    buf.push_str(ch);
                }
            }
            _ => {}
        }
    }
}
pub fn handle_insert(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    mut mode: ResMut<Mode>,
    mut last: ResMut<LastNode>,
    cursor: Res<CursorLocation>,
    mut truss: ResMut<Truss>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
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
                id: memcount,
            };
            if member.start.pos.distance(member.end.pos) > 1e-6 {
                commands.queue(member.clone());
                truss.edges.push(member);
            }

            last.position = Some(old_node.pos);
        } else {
            let nodecount = truss.nodes.len();

            let memcount = truss.edges.len();
            let node = Node {
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
                    id: memcount,
                };
                commands.queue(member.clone());
                truss.edges.push(member);
            }
            last.position = Some(node.pos);
        }
    }
    if keys.just_pressed(KeyCode::Escape) {
        *mode = Mode::Command;
        if truss.preview.is_some() {
            meshes.remove(truss.preview.unwrap());
            truss.preview = None;
            if last.position.is_none() {
                truss.preview = None;
            }
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
            .filter(|x| x.pos.distance(cursorloc) < SNAP_TOLERANCE)
            .min_by_key(|x| x.pos.distance(cursorloc) < SNAP_TOLERANCE)
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
        *mode = Mode::InsertText;
    }
    // we can check multiple at once with `.any_*`
    if keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]) {
        // Either the left or right shift are being held down
    }
    if keys.any_just_pressed([KeyCode::Delete, KeyCode::Backspace]) {
        // Either delete or backspace was just pressed
    }
}
pub fn handle_command(
    mut mode: ResMut<Mode>,
    mut last: ResMut<LastNode>,
    keys: Res<ButtonInput<KeyCode>>,
    truss: ResMut<Truss>,
) {
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
pub fn handle_dimension(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    mut mode: ResMut<Mode>,
    mut last: ResMut<LastNode>,
    cursor: Res<CursorLocation>,
    mut truss: ResMut<Truss>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let cursorloc = cursor.world_position().unwrap_or(Vec2::ZERO);
    let mut pair = vec![];
    if keys.just_pressed(KeyCode::KeyD) {
        let matching_node = truss
            .nodes
            .iter()
            .filter(|x| x.pos.distance(cursorloc) < SNAP_TOLERANCE)
            .min_by_key(|x| x.pos.distance(cursorloc) < SNAP_TOLERANCE)
            .unwrap();
        pair.push(matching_node);
        if pair.len() == 2 {
            //fix
            pair.clear();
        }
    }
}
