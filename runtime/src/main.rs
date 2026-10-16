//! # NeuralBev Runtime
//!
//! Запуск игры без редактора — только NeuralBevCorePlugin + DefaultPlugins.
//!
//! ## Использование
//! ```bash
//! cargo run --bin runtime
//! ```
//!
//! ## AI-HINT
//! Это минимальный рантайм. Добавляй сюда свои плагины для конкретной игры.
//! Редактор не включён — только игровой loop.

use bevy::prelude::*;
use neuralbev_core::NeuralBevCorePlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "NeuralBev Runtime".to_string(),
                resolution: (1280.0, 720.0).into(),
                ..Default::default()
            }),
            ..Default::default()
        }))
        .add_plugins(NeuralBevCorePlugin)
        .add_systems(Startup, setup)
        .run();
}

/// Начальная сцена рантайма — загружает демо-сцену.
fn setup(mut commands: Commands, mut ev_spawn: EventWriter<neuralbev_core::entity::SpawnBlueprint>) {
    // Камера
    commands.spawn(Camera2d);

    // Спаун тестовой сущности через Blueprint
    ev_spawn.send(
        neuralbev_core::entity::SpawnBlueprint::new("DemoPlayer")
            .tag("player")
            .health(neuralbev_core::components::Health::new(100.0))
            .movement(neuralbev_core::components::Movement::with_max_speed(200.0))
            .at(Transform::from_xyz(0.0, 0.0, 0.0)),
    );

    info!("[Runtime] Демо-сцена запущена. Нажми Esc для выхода.");
}
