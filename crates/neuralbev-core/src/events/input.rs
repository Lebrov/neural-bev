//! # events::input
//!
//! Абстрактные входные события (действие-ориентированный ввод).
//!
//! ## AI-HINT
//! Вместо `KeyCode::W` используй именованные Action-строки (`"MoveRight"`).
//! Маппинг клавиш → Action настраивается в ресурсе InputMap (не здесь).

use bevy::prelude::*;

// ──────────────────────────────────────────────────────────────────────────────

/// WHAT: Нажатие абстрактного действия (в начале кадра нажатия).  
/// WHY:  Отвязывает игровую логику от конкретных клавиш — можно менять управление
///       без изменения систем.  
/// HOW:  ```rust
///       fn on_jump(mut ev: EventReader<ActionPressed>) {
///           for ap in ev.read() {
///               if ap.action == "Jump" {
///                   // apply jump impulse
///               }
///           }
///       }
///       ```
/// AI-HINT: Список доступных действий: MoveRight, MoveLeft, MoveUp, MoveDown,
///           Jump, Attack, Interact, Pause.
#[derive(Event, Debug, Clone)]
pub struct ActionPressed {
    /// Имя действия (например `"Jump"`, `"Attack"`).
    pub action: String,
}

// ──────────────────────────────────────────────────────────────────────────────

/// WHAT: Отпускание абстрактного действия.  
/// WHY:  Для действий с удержанием (заряд удара, прицеливание).  
/// AI-HINT: Используй вместе с [`ActionPressed`] для charge-механик.
#[derive(Event, Debug, Clone)]
pub struct ActionReleased {
    /// Имя действия.
    pub action: String,
}

// ──────────────────────────────────────────────────────────────────────────────

/// WHAT: Изменение аналогового значения оси (стик геймпада, мышь).  
/// WHY:  Для плавного движения/прицеливания, где нужен float, а не bool.  
/// AI-HINT: `value` в диапазоне `[-1.0, 1.0]`.
#[derive(Event, Debug, Clone)]
pub struct AxisChanged {
    /// Имя оси (например `"MoveX"`, `"MoveY"`, `"LookX"`).
    pub axis: String,
    /// Значение оси: `-1.0` … `1.0`.
    pub value: f32,
}
