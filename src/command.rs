use bevy::prelude::Node as Bevy_Node;
use bevy::prelude::*;
pub fn command_bar(mut commands: Commands) {}
pub fn spawn_command_bar(mut commands: Commands) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(5.0),
            right: Val::Px(5.0),
            ..default()
        },
        Text::new("command_bar "),
    ));
}
