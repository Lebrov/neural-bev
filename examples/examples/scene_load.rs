//! # Пример: Загрузка сцены из YAML
//!
//! Демонстрирует полный цикл: YAML → [`SpawnBlueprint`] → Bevy entities.
//!
//! ## Запуск
//! ```bash
//! cargo run --example scene_load -p neuralbev-examples
//! ```

use bevy::prelude::*;
use neuralbev_core::scene::{LoadSceneRequest, SceneSerializer};
use neuralbev_core::NeuralBevCorePlugin;

/// YAML-сцена, встроенная прямо в код для демо.
const DEMO_SCENE: &str = r#"
meta:
  engine_version: "0.1.0"
  scene_name: "DemoLevel"
entities:
  - id: "player"
    name: "Player"
    tags:
      - player
      - controllable
    components:
      Transform:
        translation: [0.0, 0.0, 0.0]
        scale: [1.0, 1.0, 1.0]
      Health:
        current: 100.0
        max_health: 100.0
        invincibility_timer: 0.0
      Script:
        path: scripts/player_controller.lua

  - id: "enemy_1"
    name: "Goblin"
    tags:
      - enemy
    components:
      Transform:
        translation: [200.0, 0.0, 0.0]
        scale: [1.0, 1.0, 1.0]
      Health:
        current: 30.0
        max_health: 30.0
        invincibility_timer: 0.0
      AiBehavior:
        type: Patrol
        patrol_points: [[100.0, 0.0], [300.0, 0.0]]
        speed: 80.0
"#;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "NeuralBev — Scene Load Example".to_string(),
                resolution: (1280.0, 720.0).into(),
                ..Default::default()
            }),
            ..Default::default()
        }))
        .add_plugins(NeuralBevCorePlugin)
        .add_systems(Startup, load_demo_scene)
        .add_systems(
            Update,
            neuralbev_core::scene::process_load_scene_requests,
        )
        .run();
}

/// Запрашивает загрузку сцены через LoadSceneRequest.
fn load_demo_scene(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn(LoadSceneRequest {
        yaml: DEMO_SCENE.to_string(),
    });
    info!("[SceneLoad] Запрос на загрузку DemoLevel отправлен.");
}
