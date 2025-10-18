use crate::structs::*;
use bevy::prelude::Node as Bevy_Node;
use bevy::prelude::*;
pub fn spawn_command_bar(mut commands: Commands, assets: Res<AssetServer>) -> Entity {
    let font = assets.load("fonts/FiraCodeNerdFont-Regular.ttf");
    commands
        .spawn((
            Text2d::new("Insert"),
            TextColor(Color::WHITE),
            TextFont::from_font(font).with_font_size(30.),
        ))
        .id()
}
pub fn handle_command_bar(mut commands: Commands, assets: Res<AssetServer>, mode: Res<Mode>) {
    match *mode {
        Mode::Insert => {}
        Mode::Edit => {}
        Mode::InsertText => {}
        Mode::Command => {}
        Mode::Dimension => {}
    }
}
