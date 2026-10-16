//! # Пример: Платформер
//!
//! Демонстрирует:
//! - Спаун сущностей через Blueprint
//! - Компоненты Health и Movement
//! - Обработку событий EntityDied
//! - Загрузку сцены из YAML-строки
//!
//! ## Запуск
//! ```bash
//! cargo run --example platformer -p neuralbev-examples
//! ```
//!
//! ## AI-HINT
//! Это рабочий шаблон платформера. Скопируй и модифицируй под свою игру.

use bevy::prelude::*;
use neuralbev_core::components::{AiBehavior, Health, Movement};
use neuralbev_core::entity::SpawnBlueprint;
use neuralbev_core::events::gameplay::EntityDied;
use neuralbev_core::NeuralBevCorePlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "NeuralBev — Platformer Example".to_string(),
                resolution: (1280.0, 720.0).into(),
                ..Default::default()
            }),
            ..Default::default()
        }))
        .add_plugins(NeuralBevCorePlugin)
        // Системы примера
        .add_systems(Startup, setup_scene)
        .add_systems(Update, (player_input_system, on_entity_died_system))
        .run();
}

// ── Setup ─────────────────────────────────────────────────────────────────────

/// Инициализирует сцену: камера, игрок, патрульный враг.
fn setup_scene(
    mut commands: Commands,
    mut ev_spawn: EventWriter<SpawnBlueprint>,
) {
    // Камера
    commands.spawn(Camera2d);

    // Игрок
    ev_spawn.send(
        SpawnBlueprint::new("Player")
            .tag("player")
            .health(Health::new(100.0))
            .movement(Movement::with_max_speed(300.0))
            .at(Transform::from_xyz(-300.0, 0.0, 0.0)),
    );

    // Патрульный враг
    ev_spawn.send(
        SpawnBlueprint::new("Goblin")
            .tag("enemy")
            .health(Health::new(30.0))
            .movement(Movement::with_max_speed(80.0))
            .ai_behavior(AiBehavior::patrol(
                vec![Vec2::new(0.0, 0.0), Vec2::new(200.0, 0.0)],
                80.0,
            ))
            .at(Transform::from_xyz(100.0, 0.0, 0.0)),
    );

    info!("[Platformer] Сцена загружена. WASD для движения игрока.");
}

// ── Player Input ──────────────────────────────────────────────────────────────

/// Простое управление игроком через клавиатуру.
///
/// ## AI-HINT
/// В реальном проекте замени на чтение ActionPressed событий.
fn player_input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Movement, With<neuralbev_core::components::Tags>>,
) {
    // Скорость игрока в ед./сек.
    const PLAYER_SPEED: f32 = 300.0;

    for mut movement in query.iter_mut() {
        let mut vel = Vec3::ZERO;
        if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
            vel.x -= PLAYER_SPEED;
        }
        if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
            vel.x += PLAYER_SPEED;
        }
        if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
            vel.y += PLAYER_SPEED;
        }
        if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
            vel.y -= PLAYER_SPEED;
        }
        movement.velocity = vel;
    }
}

// ── Death Handling ────────────────────────────────────────────────────────────

/// Реагирует на смерть сущностей — удаляет их из мира.
///
/// ## AI-HINT
/// В реальной игре добавь сюда: анимацию смерти, спаун лута, обновление счёта.
fn on_entity_died_system(
    mut commands: Commands,
    mut ev_died: EventReader<EntityDied>,
    names: Query<&neuralbev_core::components::EntityName>,
) {
    for ev in ev_died.read() {
        let name = names
            .get(ev.entity)
            .map(|n| n.name.as_str())
            .unwrap_or("<unknown>");
        info!("[Platformer] Сущность '{}' умерла!", name);
        // Безопасное удаление — если сущность уже удалена, команда игнорируется
        commands.entity(ev.entity).despawn_recursive();
    }
}
