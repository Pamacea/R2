//! Gestion des entrées clavier pour le client R2
//!
//! Supporte ZQSD (AZERTY) et les flèches directionnelles.

use bevy::prelude::*;
use r2_core::Vec2;

/// État des entrées du joueur
#[derive(Resource, Clone, Debug, Default)]
pub struct InputState {
    /// Direction du mouvement (normalisé)
    pub movement_direction: Vec2,
    /// Position cible pour le clic de souris
    pub mouse_target: Option<Vec2>,
    /// Tourner vers la souris
    pub look_at_mouse: bool,
    /// Touche de tir pressée
    pub fire_pressed: bool,
    /// Touche de skill (1-8)
    pub skill_pressed: [bool; 8],
    /// Touche d'interaction (E)
    pub interact_pressed: bool,
    /// Touche de ciblage (Tab)
    pub target_next_pressed: bool,
    /// Touche de sprint/dash (Shift)
    pub sprint_pressed: bool,
}

impl InputState {
    /// Crée un nouvel état d'entrée
    pub fn new() -> Self {
        Self::default()
    }

    /// Réinitialise les états "just pressed"
    pub fn reset_transient(&mut self) {
        self.fire_pressed = false;
        self.interact_pressed = false;
        self.target_next_pressed = false;
        for skill in self.skill_pressed.iter_mut() {
            *skill = false;
        }
    }

    /// Retourne true si le joueur essaie de bouger
    pub fn is_moving(&self) -> bool {
        self.movement_direction.length() > 0.01
    }
}

/// Réinitialise les états transient après les avoir lus
fn reset_transient_state(mut input: ResMut<InputState>) {
    input.reset_transient();
}

/// Plugin pour la gestion des entrées
pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputState>()
            .add_systems(Startup, setup_input)
            .add_systems(PreUpdate, (
                handle_keyboard,
                handle_mouse,
                reset_transient_state,
            ).chain());
    }
}

/// Setup des entrées
fn setup_input() {
    info!("Input system initialized - ZQSD / Arrows supported");
}

/// Gestion du clavier
fn handle_keyboard(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut input: ResMut<InputState>,
) {
    // Movement - ZQSD (AZERTY) et Flèches
    let mut dir_x = 0.0;
    let mut dir_y = 0.0;

    // Z / Up
    if keyboard.pressed(KeyCode::KeyZ) || keyboard.pressed(KeyCode::ArrowUp) {
        dir_y += 1.0;
    }
    // S / Down
    if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
        dir_y -= 1.0;
    }
    // Q / Left
    if keyboard.pressed(KeyCode::KeyQ) || keyboard.pressed(KeyCode::ArrowLeft) {
        dir_x -= 1.0;
    }
    // D / Right
    if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
        dir_x += 1.0;
    }

    // Normaliser la direction
    input.movement_direction = if dir_x != 0.0 || dir_y != 0.0 {
        let len = f32::sqrt(dir_x * dir_x + dir_y * dir_y);
        Vec2 {
            x: dir_x / len,
            y: dir_y / len,
        }
    } else {
        Vec2::ZERO
    };

    // Sprint (Shift gauche ou droit)
    input.sprint_pressed =
        keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);

    // Tir (Espace ou Clic gauche géré dans handle_mouse)
    input.fire_pressed = keyboard.just_pressed(KeyCode::Space);

    // Interaction (E)
    input.interact_pressed = keyboard.just_pressed(KeyCode::KeyE);

    // Ciblage (Tab)
    input.target_next_pressed = keyboard.just_pressed(KeyCode::Tab);

    // Skills (1-8)
    for (i, key) in [
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::Digit3,
        KeyCode::Digit4,
        KeyCode::Digit5,
        KeyCode::Digit6,
        KeyCode::Digit7,
        KeyCode::Digit8,
    ]
    .iter()
    .enumerate()
    {
        if keyboard.just_pressed(*key) {
            input.skill_pressed[i] = true;
        }
    }
}

/// Gestion de la souris
fn handle_mouse(
    windows: Query<&Window>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut input: ResMut<InputState>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
) {
    let Ok(window) = windows.get_single() else {
        return;
    };

    let (camera, camera_transform) = match camera_query.get_single() {
        Ok(c) => c,
        Err(_) => return,
    };

    // Vérifier si la souris est dans la fenêtre
    let Some(cursor_position) = window.cursor_position() else {
        input.look_at_mouse = false;
        return;
    };

    input.look_at_mouse = true;

    // Convertir la position souris en monde
    if let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_position) {
        input.mouse_target = Some(Vec2 {
            x: world_pos.x,
            y: world_pos.y,
        });
    }

    // Clic gauche pour tirer
    input.fire_pressed = input.fire_pressed || buttons.just_pressed(MouseButton::Left);
}

/// Extension pour convertir Vec2 de r2_core vers bevy::math::Vec2
pub trait ToBevyVec2 {
    fn to_bevy(self) -> bevy::math::Vec2;
}

impl ToBevyVec2 for Vec2 {
    fn to_bevy(self) -> bevy::math::Vec2 {
        bevy::math::Vec2::new(self.x, self.y)
    }
}

/// Extension pour convertir bevy::math::Vec2 vers r2_core::Vec2
pub trait ToR2Vec2 {
    fn to_r2(self) -> Vec2;
}

impl ToR2Vec2 for bevy::math::Vec2 {
    fn to_r2(self) -> Vec2 {
        Vec2 { x: self.x, y: self.y }
    }
}
