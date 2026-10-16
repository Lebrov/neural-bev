//! # events::gameplay
//!
//! Игровые события: урон, смерть, …
//!
//! ## AI-HINT
//! Не изменяй `Health.current` напрямую.
//! Посылай `HealthChanged` — это сохраняет аудит и позволяет редактору делать Undo.

use bevy::prelude::*;

// ──────────────────────────────────────────────────────────────────────────────

/// WHAT: Событие изменения здоровья сущности.  
/// WHY:  Единая точка, через которую проходит весь урон/лечение.
///       Систематизирует логику HealthBar, звуков урона, игрового UI.  
/// HOW:  ```rust
///       // Нанести 10 урона:
///       ev_health.send(HealthChanged {
///           entity,
///           delta: -10.0,
///           new_value: health.current - 10.0,
///           source: DamageSource::Enemy,
///       });
///       ```
/// AI-HINT: `delta` < 0 — урон, `delta` > 0 — лечение.
///           Система health::damage_apply_system слушает это событие и применяет изменения.
#[derive(Event, Debug, Clone)]
pub struct HealthChanged {
    /// Сущность, чьё здоровье изменилось.
    pub entity: Entity,
    /// Изменение: отрицательное = урон, положительное = лечение.
    pub delta: f32,
    /// Новое значение здоровья после изменения.
    pub new_value: f32,
    /// Источник изменения (для аналитики и эффектов).
    pub source: DamageSource,
}

/// Источник урона или лечения.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DamageSource {
    /// Урон от другой сущности.
    Entity(Entity),
    /// Урон от окружения (лава, шипы, …).
    Environment,
    /// Лечение (зелье, абилка, …).
    Healing,
    /// Изменение из редактора/скрипта.
    Script,
}

// ──────────────────────────────────────────────────────────────────────────────

/// WHAT: Событие смерти сущности (Health.current <= 0).  
/// WHY:  Отделяет обнаружение смерти от реакции на неё (анимация, спаун лута, …).  
/// HOW:  ```rust
///       fn on_death(mut ev: EventReader<EntityDied>, mut commands: Commands) {
///           for ev in ev.read() {
///               commands.entity(ev.entity).despawn_recursive();
///           }
///       }
///       ```
/// AI-HINT: Система `death_detection_system` автоматически посылает это событие.
///           Тебе только нужно на него реагировать.
#[derive(Event, Debug, Clone)]
pub struct EntityDied {
    /// Умершая сущность.
    pub entity: Entity,
    /// Последний источник урона.
    pub killed_by: DamageSource,
}
