//! # scene
//!
//! Сериализация и десериализация сцен в формат `.neuralbev.yaml`.
//!
//! ## Формат файла
//! ```yaml
//! meta:
//!   engine_version: "0.1.0"
//!   scene_name: "MainLevel"
//! entities:
//!   - id: "player"
//!     name: "Player"
//!     tags: ["player"]
//!     components:
//!       Transform:
//!         translation: [0.0, 0.0, 0.0]
//!       Health:
//!         current: 100.0
//!         max_health: 100.0
//! ```
//!
//! ## AI-HINT
//! - Для сохранения сцены используй метод `SceneSerializer::save_to_string`.
//! - Для загрузки сцены используй `SceneSerializer::load_from_str` — получишь `Vec<SpawnBlueprint>`.
//! - Не пиши парсинг вручную — всё здесь.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::entity::{BlueprintData, BlueprintError, SpawnBlueprint};
use crate::events::world::SceneLoaded;

// Версия движка — фиксируется в файлах сцен для контроля совместимости.
/// Текущая версия формата сцены.
pub const SCENE_FORMAT_VERSION: &str = "0.1.0";

// ──────────────────────────────────────────────────────────────────────────────
// SceneFile — корень YAML-файла
// ──────────────────────────────────────────────────────────────────────────────

/// WHAT: Корневая структура файла сцены `.neuralbev.yaml`.  
/// WHY:  Обёртка для десериализации всего файла целиком.  
/// AI-HINT: Используй через [`SceneSerializer`], не напрямую.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneFile {
    /// Метаданные сцены.
    pub meta: SceneMeta,
    /// Список сущностей верхнего уровня.
    pub entities: Vec<BlueprintData>,
}

/// Метаданные сцены.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneMeta {
    /// Версия движка, которым создан файл.
    pub engine_version: String,
    /// Читаемое имя сцены.
    pub scene_name: String,
}

// ──────────────────────────────────────────────────────────────────────────────
// SceneSerializer — ресурс для работы со сценами
// ──────────────────────────────────────────────────────────────────────────────

/// WHAT: Ресурс для сохранения и загрузки сцен.  
/// WHY:  Центральная точка I/O сцены — кешируем имя текущей сцены,
///       обеспечиваем единый API.  
/// HOW:  ```rust
///       fn save_scene(serializer: Res<SceneSerializer>, query: Query<...>) {
///           let yaml = serializer.save_to_string("MainLevel", &entities_data)
///               .expect("serialization ok");
///           std::fs::write("scene.neuralbev.yaml", yaml).expect("write ok");
///       }
///       ```
/// AI-HINT: Ресурс инициализируется автоматически плагином. Не создавай вручную.
#[derive(Resource, Default)]
pub struct SceneSerializer {
    /// Имя текущей загруженной сцены.
    pub current_scene_name: Option<String>,
}

impl SceneSerializer {
    /// Десериализует сцену из YAML-строки.
    ///
    /// Возвращает список [`SpawnBlueprint`] для каждой сущности сцены.
    ///
    /// # Errors
    /// Возвращает [`SceneError`] при невалидном YAML или несовместимой версии.
    pub fn load_from_str(&self, yaml: &str) -> Result<LoadedScene, SceneError> {
        let file: SceneFile = serde_yaml::from_str(yaml).map_err(SceneError::Yaml)?;

        // Проверка совместимости версий (предупреждение, не фатально)
        if file.meta.engine_version != SCENE_FORMAT_VERSION {
            log::warn!(
                "[Scene] Версия файла '{}' отличается от текущей '{}'. Возможна несовместимость.",
                file.meta.engine_version,
                SCENE_FORMAT_VERSION
            );
        }

        // Конвертируем каждую BlueprintData в SpawnBlueprint
        let blueprints: Result<Vec<_>, _> = file
            .entities
            .iter()
            .map(SpawnBlueprint::from_blueprint_data)
            .collect();
        let blueprints = blueprints.map_err(SceneError::Blueprint)?;

        Ok(LoadedScene {
            name: file.meta.scene_name,
            blueprints,
        })
    }

    /// Сериализует набор Blueprint'ов обратно в YAML-строку.
    ///
    /// `scene_name` — имя сцены, записывается в `meta.scene_name`.
    ///
    /// # Errors
    /// Возвращает [`SceneError`] при сбое сериализации.
    pub fn save_to_string(
        &self,
        scene_name: &str,
        entities: &[BlueprintData],
    ) -> Result<String, SceneError> {
        let file = SceneFile {
            meta: SceneMeta {
                engine_version: SCENE_FORMAT_VERSION.to_string(),
                scene_name: scene_name.to_string(),
            },
            entities: entities.to_vec(),
        };
        serde_yaml::to_string(&file).map_err(SceneError::Yaml)
    }
}

/// Результат успешной загрузки сцены.
#[derive(Debug)]
pub struct LoadedScene {
    /// Имя сцены из файла.
    pub name: String,
    /// Blueprint'ы для спауна.
    pub blueprints: Vec<SpawnBlueprint>,
}

// ──────────────────────────────────────────────────────────────────────────────
// SceneError
// ──────────────────────────────────────────────────────────────────────────────

