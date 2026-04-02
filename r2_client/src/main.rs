//! Point d'entrée du client R2

use bevy::prelude::*;

fn main() {
    App::new()
        // Plugins par défaut de Bevy
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "R2 - Spatial MMO".to_string(),
                resolution: (1280., 720.).into(),
                resizable: true,
                decorations: true,
                ..default()
            }),
            ..default()
        }))
        // Plugin R2
        .add_plugins(r2_client::R2ClientPlugin)
        // Système pour quitter avec ESC
        .add_systems(Update, exit_on_esc)
        .run();
}

/// Quitter l'application quand on appuie sur Escape
fn exit_on_esc(keyboard: Res<ButtonInput<KeyCode>>, mut exit: EventWriter<AppExit>) {
    if keyboard.just_pressed(KeyCode::Escape) {
        exit.send(AppExit::Success);
    }
}
