# NeuralBev 🧠⚡

**AI-Friendly Game Engine поверх Bevy**

NeuralBev — игровой движок, оптимизированный для работы с AI-агентами (вайбкодинг).
Предсказуемая архитектура, богатые doc-комментарии, декларативный API.

---

## Быстрый старт

> **Требования:** Rust 1.80+, Cargo

```bash
# Клонируй репозиторий
git clone https://github.com/your-org/neuralbev

# Запусти рантайм
cargo run --bin runtime

# Запусти редактор (Phase 2 stub)
cargo run --bin editor

# Запусти примеры
cargo run --example platformer -p neuralbev-examples
cargo run --example scene_load -p neuralbev-examples
```

---

## Структура проекта

```
neuralbev/
├── crates/
│   ├── neuralbev-core/     # ECS-ядро, компоненты, Blueprint, Events, Scene I/O
│   └── neuralbev-macros/   # proc-макрос #[neuralbev_component]
├── runtime/                # cargo run --bin runtime
├── editor/                 # cargo run --bin editor (Phase 2)
├── examples/               # cargo run --example <name>
│   ├── platformer.rs
│   └── scene_load.rs
├── assets/
│   ├── scenes/             # .neuralbev.yaml файлы сцен
│   └── scripts/            # Lua скрипты
└── docs/
    ├── AI_GUIDE.md         # ← Читай это первым!
    ├── COOKBOOK.md
    └── ARCHITECTURE.md
```

---

## Ключевые концепции

| Концепция | Описание |
|-----------|----------|
| `#[neuralbev_component]` | Макрос для объявления компонентов |
| `SpawnBlueprint` | Event-builder для создания сущностей |
| `SceneSerializer` | Сохранение/загрузка сцен из YAML |
| `LoadSceneRequest` | Компонент-запрос на загрузку сцены |
| `events::gameplay::HealthChanged` | Правильный способ менять HP |

---

## Для AI-агентов

📖 **Начни с [`docs/AI_GUIDE.md`](docs/AI_GUIDE.md)**

Там есть:
- Карта всех модулей
- Как создать сущность / компонент / систему
- Что можно и нельзя делать
- Диагностика проблем

---

## Фазы разработки

- [x] **Фаза 1: Core** — компоненты, Blueprint, Events, Scene I/O ✅
- [ ] **Фаза 2: Editor** — egui редактор (Hierarchy, Inspector, Viewport)
- [ ] **Фаза 3: Scripting** — Lua hot-reload через mlua
- [ ] **Фаза 4: Polish** — AI Assistant Panel, Asset Browser

---

## Лицензия

MIT OR Apache-2.0
