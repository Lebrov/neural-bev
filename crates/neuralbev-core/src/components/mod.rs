//! # components
//!
//! Все базовые компоненты NeuralBev.
//!
//! ## AI-HINT
//! - Для нового компонента: создай здесь struct с `#[neuralbev_component]`,
//!   зарегистрируй его в `register_all_components`.
//! - Не используй `unwrap()` в методах компонентов.
//! - Каждый компонент имеет `Default` — значения по умолчанию всегда
//!   в пределах допустимого диапазона.
//!
//! ## Список компонентов
//! - [`Health`] — очки здоровья и неуязвимость
//! - [`Movement`] — скорость и направление движения
//! - [`Tags`] — произвольные строковые теги (аналог Unity tags)
//! - [`ScriptRef`] — ссылка на Lua/Rhai скрипт сущности
//! - [`AiBehavior`] — описание AI-поведения (патруль, преследование, …)
//! - [`EntityName`] — человекочитаемое имя сущности

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

// Переэкспортируем макрос
pub use neuralbev_macros::neuralbev_component;

// ──────────────────────────────────────────────────────────────────────────────
// Вспомогательный трейт ComponentMeta
// ──────────────────────────────────────────────────────────────────────────────

/// WHAT: Трейт, добавляемый макросом `#[neuralbev_component]` ко всем компонентам.  
/// WHY:  Позволяет Inspector и сериализатору получить имя компонента без Any.  
/// HOW:  Реализуется автоматически макросом, вручную реализовывать не надо.  
/// AI-HINT: Можешь запросить имя компонента через `MyComp::component_name()`.
pub trait ComponentMeta {
    /// Возвращает строковое имя компонента — то же, что в YAML-сцене.
    fn component_name() -> &'static str
    where
        Self: Sized;
}

// ──────────────────────────────────────────────────────────────────────────────
// Health
// ──────────────────────────────────────────────────────────────────────────────

/// WHAT: Компонент здоровья сущности.  
/// WHY:  Единый источник истины о HP — все системы урона/лечения работают
///       только через этот компонент + событие [`crate::events::gameplay::HealthChanged`].  
/// HOW:  ```rust
///       commands.spawn(Health::new(100.0));
///       // Уменьшить HP — посылай событие HealthChanged, не пиши напрямую
///       ```
/// AI-HINT: Не изменяй `current` напрямую из чужих систем. Используй событие
///           `HealthChanged` — это сохраняет возможность Undo и аудит в логах.
#[derive(Component, Reflect, Default, Debug, Clone, Serialize, Deserialize)]
#[reflect(Component, Default)]
pub struct Health {
    /// Текущее здоровье. Диапазон: `[0.0, max_health]`.
    pub current: f32,

    /// Максимальное здоровье. AI-HINT: меняй только через событие `MaxHealthChanged`.
    pub max_health: f32,

    /// Оставшееся время неуязвимости в секундах (0.0 = нет иммунитета).
    pub invincibility_timer: f32,
}

impl ComponentMeta for Health {
    fn component_name() -> &'static str {
        "Health"
    }
}

impl Health {
    /// Создаёт компонент с полным здоровьем. `max` должен быть > 0.
    pub fn new(max: f32) -> Self {
        // Почему assert, не panic: помогает поймать ошибку при Blueprint-создании
        assert!(max > 0.0, "Health::new: max_health должен быть > 0, получено {max}");
        Self {
            current: max,
            max_health: max,
            invincibility_timer: 0.0,
        }
    }

    /// Возвращает `true`, если сущность жива.
    pub fn is_alive(&self) -> bool {
        self.current > 0.0
    }

    /// Возвращает `true`, если сейчас активен иммунитет к урону.
    pub fn is_invincible(&self) -> bool {
        self.invincibility_timer > 0.0
    }

    /// Нормализованное HP в диапазоне `[0.0, 1.0]` — удобно для HealthBar UI.
    pub fn ratio(&self) -> f32 {
        if self.max_health <= 0.0 {
            return 0.0;
        }
        (self.current / self.max_health).clamp(0.0, 1.0)
    }