/// WHAT: Ошибки при работе со сценами.
#[derive(Debug, thiserror::Error)]
pub enum SceneError {
    /// Ошибка парсинга YAML.
    #[error("Ошибка YAML: {0}")]
    Yaml(#[from] serde_yaml::Error),
    /// Ошибка Blueprint.
    #[error("Ошибка Blueprint: {0}")]
    Blueprint(#[from] BlueprintError),
    /// Ошибка I/O.
    #[error("Ошибка чтения файла: {0}")]
    Io(#[from] std::io::Error),
}

// ──────────────────────────────────────────────────────────────────────────────
// scene_load_system — загрузка сцены из AssetEvent
// ──────────────────────────────────────────────────────────────────────────────

/// WHAT: Компонент-маркер для запроса загрузки сцены.  
/// WHY:  Позволяет запрашивать загрузку сцены из любой системы через spawn.  
/// HOW:  ```rust
///       commands.spawn(LoadSceneRequest {
///           yaml: include_str!("../assets/scenes/main.neuralbev.yaml").to_string(),
///       });
///       ```
/// AI-HINT: Компонент обрабатывается системой `process_load_scene_requests`.
///           После обработки сущность-запрос уничтожается автоматически.
#[derive(Component)]
pub struct LoadSceneRequest {
    /// YAML-содержимое файла сцены.
    pub yaml: String,
}

/// WHAT: Обрабатывает [`LoadSceneRequest`]-сущности и выполняет спаун Blueprint'ов.  
/// WHY:  Разделяет запрос на загрузку и его исполнение.  
/// HOW:  Добавляется автоматически в [`crate::NeuralBevCorePlugin`].  
/// AI-HINT: Не вызывай напрямую. Создавай [`LoadSceneRequest`]-сущности.
pub fn process_load_scene_requests(
    mut commands: Commands,
    requests: Query<(Entity, &LoadSceneRequest)>,
    serializer: Res<SceneSerializer>,
    mut ev_spawn: EventWriter<SpawnBlueprint>,
    mut ev_loaded: EventWriter<SceneLoaded>,
) {
    for (req_entity, req) in requests.iter() {
        match serializer.load_from_str(&req.yaml) {
            Ok(loaded) => {
                log::info!("[Scene] Загрузка сцены '{}'", loaded.name);
                for bp in loaded.blueprints {
                    ev_spawn.send(bp);
                }
                ev_loaded.send(SceneLoaded {
                    scene_name: loaded.name.clone(),
                    path: String::new(), // путь известен вызывающей стороне
                });
                log::info!("[Scene] Сцена '{}' успешно загружена", loaded.name);
            }
            Err(e) => {
                log::error!("[Scene] Ошибка загрузки сцены: {e}");
            }
        }
        // Удаляем запрос после обработки
        commands.entity(req_entity).despawn();
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Тесты
// ──────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const SCENE_YAML: &str = r#"
meta:
  engine_version: "0.1.0"
  scene_name: "TestLevel"
entities:
  - id: "player"
    name: "Player"
    tags:
      - player
    components:
      Transform:
        translation: [0.0, 0.0, 0.0]
        scale: [1.0, 1.0, 1.0]
      Health:
        current: 100.0
        max_health: 100.0
        invincibility_timer: 0.0
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
"#;

    #[test]
    fn scene_load_returns_correct_count() {
        let s = SceneSerializer::default();
        let loaded = s.load_from_str(SCENE_YAML).expect("парсинг должен пройти");
        assert_eq!(loaded.blueprints.len(), 2);
    }

    #[test]
    fn scene_load_name() {
        let s = SceneSerializer::default();
        let loaded = s.load_from_str(SCENE_YAML).expect("парсинг должен пройти");
        assert_eq!(loaded.name, "TestLevel");
    }

    #[test]
    fn scene_load_first_entity_health() {
        let s = SceneSerializer::default();
        let loaded = s.load_from_str(SCENE_YAML).expect("парсинг должен пройти");
        let player = &loaded.blueprints[0];
        assert_eq!(player.name, "Player");
        assert_eq!(player.health.as_ref().unwrap().max_health, 100.0);
    }

    #[test]
    fn scene_load_second_entity() {
        let s = SceneSerializer::default();
        let loaded = s.load_from_str(SCENE_YAML).expect("парсинг должен пройти");
        let goblin = &loaded.blueprints[1];
        assert_eq!(goblin.name, "Goblin");
        assert!(goblin.tags.contains(&"enemy".to_string()));
        assert_eq!(goblin.health.as_ref().unwrap().max_health, 30.0);
    }

    #[test]
    fn scene_save_roundtrip() {
        let s = SceneSerializer::default();
        let original = s.load_from_str(SCENE_YAML).expect("load");
        // Конвертируем обратно в BlueprintData для сохранения
        let data: Vec<BlueprintData> = original
            .blueprints
            .iter()
            .map(|bp| BlueprintData {
                id: bp.name.to_lowercase(),
                name: bp.name.clone(),
                tags: bp.tags.clone(),
                components: HashMap::new(), // упрощено для теста
                children: vec![],
            })
            .collect();
        let yaml = s.save_to_string("TestLevel", &data).expect("save");
        assert!(yaml.contains("TestLevel"));
        assert!(yaml.contains("Player"));
        assert!(yaml.contains("Goblin"));
    }

    #[test]
    fn scene_version_mismatch_does_not_panic() {
        let mismatched = r#"
meta:
  engine_version: "99.0.0"
  scene_name: "Old"
entities: []
"#;
        let s = SceneSerializer::default();
        // Предупреждение выводится, но паники нет
        let loaded = s.load_from_str(mismatched).expect("не должно падать при несовместимости версий");
        assert_eq!(loaded.blueprints.len(), 0);
    }
}
