//! Point d'entrée du client R2

use bevy::prelude::*;

fn main() {
    let mut app = App::new();

    // Plugins par défaut de Bevy
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
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
    // Logique de base
    .add_systems(Update, bevy::window::close_on_esc);

    // Run
    app.run();
}
