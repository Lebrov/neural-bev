//! # systems::ai_behavior
//!
//! Системы AI-поведения сущностей.
//!
//! ## Системы
//! - [`patrol_system`] — патрульное движение между точками
//!
//! ## AI-HINT
//! AI-системы пишут в `Movement.velocity`, не в `Transform`.
//! Это позволяет им корректно взаимодействовать с физикой и трением.

use bevy::prelude::*;

use crate::components::{AiBehavior, AiBehaviorType, Movement, Tags};

// Расстояние, при достижении которого цель считается достигнутой.
/// Порог «достигли точки патруля», в единицах мира.
const PATROL_ARRIVAL_DISTANCE: f32 = 5.0;

// ──────────────────────────────────────────────────────────────────────────────

/// WHAT: Двигает сущности с поведением `Patrol` между заданными точками.  
/// WHY:  Простое патрульное поведение без скриптов.  
/// HOW:  Добавь компонент [`AiBehavior::patrol`] к сущности с [`Movement`].  
/// AI-HINT: Для сложного патруля (охрана, обнаружение игрока) — используй ScriptRef + Lua.
pub fn patrol_system(mut query: Query<(&mut AiBehavior, &mut Movement, &Transform)>) {
    for (mut ai, mut movement, transform) in query.iter_mut() {
        let AiBehaviorType::Patrol {
            ref points,
            ref mut current_target_idx,
        } = ai.behavior_type
        else {
            continue;
        };

        if points.is_empty() {
            continue;
        }

        let target_2d = points[*current_target_idx];
        let target_3d = Vec3::new(target_2d.x, target_2d.y, transform.translation.z);
        let to_target = target_3d - transform.translation;
        let distance = to_target.length();

        if distance < PATROL_ARRIVAL_DISTANCE {
            // Переходим к следующей точке
            *current_target_idx = (*current_target_idx + 1) % points.len();
        } else {
            // Двигаемся к текущей цели
            movement.velocity = (to_target / distance) * ai.speed;
        }
    }
}
