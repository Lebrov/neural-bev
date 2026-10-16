//! # entity
//!
//! Blueprint-система — декларативный способ создания сущностей.
//!
//! ## AI-HINT
//! Используй [`SpawnBlueprint`] вместо прямого `commands.spawn(...)`.
//! Blueprint можно:
//! - Создать из кода: `SpawnBlueprint::new("Player").with(Health::new(100.0))`
//! - Загрузить из YAML: `SpawnBlueprint::from_file("entities/goblin.yaml")`
//! - Отправить как Command в очередь: `ev_spawn.send(spawn_bp)`

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::components::{AiBehavior, EntityName, Health, Movement, ScriptRef, Tags};
use crate::events::world::{EntityDespawned, EntitySpawned};

// ──────────────────────────────────────────────────────────────────────────────
// BlueprintData — данные для десериализации из YAML/JSON
// ──────────────────────────────────────────────────────────────────────────────

/// WHAT: Сырые данные Blueprint'а — десериализуются из YAML/JSON файлов сцены.  
/// WHY:  Отделяет данные (что спаунить) от Bevy-команды (как спаунить).  
/// HOW:  Обычно не создаётся вручную — используй [`SpawnBlueprint`].  
/// AI-HINT: Этот тип — промежуточный. Работай с [`SpawnBlueprint`].
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BlueprintData {
    /// Уникальный строковый ID сущности в сцене (например `"player"`, `"enemy_1"`).
    pub id: String,
    /// Человекочитаемое имя (попадёт в компонент [`EntityName`]).
    pub name: String,
    /// Теги сущности (попадут в компонент [`Tags`]).
    #[serde(default)]
    pub tags: Vec<String>,
    /// Словарь компонентов: имя → JSON-значение.
    #[serde(default)]
    pub components: HashMap<String, serde_json::Value>,
    /// Дочерние Blueprint'ы (для иерархии parent/child).
    #[serde(default)]
    pub children: Vec<BlueprintData>,
}

// ──────────────────────────────────────────────────────────────────────────────
// SpawnBlueprint — Command для спауна
// ──────────────────────────────────────────────────────────────────────────────

/// WHAT: Высокоуровневый строитель для создания сущностей.  
/// WHY:  Единая точка создания сущностей — поддерживает кодовый и YAML пути.  
/// HOW:  ```rust
///       // Через код:
///       ev_spawn.send(
///           SpawnBlueprint::new("Player")
///               .tag("player")
///               .tag("controllable")
///               .health(Health::new(100.0))
///               .at(Transform::from_xyz(0.0, 0.0, 0.0))
///       );
///
///       // Из YAML-файла:
///       ev_spawn.send(SpawnBlueprint::from_yaml_str(yaml_str).expect("valid yaml"));
///       ```
/// AI-HINT: Предпочитай builder-методы (`.health()`, `.at()`, …) прямой работе
///           с полями — это устойчиво к будущим изменениям API.
#[derive(Event, Debug, Clone)]
pub struct SpawnBlueprint {
    /// Имя создаваемой сущности.
    pub name: String,
    /// Начальная трансформация.
    pub transform: Transform,
    /// Компонент здоровья (опционально).
    pub health: Option<Health>,
    /// Компонент движения (опционально).
    pub movement: Option<Movement>,
    /// Теги.
    pub tags: Vec<String>,
    /// Ссылка на скрипт (опционально).
    pub script: Option<ScriptRef>,
    /// AI-поведение (опционально).
    pub ai_behavior: Option<AiBehavior>,
}

impl Default for SpawnBlueprint {
    fn default() -> Self {
        Self {
            name: String::from("Entity"),
            transform: Transform::default(),
            health: None,
            movement: None,
            tags: Vec::new(),
            script: None,
            ai_behavior: None,
        }
    }
}

impl SpawnBlueprint {
    /// Создаёт новый Blueprint с именем.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    /// Добавляет тег.
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Устанавливает начальную позицию/ориентацию.
    pub fn at(mut self, transform: Transform) -> Self {
        self.transform = transform;
        self
    }

    /// Добавляет компонент здоровья.
    pub fn health(mut self, health: Health) -> Self {
        self.health = Some(health);
        self
    }

    /// Добавляет компонент движения.
    pub fn movement(mut self, movement: Movement) -> Self {
        self.movement = Some(movement);
        self
    }

    /// Добавляет ссылку на скрипт.
    pub fn script(mut self, script: ScriptRef) -> Self {
        self.script = Some(script);
        self
    }

    /// Добавляет AI-поведение.
    pub fn ai_behavior(mut self, ai: AiBehavior) -> Self {
        self.ai_behavior = Some(ai);
        self
    }

