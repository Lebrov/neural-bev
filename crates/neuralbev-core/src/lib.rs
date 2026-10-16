//! # neuralbev-core
//!
//! **AI-HINT: Это главный entry point движка. Читай отсюда.**
//!
//! `neuralbev-core` — ECS-ядро поверх Bevy, оптимизированное для работы
//! с AI-агентами (вайбкодинг). Реализует предсказуемую архитектуру:
//!
//! ```text
//! ┌──────────────────────────────────────────────────────┐
//! │                    neuralbev-core                    │
//! │                                                      │
//! │  components/   ← базовые компоненты (Health, ...)   │
//! │  entity/       ← Blueprint-система (spawn из YAML)  │
//! │  events/       ← типизированные события              │
//! │  systems/      ← системы с явными зависимостями      │
//! │  world/        ← helpers для World                   │
//! │  scene/        ← сериализация / десериализация сцен  │
//! └──────────────────────────────────────────────────────┘
//! ```
//!
//! ## Быстрый старт
//!
//! ```rust,no_run
//! use bevy::prelude::*;
//! use neuralbev_core::NeuralBevCorePlugin;
//!
//! fn main() {
//!     App::new()
//!         .add_plugins(DefaultPlugins)
//!         .add_plugins(NeuralBevCorePlugin)
//!         .run();
//! }
//! ```
//!
//! ## Правила для AI-агентов
//! 1. **Создание сущностей** — через [`entity::Blueprint`] или [`entity::BlueprintCommands`].
//! 2. **Коммуникация** — через события в [`events`], не через прямые Query.
//! 3. **Компоненты** — объявляй через макрос `#[neuralbev_component]`.
//! 4. **Сцены** — загружай/сохраняй через [`scene::SceneSerializer`].

#![warn(missing_docs)]
#![allow(clippy::type_complexity)] // Bevy query tuples бывают большими

pub mod components;
pub mod entity;
pub mod events;
pub mod scene;
pub mod systems;
pub mod world;

// Реэкспорт макроса для удобства: `use neuralbev_core::neuralbev_component`
pub use neuralbev_macros::neuralbev_component;

use bevy::prelude::*;

// ===== PLUGIN: NeuralBevCorePlugin =====
// PURPOSE: Регистрирует все core-системы, компоненты и события движка.
// SYSTEMS: scene_load_system, blueprint_spawn_system, invincibility_tick_system
// COMPONENTS: Health, Movement, Tags, ScriptRef, AiBehavior
// EVENTS: EntitySpawned, EntityDespawned, HealthChanged, EntityDied,
//         SceneLoaded, SceneUnloaded, CollisionStarted, ActionPressed
// ========================================

/// WHAT: Главный плагин `neuralbev-core`. Подключает все подсистемы ядра.  
/// WHY:  Единая точка входа — AI-агенту достаточно добавить один плагин.  
/// HOW:  ```rust
///       app.add_plugins(NeuralBevCorePlugin);
///       ```
/// AI-HINT: Добавляй **до** своих плагинов — они могут зависеть от компонентов
///           и событий, зарегистрированных здесь.
pub struct NeuralBevCorePlugin;

impl Plugin for NeuralBevCorePlugin {
    fn build(&self, app: &mut App) {
        // ── Регистрация типов в Reflect (нужно для сериализации и Inspector) ──
        components::register_all_components(app);

        // ── События ──────────────────────────────────────────────────────────
        events::register_all_events(app);

        // ── События Blueprint ─────────────────────────────────────────────────
        app.add_event::<entity::SpawnBlueprint>();

        // ── Ресурсы ──────────────────────────────────────────────────────────
        app.init_resource::<entity::BlueprintRegistry>();
        app.init_resource::<scene::SceneSerializer>();
        app.init_resource::<world::WorldInfo>();

        // ── Системы (стадии явно указаны) ────────────────────────────────────
        app.add_systems(
            Update,
            (
                systems::health::invincibility_tick_system,
                systems::health::health_change_apply_system,
                // death_detection идёт ПОСЛЕ apply — чтобы HP уже было обновлено
                systems::health::death_detection_system
                    .after(systems::health::health_change_apply_system),
                systems::movement::movement_apply_system,
                systems::ai_behavior::patrol_system,
            ),
        );

        app.add_systems(
            PreUpdate,
            (
                entity::blueprint_spawn_system,
                scene::process_load_scene_requests,
            ),
        );
    }
}
