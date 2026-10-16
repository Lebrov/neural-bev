//! # NeuralBev Editor (Phase 2 stub)
//!
//! ## AI-HINT
//! Редактор реализуется в Фазе 2 (egui + bevy_egui).
//! Сейчас это заглушка, которая запускает рантайм с основными системами.

use bevy::prelude::*;
use neuralbev_core::NeuralBevCorePlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "NeuralBev Editor v0.1 (Phase 2 coming)".to_string(),
                resolution: (1600.0, 900.0).into(),
                ..Default::default()
            }),
            ..Default::default()
        }))
        .add_plugins(NeuralBevCorePlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    info!("[Editor] Фаза 2 в разработке. Сейчас запущен базовый рантайм.");
}
