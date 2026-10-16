//! # world
//!
//! Утилиты для работы с игровым миром.
//!
//! ## AI-HINT
//! `WorldInfo` — ресурс с глобальными параметрами мира (гравитация, размер, …).
//! Читай отсюда вместо захардкоженных констант в системах.

use bevy::prelude::*;

// ──────────────────────────────────────────────────────────────────────────────

/// WHAT: Ресурс с глобальными параметрами игрового мира.  
/// WHY:  Единый источник истины о мировых константах.
///       Системы читают параметры отсюда, не из локальных констант.  
/// HOW:  ```rust
///       fn my_system(world_info: Res<WorldInfo>) {
///           let gravity = world_info.gravity;
///       }
///       ```
/// AI-HINT: Изменяй через `ResMut<WorldInfo>` только в startup или по событию.
#[derive(Resource, Debug, Clone)]
pub struct WorldInfo {
    /// Вектор гравитации (по умолчанию: 2D гравитация вниз).
    pub gravity: Vec2,

    /// Имя текущей сцены (пусто до первой загрузки).
    pub current_scene: String,

    /// Масштаб пиксель/единица мира (для 2D).
    /// По умолчанию: 1 пиксель = 1 единица мира.
    pub pixels_per_unit: f32,

    /// Паузировано ли время (Play/Pause в редакторе).
    pub is_paused: bool,
}

impl Default for WorldInfo {
    fn default() -> Self {
        Self {
            // Стандартная 2D гравитация: 9.8 * 50 пикселей/сек²
            gravity: Vec2::new(0.0, -490.0),
            current_scene: String::new(),
            // 1.0 = 1 пиксель = 1 единица (удобно для Bevy 2D)
            pixels_per_unit: 1.0,
            is_paused: false,
        }
    }
}
