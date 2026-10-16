# NeuralBev AI Guide

> **Этот файл — первое что нужно прочитать AI-агенту перед работой с кодовой базой.**

## Содержание

1. [Что такое NeuralBev](#что-такое-neuralbev)
2. [Карта модулей](#карта-модулей)
3. [Как создать сущность](#как-создать-сущность)
4. [Как добавить компонент](#как-добавить-компонент)
5. [Как добавить систему](#как-добавить-систему)
6. [Как загрузить сцену](#как-загрузить-сцену)
7. [Частые паттерны](#частые-паттерны)
8. [Что НЕЛЬЗЯ делать](#что-нельзя-делать)
9. [Диагностика проблем](#диагностика-проблем)

---

## Что такое NeuralBev

NeuralBev — игровой движок поверх Bevy, оптимизированный для AI-агентов.

**Ключевые принципы:**
- Всё создаётся через **Blueprint** (не прямой `spawn`)
- Коммуникация через **события** (не прямые Query)
- Компоненты объявляются через **`#[neuralbev_component]`**
- Сцены описываются в **YAML**

---

## Карта модулей

```
neuralbev/
├── crates/
│   ├── neuralbev-macros/   # proc-макрос #[neuralbev_component]
│   └── neuralbev-core/
│       └── src/
│           ├── lib.rs              ← точка входа, NeuralBevCorePlugin
│           ├── components/mod.rs   ← Health, Movement, Tags, EntityName, ScriptRef, AiBehavior
│           ├── entity/mod.rs       ← SpawnBlueprint, BlueprintRegistry, blueprint_spawn_system
│           ├── events/
│           │   ├── mod.rs          ← register_all_events
│           │   ├── world.rs        ← SceneLoaded, EntitySpawned, EntityDespawned
│           │   ├── gameplay.rs     ← HealthChanged, EntityDied
│           │   ├── physics.rs      ← CollisionStarted, CollisionEnded
│           │   └── input.rs        ← ActionPressed, ActionReleased, AxisChanged
│           ├── scene/mod.rs        ← SceneSerializer, LoadSceneRequest
│           ├── systems/
│           │   ├── health.rs       ← invincibility_tick, death_detection
│           │   ├── movement.rs     ← movement_apply
│           │   └── ai_behavior.rs  ← patrol_system
│           └── world/mod.rs        ← WorldInfo resource
├── runtime/src/main.rs     ← cargo run --bin runtime
├── editor/src/main.rs      ← cargo run --bin editor (Phase 2 stub)
└── examples/examples/
    ├── platformer.rs       ← cargo run --example platformer
    └── scene_load.rs       ← cargo run --example scene_load
```

---

## Как создать сущность

### Способ 1: Через код (Builder API)

```rust
// В любой системе с EventWriter<SpawnBlueprint>
fn my_system(mut ev_spawn: EventWriter<SpawnBlueprint>) {
    ev_spawn.send(
        SpawnBlueprint::new("Player")
            .tag("player")
            .tag("controllable")
            .health(Health::new(100.0))
            .movement(Movement::with_max_speed(300.0))
            .at(Transform::from_xyz(0.0, 0.0, 0.0))
    );
}
```

### Способ 2: Из YAML

```rust
fn load_enemy(mut commands: Commands) {
    let yaml = r#"
id: goblin
name: Goblin
tags: [enemy]
components:
  Transform:
    translation: [100.0, 0.0, 0.0]
    scale: [1.0, 1.0, 1.0]
  Health:
    current: 30.0
    max_health: 30.0
    invincibility_timer: 0.0
"#;
    let bp = SpawnBlueprint::from_yaml_str(yaml).expect("valid yaml");
    // Отправь в EventWriter<SpawnBlueprint>
}
```

### Способ 3: Загрузка целой сцены

```rust
fn load_level(mut commands: Commands) {
    commands.spawn(LoadSceneRequest {
        yaml: include_str!("../assets/scenes/level1.neuralbev.yaml").to_string(),
    });
}
```

---

## Как добавить компонент

### Шаг 1: Объяви компонент

```rust
// В файле crates/neuralbev-core/src/components/mod.rs

#[neuralbev_component]  // ← этот макрос добавляет все нужные трейты
pub struct Inventory {
    /// Максимум предметов. AI-HINT: проверяй перед добавлением предмета.
    pub capacity: usize,
    /// Список предметов.
    pub items: Vec<String>,
}

impl Inventory {
    /// Создаёт пустой инвентарь с заданной вместимостью.
    pub fn new(capacity: usize) -> Self {
        Self { capacity, items: Vec::new() }
    }
}
```

### Шаг 2: Зарегистрируй в register_all_components

```rust
// В functions register_all_components:
app.register_type::<Inventory>();
```

### Шаг 3: Добавь в SpawnBlueprint (опционально)

```rust
// В entity/mod.rs добавь поле и builder-метод:
pub inventory: Option<Inventory>,

pub fn inventory(mut self, inv: Inventory) -> Self {
    self.inventory = Some(inv);
    self
}
```

---

## Как добавить систему

```rust
// В crates/neuralbev-core/src/systems/my_system.rs

/// WHAT: Что делает система.
/// WHY: Зачем нужна.
/// HOW: Как добавить в App.
/// AI-HINT: В какой стадии работает, от чего зависит.
pub fn my_system(
    time: Res<Time>,
    mut query: Query<(&mut MyComponent, &Transform)>,
    mut ev_result: EventWriter<MyEvent>,
) {
    for (mut comp, transform) in query.iter_mut() {
        // логика
    }
}
```

Зарегистрируй в `NeuralBevCorePlugin::build`:

```rust
app.add_systems(Update, systems::my_module::my_system);
```

---

## Как загрузить сцену

```rust
// Вариант 1: из строки
let serializer = SceneSerializer::default();
let loaded = serializer.load_from_str(yaml_str)?;
for bp in loaded.blueprints {
    ev_spawn.send(bp);
}

// Вариант 2: через компонент-запрос (рекомендуется)
commands.spawn(LoadSceneRequest { yaml: yaml_str.to_string() });
// Система process_load_scene_requests обработает его в следующем кадре
```

---

## Частые паттерны

### Движение игрока

```rust
fn player_move(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Movement, With<PlayerMarker>>,
) {
    for mut movement in query.iter_mut() {
        movement.velocity.x = if keyboard.pressed(KeyCode::KeyD) { 300.0 }
                               else if keyboard.pressed(KeyCode::KeyA) { -300.0 }
                               else { 0.0 };
    }
}
```

### Нанесение урона

```rust
fn deal_damage(
    mut ev_health: EventWriter<HealthChanged>,
    entity: Entity,
    amount: f32,
) {
    ev_health.send(HealthChanged {
        entity,
        delta: -amount,
        new_value: current_hp - amount,  // вычисли заранее
        source: DamageSource::Entity(attacker),
    });
}
```

### Реакция на смерть

```rust
fn on_death(
    mut ev_died: EventReader<EntityDied>,
    mut commands: Commands,
) {
    for ev in ev_died.read() {
        commands.entity(ev.entity).despawn_recursive();
        // + анимация, звук, лут
    }
}
```

### Фильтрация по тегу

```rust
fn hit_enemies(query: Query<(Entity, &Tags)>) {
    for (entity, tags) in query.iter() {
        if tags.has("enemy") {
            // обработай
        }
    }
}
```

---

## Что НЕЛЬЗЯ делать

| ❌ Нельзя | ✅ Нужно |
|-----------|----------|
| `health.current -= 10.0` напрямую | Посылай `HealthChanged` |
| `transform.translation += delta` вручную | Пиши в `Movement.velocity` |
| `commands.spawn((Component1, Component2, ...))` | Используй `SpawnBlueprint` |
| `unwrap()` без причины | `expect("описание")` или `?` |
| Магические числа `if speed > 5.0` | Именованная константа `PATROL_ARRIVAL_DISTANCE` |
| Системы без doc-комментария | Обязательный WHAT/WHY/HOW/AI-HINT |
| Прямой доступ к `World` из системы | Через запросы и ресурсы |

---

## Диагностика проблем

**Компонент не появляется в Inspector?**
→ Проверь, зарегистрирован ли тип: `app.register_type::<MyComp>()`

**Blueprint не спаунит сущность?**
→ Убедись, что `blueprint_spawn_system` добавлен в App (через `NeuralBevCorePlugin`)
→ Проверь, что событие `SpawnBlueprint` отправлено через `EventWriter`, не напрямую

**Сцена не загружается из YAML?**
→ Проверь версию: `engine_version: "0.1.0"` в секции `meta`
→ Числа должны быть float: `100.0`, не `100`

**Движение не работает?**
→ Убедись, что у сущности есть компонент `Movement`
→ Пиши в `movement.velocity`, не в `transform.translation`
