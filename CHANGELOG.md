# CHANGELOG

Все значимые изменения фиксируются в этом файле.
Формат основан на [Keep a Changelog](https://keepachangelog.com/ru/1.0.0/).

---

## [0.1.0] — 2026-10-16

### Добавлено (Фаза 1: Core)

#### neuralbev-macros
- Процедурный макрос `#[neuralbev_component]` — автоматически добавляет
  `derive(Component, Reflect, Default, Debug, Clone, Serialize, Deserialize)`,
  атрибут `#[reflect(Component, Default)]` и реализацию трейта `ComponentMeta`.

#### neuralbev-core — компоненты
- `Health` — текущее/максимальное HP, таймер неуязвимости. Методы: `new`, `take_damage`, `heal`, `ratio`, `is_alive`, `is_invincible`.
- `Movement` — скорость, максимальная скорость, трение. Метод: `with_max_speed`.
- `Tags` — строковые теги. Методы: `add`, `remove`, `has`, `from_slice`.
- `EntityName` — человекочитаемое имя сущности.
- `ScriptRef` — ссылка на Lua/Rhai скрипт. Методы: `lua`, `rhai`.
- `AiBehavior` — AI-поведение (Idle, Patrol, Chase). Методы: `patrol`, `idle`.

#### neuralbev-core — Blueprint-система
- `SpawnBlueprint` — event-builder для декларативного создания сущностей.
  Builder-методы: `new`, `tag`, `at`, `health`, `movement`, `script`, `ai_behavior`.
- `SpawnBlueprint::from_yaml_str` — десериализация Blueprint из YAML.
- `SpawnBlueprint::from_blueprint_data` — конвертация из `BlueprintData`.
- `BlueprintRegistry` — ресурс реестра именованных Blueprint'ов.
- `blueprint_spawn_system` — система обработки очереди спауна.

#### neuralbev-core — Event-система
- `events::world` — `SceneLoaded`, `SceneUnloaded`, `EntitySpawned`, `EntityDespawned`.
- `events::gameplay` — `HealthChanged`, `EntityDied`, `DamageSource`.
- `events::physics` — `CollisionStarted`, `CollisionEnded`.
- `events::input` — `ActionPressed`, `ActionReleased`, `AxisChanged`.

#### neuralbev-core — Scene serialization
- `SceneSerializer` — ресурс для загрузки/сохранения сцен. Методы: `load_from_str`, `save_to_string`.
- `LoadSceneRequest` — компонент-запрос на загрузку сцены из YAML.
- `process_load_scene_requests` — система обработки запросов загрузки.
- Формат `.neuralbev.yaml` с разделами `meta` и `entities`.

#### neuralbev-core — Системы
- `systems::health::invincibility_tick_system` — уменьшает таймер неуязвимости.
- `systems::health::death_detection_system` — посылает `EntityDied` при HP = 0.
- `systems::movement::movement_apply_system` — применяет `velocity` к `Transform`.
- `systems::ai_behavior::patrol_system` — патрульное движение между точками.

#### Инфраструктура
- Workspace Cargo.toml с общими зависимостями.
- `runtime` — бинарник `cargo run --bin runtime`.
- `editor` — заглушка `cargo run --bin editor` (реализация в Фазе 2).
- `examples/platformer` — демо платформера.
- `examples/scene_load` — демо загрузки сцены из YAML.
- `docs/AI_GUIDE.md` — гид для AI-агентов.
- `docs/COOKBOOK.md` — рецепты для жанров игр.

### Технические решения
- Все события регистрируются в `register_all_events` — единая точка.
- Все компоненты регистрируются в `register_all_components` — единая точка.
- Никаких `unwrap()` — только `expect("описание")` или `?`.
- Все "магические" числа — именованные константы с комментарием.
- Unit-тесты для всех публичных методов компонентов и Blueprint-системы.

---

## [Unreleased]

### Запланировано (Фаза 2: Editor)
- egui + bevy_egui редактор
- Scene Hierarchy Panel
- Component Inspector с автовиджетами
- Viewport с Transform Gizmo
- Undo/Redo система (Command-паттерн)

### Запланировано (Фаза 3: Scripting)
- Lua-интеграция через mlua
- Hot-reload скриптов
- API: entity, world, input, time, math

### Запланировано (Фаза 4: Polish)
- AI Assistant Panel
- Asset Browser с превью
- Console с фильтрами