    /// Применяет урон с учётом неуязвимости. Возвращает фактически нанесённый урон.
    pub fn take_damage(&mut self, amount: f32, invincibility_seconds: f32) -> f32 {
        if self.is_invincible() || amount <= 0.0 {
            return 0.0;
        }
        let actual = amount.min(self.current);
        self.current -= actual;
        self.invincibility_timer = invincibility_seconds;
        actual
    }

    /// Применяет лечение. Не выходит за `max_health`. Возвращает фактически восстановленное.
    pub fn heal(&mut self, amount: f32) -> f32 {
        let actual = amount.min(self.max_health - self.current);
        self.current += actual;
        actual
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Movement
// ──────────────────────────────────────────────────────────────────────────────

/// WHAT: Компонент, управляющий движением сущности.  
/// WHY:  Отделяет намерение (желаемая скорость) от исполнения (физика/Transform).
///       Системы игрока и AI записывают сюда `velocity`, система движения применяет.  
/// HOW:  ```rust
///       // В системе управления игроком:
///       movement.velocity = Vec3::new(speed, 0.0, 0.0);
///       // Система movement_apply_system применяет к Transform автоматически.
///       ```
/// AI-HINT: Пиши в `velocity`, не двигай `Transform` вручную —
///           иначе физика потеряет синхронизацию.
#[derive(Component, Reflect, Default, Debug, Clone, Serialize, Deserialize)]
#[reflect(Component, Default)]
pub struct Movement {
    /// Скорость в единицах/секунду. Система движения умножает на `delta_time`.
    pub velocity: Vec3,

    /// Максимальная скорость. `0.0` = неограничено.
    pub max_speed: f32,

    /// Коэффициент трения `[0.0, 1.0]`. `1.0` = нет трения, `0.0` = мгновенная остановка.
    pub friction: f32,
}

impl ComponentMeta for Movement {
    fn component_name() -> &'static str {
        "Movement"
    }
}

impl Movement {
    /// Создаёт компонент с заданной максимальной скоростью.
    pub fn with_max_speed(max_speed: f32) -> Self {
        Self {
            velocity: Vec3::ZERO,
            max_speed,
            // Значение 0.85 — приятное аркадное трение
            friction: 0.85,
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Tags
// ──────────────────────────────────────────────────────────────────────────────

/// WHAT: Набор строковых тегов сущности.  
/// WHY:  Позволяет фильтровать сущности в скриптах и системах без создания
///       маркерных компонентов на каждый тип.  
/// HOW:  ```rust
///       let mut tags = Tags::default();
///       tags.add("enemy");
///       tags.add("flying");
///       commands.spawn(tags);
///       // В скрипте: if other:has_tag("enemy") then ...
///       ```
/// AI-HINT: Не используй теги как замену компонентам. Теги — для грубой
///           категоризации. Данные — в отдельных компонентах.
#[derive(Component, Reflect, Default, Debug, Clone, Serialize, Deserialize)]
#[reflect(Component, Default)]
pub struct Tags {
    /// Список строковых тегов.
    pub list: Vec<String>,
}

impl ComponentMeta for Tags {
    fn component_name() -> &'static str {
        "Tags"
    }
}

impl Tags {
    /// Добавляет тег, если ещё не присутствует.
    pub fn add(&mut self, tag: impl Into<String>) {
        let t = tag.into();
        if !self.list.contains(&t) {
            self.list.push(t);
        }
    }

    /// Удаляет тег. Возвращает `true` если тег был найден и удалён.
    pub fn remove(&mut self, tag: &str) -> bool {
        if let Some(pos) = self.list.iter().position(|t| t == tag) {
            self.list.remove(pos);
            true
        } else {
            false
        }
    }

    /// Проверяет наличие тега.
    pub fn has(&self, tag: &str) -> bool {
        self.list.iter().any(|t| t == tag)
    }

    /// Создаёт Tags из списка строк.
    pub fn from_slice(tags: &[&str]) -> Self {
        Self {
            list: tags.iter().map(|s| s.to_string()).collect(),
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// EntityName
// ──────────────────────────────────────────────────────────────────────────────

/// WHAT: Человекочитаемое имя сущности (отображается в Hierarchy и логах).  
/// WHY:  Bevy `Entity` — числовой ID. Для редактора и отладки нужно имя.  
/// HOW:  ```rust
///       commands.spawn(EntityName::new("Player"));
///       ```
/// AI-HINT: Используй уникальные имена в сцене — Inspector и поиск по ним работают.
#[derive(Component, Reflect, Default, Debug, Clone, Serialize, Deserialize)]
#[reflect(Component, Default)]
pub struct EntityName {
    /// Имя сущности. Рекомендуется: PascalCase или "Goblin_01".
    pub name: String,
}

impl ComponentMeta for EntityName {
    fn component_name() -> &'static str {
        "EntityName"
    }
}

impl EntityName {
    /// Создаёт компонент с именем.
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// ScriptRef
// ──────────────────────────────────────────────────────────────────────────────

/// WHAT: Ссылка на файл Lua/Rhai скрипта, прикреплённого к сущности.  
/// WHY:  Позволяет скриптовому слою найти и выполнить поведение сущности.  
/// HOW:  ```rust
///       commands.spawn(ScriptRef::lua("scripts/player_controller.lua"));
///       ```
/// AI-HINT: Путь — относительно папки assets проекта.
///           Скрипт горячо перезагружается при изменении файла.
#[derive(Component, Reflect, Default, Debug, Clone, Serialize, Deserialize)]
#[reflect(Component, Default)]
pub struct ScriptRef {
    /// Путь к файлу скрипта, относительно assets/.
    pub path: String,

    /// Язык скрипта.
    pub language: ScriptLanguage,
}

impl ComponentMeta for ScriptRef {
    fn component_name() -> &'static str {
        "ScriptRef"
    }
}

impl ScriptRef {
    /// Создаёт ссылку на Lua-скрипт.
    pub fn lua(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            language: ScriptLanguage::Lua,
        }
    }

    /// Создаёт ссылку на Rhai-скрипт.
    pub fn rhai(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            language: ScriptLanguage::Rhai,
        }
    }
}

/// Поддерживаемые языки скриптов.
#[derive(Reflect, Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScriptLanguage {
    /// Lua 5.4 через mlua.
    #[default]
    Lua,
    /// Rhai через crate rhai.
    Rhai,
}

// ──────────────────────────────────────────────────────────────────────────────
// AiBehavior
// ──────────────────────────────────────────────────────────────────────────────

/// WHAT: Описывает AI-поведение неигровой сущности.  
/// WHY:  Позволяет задать поведение декларативно из YAML без написания кода.  
/// HOW:  ```rust
///       commands.spawn(AiBehavior::patrol(vec![
///           Vec2::new(0.0, 0.0),
///           Vec2::new(200.0, 0.0),
///       ], 80.0));
///       ```
/// AI-HINT: Для сложного AI используй ScriptRef + Lua-скрипт. AiBehavior —
///           для простых паттернов (патруль, охрана точки).
#[derive(Component, Reflect, Debug, Clone, Serialize, Deserialize)]
#[reflect(Component)]
pub struct AiBehavior {
    /// Текущий тип поведения.
    pub behavior_type: AiBehaviorType,
    /// Скорость перемещения в ед./сек.
    pub speed: f32,
}

impl ComponentMeta for AiBehavior {
    fn component_name() -> &'static str {
        "AiBehavior"
    }
}

impl Default for AiBehavior {
    fn default() -> Self {
        Self {
            behavior_type: AiBehaviorType::Idle,
            speed: 80.0,
        }
    }
}

impl AiBehavior {
    /// Создаёт патрульное поведение между точками.
    pub fn patrol(points: Vec<Vec2>, speed: f32) -> Self {
        Self {
            behavior_type: AiBehaviorType::Patrol {
                points,
                current_target_idx: 0,
            },
            speed,
        }
    }

