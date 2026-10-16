//! # systems::health
//!
//! Системы управления здоровьем.
//!
//! ## Системы
//! - [`invincibility_tick_system`] — уменьшает таймер иммунитета каждый кадр
//! - [`death_detection_system`] — обнаруживает смерть и посылает событие EntityDied
//!
//! ## AI-HINT
//! Не применяй урон/лечение напрямую в своих системах.
//! Посылай событие [`crate::events::gameplay::HealthChanged`] и читай
//! результат через [`crate::events::gameplay::EntityDied`].

use bevy::prelude::*;

use crate::components::Health;
use crate::events::gameplay::{DamageSource, EntityDied, HealthChanged};

// ──────────────────────────────────────────────────────────────────────────────

/// WHAT: Уменьшает `invincibility_timer` у всех сущностей с Health.  
/// WHY:  Иммунитет после получения урона работает по таймеру.  
/// HOW:  Система автоматически добавлена плагином, не вызывай вручную.  
/// AI-HINT: Если хочешь выдать иммунитет — установи `health.invincibility_timer`.
///           Эта система сама уменьшит его до 0.
pub fn invincibility_tick_system(time: Res<Time>, mut query: Query<&mut Health>) {
    let dt = time.delta_secs();
    for mut health in query.iter_mut() {
        if health.invincibility_timer > 0.0 {
            health.invincibility_timer = (health.invincibility_timer - dt).max(0.0);
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────────

/// WHAT: Обнаруживает смерть (Health.current <= 0) и посылает событие EntityDied.  
/// WHY:  Отделяет обнаружение смерти от реакции — несколько систем могут
///       реагировать на EntityDied независимо.  
/// HOW:  Система автоматически добавлена плагином.  
/// AI-HINT: Не удаляй сущность здесь. Слушай EntityDied в своей системе и там despawn.
pub fn death_detection_system(
    mut ev_health: EventReader<HealthChanged>,
    query: Query<&Health>,
    mut ev_died: EventWriter<EntityDied>,
) {
    for ev in ev_health.read() {
        // Проверяем, жива ли сущность после изменения HP
        if let Ok(health) = query.get(ev.entity) {
            if !health.is_alive() {
                ev_died.send(EntityDied {
                    entity: ev.entity,
                    killed_by: ev.source.clone(),
                });
                log::info!(
                    "[Health] Сущность {:?} погибла (killed_by: {:?})",
                    ev.entity,
                    ev.source
                );
            }
        }
    }
}

/// WHAT: Применяет события HealthChanged к компонентам Health.  
/// WHY:  Единая точка применения урона/лечения — гарантирует правильный порядок.  
/// HOW:  Система автоматически добавлена плагином.  
/// AI-HINT: Не пиши `health.current -= damage` в своих системах.
///           Посылай `HealthChanged` и позволь этой системе обработать его.
pub fn health_change_apply_system(
    mut ev_health: EventReader<HealthChanged>,
    mut query: Query<&mut Health>,
) {
    for ev in ev_health.read() {
        if let Ok(mut health) = query.get_mut(ev.entity) {
            health.current = ev.new_value.clamp(0.0, health.max_health);
        }
    }
}
