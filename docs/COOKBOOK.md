# NeuralBev Cookbook

> Готовые рецепты для типичных жанров игр на NeuralBev.

---

## Рецепт: Платформер

```rust
// 1. Создай игрока
ev_spawn.send(
    SpawnBlueprint::new("Player")
        .tag("player")
        .health(Health::new(100.0))
        .movement(Movement::with_max_speed(350.0))
        .script(ScriptRef::lua("scripts/platformer_player.lua"))
        .at(Transform::from_xyz(0.0, 100.0, 0.0))
);

// 2. Управление (в системе):
fn platformer_input(keyboard: Res<ButtonInput<KeyCode>>,
                    mut query: Query<&mut Movement, With<PlayerTag>>) {
    for mut mv in query.iter_mut() {
        mv.velocity.x = match () {
            _ if keyboard.pressed(KeyCode::KeyA) => -350.0,
            _ if keyboard.pressed(KeyCode::KeyD) =>  350.0,
            _ => 0.0,
        };
        if keyboard.just_pressed(KeyCode::Space) {
            mv.velocity.y = 600.0; // прыжок
        }
    }
}
```

---

## Рецепт: Top-down Shooter

```rust
// Игрок стреляет в направлении мыши:
fn shoot_system(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform)>,
    player: Query<&Transform, With<PlayerTag>>,
    mut ev_spawn: EventWriter<SpawnBlueprint>,
) {
    if !mouse.just_pressed(MouseButton::Left) { return; }

    // Получаем мировую позицию курсора
    let Ok(window) = windows.get_single() else { return };
    let Some(cursor_pos) = window.cursor_position() else { return };
    // ... raycast из камеры ...

    ev_spawn.send(
        SpawnBlueprint::new("Bullet")
            .tag("bullet")
            .movement(Movement { velocity: direction * 600.0, max_speed: 600.0, friction: 1.0 })
            .at(Transform::from_translation(player_pos))
    );
}
```

---

## Рецепт: RPG с диалогами

```yaml
# assets/scenes/village.neuralbev.yaml
entities:
  - id: npc_merchant
    name: "Торговец Иван"
    tags: [npc, merchant]
    components:
      Transform:
        translation: [150.0, 50.0, 0.0]
        scale: [1.0, 1.0, 1.0]
      Script:
        path: scripts/merchant_dialog.lua
```

```lua
-- scripts/merchant_dialog.lua
function on_interact(entity, player)
    world:start_dialog("merchant_dialog.yaml", player)
end
```

---

## Рецепт: Puzzle Game

```rust
// Механика "толкни блок":
fn push_block(
    player: Query<&Transform, With<PlayerTag>>,
    mut blocks: Query<(&Transform, &mut Movement), With<PushableTag>>,
) {
    // При коллизии игрока с блоком — добавляй импульс блоку
    // Слушай CollisionStarted для обнаружения касания
}
```

---

## Рецепт: Бесконечный раннер

```rust
// Генерируй препятствия спереди игрока:
fn spawn_obstacles(
    player: Query<&Transform, With<PlayerTag>>,
    mut ev_spawn: EventWriter<SpawnBlueprint>,
    mut timer: Local<f32>,
    time: Res<Time>,
) {
    *timer += time.delta_secs();
    if *timer < 2.0 { return; } // каждые 2 секунды
    *timer = 0.0;

    let Ok(pt) = player.get_single() else { return };
    ev_spawn.send(
        SpawnBlueprint::new("Obstacle")
            .tag("obstacle")
            .at(Transform::from_xyz(pt.translation.x + 600.0, 0.0, 0.0))
    );
}
```
