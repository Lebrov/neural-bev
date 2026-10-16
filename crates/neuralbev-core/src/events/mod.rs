//! # events
//!
//! Типизированные события NeuralBev.
//!
//! ## Иерархия
//! ```text
//! events::
//!   world::     SceneLoaded, SceneUnloaded, EntitySpawned, EntityDespawned
//!   gameplay::  HealthChanged, EntityDied
//!   physics::   CollisionStarted, CollisionEnded
//!   input::     ActionPressed, ActionReleased, AxisChanged
//! ```
//!
//! ## AI-HINT
//! Используй события для коммуникации между системами вместо прямых Query.
//! Пример отправки события:
//! ```rust
//! ev_health_changed.send(HealthChanged {
//!     entity,
//!     delta: -10.0,
//!     new_value: health.current,
//! });
//! ```

pub mod gameplay;
pub mod input;
pub mod physics;
pub mod world;

use bevy::prelude::*;

/// WHAT: Регистрирует все события движка в App.  
/// WHY:  События должны быть зарегистрированы перед использованием.  
/// HOW:  Вызывается автоматически из [`crate::NeuralBevCorePlugin`].  
/// AI-HINT: При добавлении нового события — добавь его сюда.
pub fn register_all_events(app: &mut App) {
    // World events
    app.add_event::<world::SceneLoaded>()
        .add_event::<world::SceneUnloaded>()
        .add_event::<world::EntitySpawned>()
        .add_event::<world::EntityDespawned>();

    // Gameplay events
    app.add_event::<gameplay::HealthChanged>()
        .add_event::<gameplay::EntityDied>();

    // Physics events
    app.add_event::<physics::CollisionStarted>()
        .add_event::<physics::CollisionEnded>();

    // Input events
    app.add_event::<input::ActionPressed>()
        .add_event::<input::ActionReleased>()
        .add_event::<input::AxisChanged>();
}
