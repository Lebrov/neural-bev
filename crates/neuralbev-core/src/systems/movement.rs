//! # systems::movement
//!
//! Система применения скорости к Transform.
//!
//! ## AI-HINT
//! Пиши желаемую скорость в `Movement.velocity`.
//! Эта система применяет её к `Transform.translation` с учётом `delta_time`.

use bevy::prelude::*;

use crate::components::Movement;

// Минимальная скорость для применения трения (избегаем дрожания при ~0)
/// Порог скорости, ниже которого трение обнуляет скорость.
const VELOCITY_STOP_THRESHOLD: f32 = 0.5;

// ──────────────────────────────────────────────────────────────────────────────

/// WHAT: Применяет `Movement.velocity` к `Transform.translation`.  
/// WHY:  Единая точка движения — все источники скорости (игрок, AI, скрипты)
///       пишут в `velocity`, эта система применяет результат.  
/// HOW:  Система автоматически добавлена плагином.  
/// AI-HINT: Не двигай Transform вручную. Пиши в `Movement.velocity`.
pub fn movement_apply_system(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut Movement)>,
) {
    let dt = time.delta_secs();

    for (mut transform, mut movement) in query.iter_mut() {
        // Ограничиваем скорость
        if movement.max_speed > 0.0 {
            let speed = movement.velocity.length();
            if speed > movement.max_speed {
                movement.velocity = movement.velocity / speed * movement.max_speed;
            }
        }

        // Применяем перемещение
        transform.translation += movement.velocity * dt;

        // Трение: замедляем скорость
        if movement.friction < 1.0 {
            movement.velocity *= movement.friction;
            // Обнуляем очень маленькую скорость чтобы не дрожать
            if movement.velocity.length() < VELOCITY_STOP_THRESHOLD {
                movement.velocity = Vec3::ZERO;
            }
        }
    }
}