    /// Создаёт поведение «стоять на месте».
    pub fn idle() -> Self {
        Self {
            behavior_type: AiBehaviorType::Idle,
            speed: 0.0,
        }
    }
}

/// Типы AI-поведений.
#[derive(Reflect, Debug, Clone, Serialize, Deserialize)]
pub enum AiBehaviorType {
    /// Стоять на месте.
    Idle,
    /// Патрулировать между точками.
    Patrol {
        /// Точки маршрута в 2D-пространстве.
        points: Vec<Vec2>,
        /// Индекс текущей целевой точки.
        current_target_idx: usize,
    },
    /// Преследовать сущность с тегом.
    Chase {
        /// Тег цели преследования (например `"player"`).
        target_tag: String,
        /// Дистанция, с которой начинается преследование.
        detection_range: f32,
    },
}

// ──────────────────────────────────────────────────────────────────────────────
// Регистрация компонентов
// ──────────────────────────────────────────────────────────────────────────────

/// WHAT: Регистрирует все компоненты в Bevy TypeRegistry.  
/// WHY:  Без регистрации компоненты не попадут в Inspector и не сериализуются.  
/// HOW:  Вызывается автоматически из [`crate::NeuralBevCorePlugin`].  
/// AI-HINT: При добавлении нового компонента — добавь его сюда.
pub fn register_all_components(app: &mut bevy::prelude::App) {
    app.register_type::<Health>()
        .register_type::<Movement>()
        .register_type::<Tags>()
        .register_type::<EntityName>()
        .register_type::<ScriptRef>()
        .register_type::<ScriptLanguage>()
        .register_type::<AiBehavior>()
        .register_type::<AiBehaviorType>();
}

// ──────────────────────────────────────────────────────────────────────────────
// Тесты
// ──────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_new_full() {
        let h = Health::new(100.0);
        assert_eq!(h.current, 100.0);
        assert_eq!(h.max_health, 100.0);
        assert!(h.is_alive());
        assert!(!h.is_invincible());
    }

    #[test]
    fn health_take_damage_applies_correctly() {
        let mut h = Health::new(100.0);
        let dealt = h.take_damage(30.0, 0.5);
        assert_eq!(dealt, 30.0);
        assert_eq!(h.current, 70.0);
        assert!(h.is_invincible());
    }

    #[test]
    fn health_invincible_blocks_damage() {
        let mut h = Health::new(100.0);
        h.take_damage(10.0, 1.0); // первый удар — выдаёт иммунитет
        let dealt = h.take_damage(50.0, 0.0); // не должен пройти
        assert_eq!(dealt, 0.0);
        assert_eq!(h.current, 90.0);
    }

    #[test]
    fn health_cannot_go_below_zero() {
        let mut h = Health::new(10.0);
        let dealt = h.take_damage(999.0, 0.0);
        assert_eq!(dealt, 10.0);
        assert_eq!(h.current, 0.0);
        assert!(!h.is_alive());
    }

    #[test]
    fn health_heal_clamps_to_max() {
        let mut h = Health::new(100.0);
        h.take_damage(50.0, 0.0);
        let healed = h.heal(200.0);
        assert_eq!(healed, 50.0);
        assert_eq!(h.current, 100.0);
    }

    #[test]
    fn health_ratio_correct() {
        let mut h = Health::new(100.0);
        h.take_damage(25.0, 0.0);
        assert!((h.ratio() - 0.75).abs() < f32::EPSILON);
    }

    #[test]
    fn tags_add_remove() {
        let mut t = Tags::default();
        t.add("enemy");
        t.add("flying");
        t.add("enemy"); // дубликат
        assert_eq!(t.list.len(), 2);
        assert!(t.has("flying"));
        assert!(t.remove("flying"));
        assert!(!t.has("flying"));
    }

    #[test]
    #[should_panic]
    fn health_new_zero_panics() {
        let _ = Health::new(0.0);
    }
}
