-- player_controller.lua
-- AI-HINT: Это скрипт поведения игрока. on_update вызывается каждый кадр.
-- Доступные объекты: entity, world, input, time, math
-- Фаза 3 (scripting) подключит mlua и активирует этот скрипт.

-- Скорость движения игрока в единицах/секунду
local SPEED = 200.0

function on_start(entity)
    entity:set_name("Player")
    print("[Script] Игрок инициализирован")
end

function on_update(entity, dt)
    local vx, vy = 0, 0

    if input.is_held("MoveRight") then vx = SPEED end
    if input.is_held("MoveLeft")  then vx = -SPEED end
    if input.is_held("MoveUp")    then vy = SPEED end
    if input.is_held("MoveDown")  then vy = -SPEED end

    -- Пишем в компонент Movement, не двигаем Transform напрямую
    local mv = entity:get_component("Movement")
    if mv then
        mv.velocity = { x = vx, y = vy, z = 0 }
    end

    if input.is_just_pressed("Jump") then
        entity:send_event("Jump", {})
    end
end

function on_collision(entity, other, normal)
    if other:has_tag("Enemy") then
        local health = entity:get_component("Health")
        if health and not health.invincibility_timer > 0 then
            entity:send_event("TakeDamage", { amount = 10.0 })
        end
    end
end