    /// Десериализует Blueprint из YAML-строки (формат файла сцены).
    ///
    /// # Errors
    /// Возвращает ошибку при невалидном YAML или несовместимой структуре.
    pub fn from_yaml_str(yaml: &str) -> Result<Self, BlueprintError> {
        let data: BlueprintData =
            serde_yaml::from_str(yaml).map_err(BlueprintError::DeserializationError)?;
        Self::from_blueprint_data(&data)
    }

    /// Конвертирует сырые [`BlueprintData`] в [`SpawnBlueprint`].
    pub fn from_blueprint_data(data: &BlueprintData) -> Result<Self, BlueprintError> {
        let mut bp = Self::new(data.name.clone());

        for tag in &data.tags {
            bp = bp.tag(tag.clone());
        }

        // Парсим Transform
        if let Some(t) = data.components.get("Transform") {
            let translation = parse_vec3(t, "translation")?;
            let scale = parse_vec3(t, "scale").unwrap_or(Vec3::ONE);
            bp.transform = Transform {
                translation,
                scale,
                ..Default::default()
            };
        }

        // Парсим Health
        if let Some(h) = data.components.get("Health") {
            let current = h["current"]
                .as_f64()
                .ok_or(BlueprintError::MissingField("Health.current"))? as f32;
            let max_health = h["max_health"]
                .as_f64()
                .ok_or(BlueprintError::MissingField("Health.max_health"))? as f32;
            bp.health = Some(Health {
                current,
                max_health,
                invincibility_timer: h["invincibility_timer"].as_f64().unwrap_or(0.0) as f32,
            });
        }

        // Парсим ScriptRef
        if let Some(s) = data.components.get("Script") {
            let path = s["path"]
                .as_str()
                .ok_or(BlueprintError::MissingField("Script.path"))?
                .to_string();
            bp.script = Some(ScriptRef::lua(path));
        }

        // Парсим AiBehavior
        if let Some(ai) = data.components.get("AiBehavior") {
            let speed = ai["speed"].as_f64().unwrap_or(80.0) as f32;
            if ai["type"].as_str() == Some("Patrol") {
                if let Some(pts) = ai["patrol_points"].as_array() {
                    let points: Vec<Vec2> = pts
                        .iter()
                        .filter_map(|p| {
                            let arr = p.as_array()?;
                            Some(Vec2::new(
                                arr.first()?.as_f64()? as f32,
                                arr.get(1)?.as_f64()? as f32,
                            ))
                        })
                        .collect();
                    bp.ai_behavior = Some(AiBehavior::patrol(points, speed));
                }
            }
        }

        Ok(bp)
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// BlueprintError
// ──────────────────────────────────────────────────────────────────────────────

/// WHAT: Ошибки Blueprint-системы.  
/// WHY:  Структурированные ошибки позволяют AI/редактору вывести полезное сообщение.
#[derive(Debug, thiserror::Error)]
pub enum BlueprintError {
    /// Ошибка десериализации YAML.
    #[error("Ошибка парсинга Blueprint YAML: {0}")]
    DeserializationError(#[from] serde_yaml::Error),
    /// Отсутствует обязательное поле.
    #[error("Blueprint: отсутствует поле `{0}`")]
    MissingField(&'static str),
    /// Ошибка чтения файла.
    #[error("Ошибка чтения файла Blueprint `{path}`: {source}")]
    IoError {
        /// Путь к файлу.
        path: String,
        /// Системная ошибка.
        source: std::io::Error,
    },
}

// ──────────────────────────────────────────────────────────────────────────────
// BlueprintRegistry — реестр именованных Blueprint'ов
// ──────────────────────────────────────────────────────────────────────────────

/// WHAT: Реестр предзагруженных Blueprint'ов по строковому ключу.  
/// WHY:  Позволяет ссылаться на Blueprint по имени из скриптов и редактора.  
/// HOW:  ```rust
///       registry.register("goblin", SpawnBlueprint::new("Goblin").health(Health::new(30.0)));
///       // Позже:
///       let bp = registry.get("goblin").cloned().unwrap();
///       ev_spawn.send(bp.at(Transform::from_xyz(100.0, 0.0, 0.0)));
///       ```
/// AI-HINT: Регистрируй Blueprint'ы в startup-системах.
#[derive(Resource, Default)]
pub struct BlueprintRegistry {
    blueprints: HashMap<String, SpawnBlueprint>,
}

impl BlueprintRegistry {
    /// Регистрирует Blueprint под именем.
    pub fn register(&mut self, name: impl Into<String>, bp: SpawnBlueprint) {
        self.blueprints.insert(name.into(), bp);
    }

    /// Возвращает Blueprint по имени, если зарегистрирован.
    pub fn get(&self, name: &str) -> Option<&SpawnBlueprint> {
        self.blueprints.get(name)
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// blueprint_spawn_system — система, выполняющая SpawnBlueprint из очереди событий
// ──────────────────────────────────────────────────────────────────────────────

/// WHAT: Читает события [`SpawnBlueprint`] и создаёт сущности в World.  
/// WHY:  Разделяет «намерение создать» (событие) и «создание» (система).  
///       Позволяет откладывать спаун до PreUpdate.  
/// HOW:  Система добавлена автоматически плагином [`crate::NeuralBevCorePlugin`].  
/// AI-HINT: Не вызывай напрямую. Посылай [`SpawnBlueprint`] через EventWriter.
pub fn blueprint_spawn_system(
    mut commands: Commands,
    mut ev_spawn: EventReader<SpawnBlueprint>,
    mut ev_spawned: EventWriter<EntitySpawned>,
) {
    for bp in ev_spawn.read() {
        // Начинаем собирать bundle сущности
        let entity = commands
            .spawn((
                EntityName::new(bp.name.clone()),
                Transform::from(bp.transform),
                GlobalTransform::default(),
                VisibilityBundle::default(),
            ))
            .id();

        // Добавляем опциональные компоненты
        let mut ec = commands.entity(entity);

        if !bp.tags.is_empty() {
            ec.insert(Tags::from_slice(
                &bp.tags.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
            ));
        }

        if let Some(health) = &bp.health {
            ec.insert(health.clone());
        }

        if let Some(movement) = &bp.movement {
            ec.insert(movement.clone());
        }

        if let Some(script) = &bp.script {
            ec.insert(script.clone());
        }

        if let Some(ai) = &bp.ai_behavior {
            ec.insert(ai.clone());
        }

        // Сообщаем миру о создании
        ev_spawned.send(EntitySpawned {
            entity,
            name: bp.name.clone(),
        });

        log::info!("[Blueprint] Создана сущность '{}' ({:?})", bp.name, entity);
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Вспомогательные функции парсинга
// ──────────────────────────────────────────────────────────────────────────────

fn parse_vec3(value: &serde_json::Value, key: &str) -> Result<Vec3, BlueprintError> {
    let arr = value[key]
        .as_array()
        .ok_or(BlueprintError::MissingField(key))?;
    let x = arr.first().and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
    let y = arr.get(1).and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
    let z = arr.get(2).and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
    Ok(Vec3::new(x, y, z))
}

// ──────────────────────────────────────────────────────────────────────────────
// Тесты
// ──────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const SIMPLE_YAML: &str = r#"
id: player
name: Player
tags:
  - player
  - controllable
components:
  Transform:
    translation: [10.0, 20.0, 0.0]
    scale: [1.0, 1.0, 1.0]
  Health:
    current: 100.0
    max_health: 100.0
    invincibility_timer: 0.0
  Script:
    path: scripts/player.lua
"#;

    #[test]
    fn blueprint_from_yaml_name() {
        let bp = SpawnBlueprint::from_yaml_str(SIMPLE_YAML).expect("парсинг должен пройти");
        assert_eq!(bp.name, "Player");
    }

    #[test]
    fn blueprint_from_yaml_tags() {
        let bp = SpawnBlueprint::from_yaml_str(SIMPLE_YAML).expect("парсинг должен пройти");
        assert!(bp.tags.contains(&"player".to_string()));
        assert!(bp.tags.contains(&"controllable".to_string()));
    }

    #[test]
    fn blueprint_from_yaml_transform() {
        let bp = SpawnBlueprint::from_yaml_str(SIMPLE_YAML).expect("парсинг должен пройти");
        assert_eq!(bp.transform.translation, Vec3::new(10.0, 20.0, 0.0));
    }

    #[test]
    fn blueprint_from_yaml_health() {
        let bp = SpawnBlueprint::from_yaml_str(SIMPLE_YAML).expect("парсинг должен пройти");
        let h = bp.health.expect("Health должен присутствовать");
        assert_eq!(h.max_health, 100.0);
    }

    #[test]
    fn blueprint_from_yaml_script() {
        let bp = SpawnBlueprint::from_yaml_str(SIMPLE_YAML).expect("парсинг должен пройти");
        let s = bp.script.expect("Script должен присутствовать");
        assert_eq!(s.path, "scripts/player.lua");
    }

    #[test]
    fn blueprint_builder_chain() {
        let bp = SpawnBlueprint::new("Goblin")
            .tag("enemy")
            .health(Health::new(30.0))
            .at(Transform::from_xyz(100.0, 0.0, 0.0));

        assert_eq!(bp.name, "Goblin");
        assert!(bp.tags.contains(&"enemy".to_string()));
        assert_eq!(bp.transform.translation.x, 100.0);
        assert_eq!(bp.health.unwrap().max_health, 30.0);
    }

    #[test]
    fn blueprint_registry_get() {
        let mut reg = BlueprintRegistry::default();
        reg.register("goblin", SpawnBlueprint::new("Goblin").tag("enemy"));
        assert!(reg.get("goblin").is_some());
        assert!(reg.get("dragon").is_none());
    }
}
