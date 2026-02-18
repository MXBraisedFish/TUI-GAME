GAME_META = {
    name = "Pac-Man",
    description = "Collect pellets while avoiding roaming ghosts."
}

local FPS = 60
local FRAME_MS = 16
local PLAYER_STEP_FRAMES = 12
local GHOST_SLOW_FACTOR = 2
local MAX_LEVEL = 20
local EXTRA_LIFE_SCORE = 10000

local PELLET_CHAR = "·"
local POWER_CHAR = "*"
local DOOR_CHAR = "-"

local MAP_TEMPLATE = {
    "╔═════════╦═════════╗",
    "║·········║·········║",
    "║·╔═╗·╔═╗·║·╔═╗·╔═╗·║",
    "║*║ ║·║ ║·║·║ ║·║ ║*║",
    "║·╚═╝·╚═╝·║·╚═╝·╚═╝·║",
    "║···················║",
    "║·╔═╗·║·╔═══╗·║·╔═╗·║",
    "║·╚═╝·║·╚═╦═╝·║·╚═╝·║",
    "║·····║···║···║·····║",
    "╚═══╗·╠══ ║ ══╣·╔═══╝",
    "    ║·║       ║·║    ",
    "════╝·║ ╔═-═╗ ║·╚════",
    "<    ·  ║   ║  ·    >",
    "════╗·║ ╚═══╝ ║·╔════",
    "    ║·║       ║·║    ",
    "    ║·║ ╔═══╗ ║·║    ",
    "╔═══╝·║ ╚═╦═╝ ║·╚═══╗",
    "║·········║·········║",
    "║·══╗·═══·║·═══·╔══·║",
    "║*··║···········║··*║",
    "╠═╗·║·║·╔═══╗·║·║·╔═╣",
    "╠═╝·║·║·╚═╦═╝·║·║·╚═╣",
    "║·····║···║···║·····║",
    "║·════╩══·║·══╩════·║",
    "║···················║",
    "╚═══════════════════╝"
}

local WALL_SET = {
    ["╔"] = true, ["╗"] = true, ["╚"] = true, ["╝"] = true,
    ["═"] = true, ["║"] = true, ["╦"] = true, ["╩"] = true,
    ["╠"] = true, ["╣"] = true,
}

local DIRS = {
    { name = "up", dr = -1, dc = 0 },
    { name = "left", dr = 0, dc = -1 },
    { name = "down", dr = 1, dc = 0 },
    { name = "right", dr = 0, dc = 1 },
}

local FRUIT_TABLE = {
    { symbol = "%", points = 100, key = "game.pacman.fruit.cherry", fallback = "Cherry" },
    { symbol = "U", points = 300, key = "game.pacman.fruit.strawberry", fallback = "Strawberry" },
    { symbol = "O", points = 500, key = "game.pacman.fruit.orange", fallback = "Orange" },
    { symbol = "O", points = 500, key = "game.pacman.fruit.orange", fallback = "Orange" },
    { symbol = "Q", points = 700, key = "game.pacman.fruit.apple", fallback = "Apple" },
    { symbol = "Q", points = 700, key = "game.pacman.fruit.apple", fallback = "Apple" },
    { symbol = "§", points = 1000, key = "game.pacman.fruit.grape", fallback = "Grape" },
    { symbol = "§", points = 1000, key = "game.pacman.fruit.grape", fallback = "Grape" },
    { symbol = "W", points = 2000, key = "game.pacman.fruit.galaxian", fallback = "Galaxian" },
    { symbol = "W", points = 2000, key = "game.pacman.fruit.galaxian", fallback = "Galaxian" },
    { symbol = "?", points = 3000, key = "game.pacman.fruit.bell", fallback = "Bell" },
    { symbol = "?", points = 3000, key = "game.pacman.fruit.bell", fallback = "Bell" },
}

local state = {
    rows = 0, cols = 0,
    base_map = {}, pellets = {},
    total_pellets = 0, remaining_pellets = 0,
    player = { r = 1, c = 1, dir = "left", next_dir = "left", next_step_at = 0 },
    player_start = { r = 1, c = 1 },
    ghosts = {}, ghost_spawn = { r = 1, c = 1 }, global_pause_until = 0,
    tunnel_left = nil, tunnel_right = nil, door_cells = {},
    fruit = { active = false, spawned = false, r = 1, c = 1 },
    level = 1, score = 0, best_score = 0, lives = 3, extra_life_granted = false,
    power_until = 0, power_chain = 0, power_eaten = {},
    phase = "playing", frame = 0, run_start_frame = 0, level_start_frame = 0,
    end_frame = nil, stats_committed = false,
    confirm_mode = nil,
    countdown_until = 0,
    last_countdown_sec = nil,
    info_message = "", info_color = "dark_gray",
    info_message_until = nil,
    collected_fruits = {},
    dirty = true, last_area = nil, last_term_w = 0, last_term_h = 0,
    size_warning_active = false,
    last_warn_term_w = 0, last_warn_term_h = 0, last_warn_min_w = 0, last_warn_min_h = 0,
}

local function tr(key, fallback)
    if type(translate) ~= "function" then return fallback end
    local ok, value = pcall(translate, key)
    if not ok or value == nil or value == "" or value == key then return fallback end
    return value
end

local function key_width(text)
    if type(get_text_width) == "function" then
        local ok, w = pcall(get_text_width, text)
        if ok and type(w) == "number" then return w end
    end
    return #text
end

local function normalize_key(key)
    if key == nil then return "" end
    if type(key) == "string" then return string.lower(key) end
    return tostring(key):lower()
end

local function terminal_size()
    local w, h = 120, 40
    if type(get_terminal_size) == "function" then
        local tw, th = get_terminal_size()
        if type(tw) == "number" and type(th) == "number" then w, h = tw, th end
    end
    return w, h
end

local function wrap_words(text, max_width)
    if max_width <= 1 then return { text } end
    local lines, current, had = {}, "", false
    for token in string.gmatch(text, "%S+") do
        had = true
        if current == "" then
            current = token
        else
            local candidate = current .. " " .. token
            if key_width(candidate) <= max_width then current = candidate else lines[#lines + 1] = current; current = token end
        end
    end
    if not had then return { "" } end
    if current ~= "" then lines[#lines + 1] = current end
    return lines
end

local function min_width_for_lines(text, max_lines, hard_min)
    local full, width = key_width(text), hard_min
    while width <= full do
        if #wrap_words(text, width) <= max_lines then return width end
        width = width + 1
    end
    return full
end

local function clamp(v, lo, hi) if v < lo then return lo end if v > hi then return hi end return v end

local function elapsed_seconds()
    local ending = state.end_frame
    if ending == nil then ending = state.frame end
    return math.floor((ending - state.run_start_frame) / FPS)
end

local function format_duration(sec)
    local h = math.floor(sec / 3600)
    local m = math.floor((sec % 3600) / 60)
    local s = sec % 60
    return string.format("%02d:%02d:%02d", h, m, s)
end

local function utf8_chars(str)
    local out = {}
    for _, code in utf8.codes(str) do out[#out + 1] = utf8.char(code) end
    return out
end

local function blank_matrix(rows, cols, value)
    local m = {}
    for r = 1, rows do
        m[r] = {}
        for c = 1, cols do m[r][c] = value end
    end
    return m
end

local function parse_map()
    local lines, max_cols = {}, 0
    for i = 1, #MAP_TEMPLATE do
        lines[i] = utf8_chars(MAP_TEMPLATE[i])
        if #lines[i] > max_cols then max_cols = #lines[i] end
    end

    state.rows, state.cols = #lines, max_cols
    state.base_map = blank_matrix(state.rows, state.cols, " ")
    state.pellets = blank_matrix(state.rows, state.cols, " ")
    state.door_cells, state.tunnel_left, state.tunnel_right = {}, nil, nil

    for r = 1, state.rows do
        for c = 1, state.cols do
            local ch = lines[r][c] or " "
            state.base_map[r][c] = ch
            state.pellets[r][c] = (ch == PELLET_CHAR or ch == POWER_CHAR) and ch or " "
            if ch == DOOR_CHAR then state.door_cells[#state.door_cells + 1] = { r = r, c = c }
            elseif ch == "<" then state.tunnel_left = { r = r, c = c }
            elseif ch == ">" then state.tunnel_right = { r = r, c = c } end
        end
    end
end

local function in_bounds(r, c) return r >= 1 and r <= state.rows and c >= 1 and c <= state.cols end
local function is_wall(r, c) return (not in_bounds(r, c)) or WALL_SET[state.base_map[r][c]] == true end
local function is_door(r, c) return in_bounds(r, c) and state.base_map[r][c] == DOOR_CHAR end
local function is_walkable(r, c) return in_bounds(r, c) and (not is_wall(r, c)) end

local function apply_tunnel(r, c)
    if state.tunnel_left and state.tunnel_right then
        if r == state.tunnel_left.r and c < 1 then return state.tunnel_right.r, state.tunnel_right.c end
        if r == state.tunnel_right.r and c > state.cols then return state.tunnel_left.r, state.tunnel_left.c end
    end
    return r, c
end

local function can_move_player(r, c)
    r, c = apply_tunnel(r, c)
    if not in_bounds(r, c) or is_wall(r, c) or is_door(r, c) then return false, r, c end
    return true, r, c
end

local function can_move_ghost(r, c)
    r, c = apply_tunnel(r, c)
    if not in_bounds(r, c) or is_wall(r, c) then return false, r, c end
    return true, r, c
end

local function direction_delta(dir)
    if dir == "up" then return -1, 0 end
    if dir == "down" then return 1, 0 end
    if dir == "left" then return 0, -1 end
    return 0, 1
end

local function opposite_dir(dir)
    if dir == "up" then return "down" end
    if dir == "down" then return "up" end
    if dir == "left" then return "right" end
    return "left"
end

local function pellet_counts()
    local total = 0
    for r = 1, state.rows do
        for c = 1, state.cols do
            if state.pellets[r][c] == PELLET_CHAR or state.pellets[r][c] == POWER_CHAR then total = total + 1 end
        end
    end
    return total
end

local function power_duration_sec(level)
    if level <= 1 then return 6 elseif level == 2 then return 5 elseif level <= 10 then return 4 elseif level <= 16 then return 3 else return 2 end
end

local function ghost_revive_sec(level)
    if level >= 11 and level <= 16 then return 5 end
    return 3
end

local function fruit_for_level(level)
    if level >= 13 then return { symbol = "!", points = 5000, key = "game.pacman.fruit.key", fallback = "Key" } end
    return FRUIT_TABLE[level] or FRUIT_TABLE[#FRUIT_TABLE]
end

local function scatter_schedule(level)
    if level >= 17 then
        return { { mode = "scatter", sec = 1 }, { mode = "chase", sec = 20 }, { mode = "scatter", sec = 1 }, { mode = "chase", sec = 20 }, { mode = "chase", sec = 9999 } }
    elseif level >= 5 then
        return { { mode = "scatter", sec = 5 }, { mode = "chase", sec = 20 }, { mode = "scatter", sec = 5 }, { mode = "chase", sec = 20 }, { mode = "scatter", sec = 5 }, { mode = "chase", sec = 9999 } }
    end
    return { { mode = "scatter", sec = 7 }, { mode = "chase", sec = 20 }, { mode = "scatter", sec = 7 }, { mode = "chase", sec = 20 }, { mode = "scatter", sec = 5 }, { mode = "chase", sec = 20 }, { mode = "scatter", sec = 5 }, { mode = "chase", sec = 9999 } }
end

local function current_chase_mode()
    local t = math.floor((state.frame - state.level_start_frame) / FPS)
    local schedule = scatter_schedule(state.level)
    for i = 1, #schedule do
        local seg = schedule[i]
        if t < seg.sec then return seg.mode end
        t = t - seg.sec
    end
    return "chase"
end

local function ghost_release_delays(level)
    if level <= 1 then return { blinky = 0, pinky = 2, inky = 4, clyde = 6 }
    elseif level <= 4 then return { blinky = 0, pinky = 1, inky = 3, clyde = 5 }
    end
    return { blinky = 0, pinky = 1, inky = 2, clyde = 3 }
end

local function find_nearest_walkable(start_r, start_c)
    if is_walkable(start_r, start_c) and (not is_door(start_r, start_c)) then return start_r, start_c end
    local q_r, q_c, head = { start_r }, { start_c }, 1
    local visited = blank_matrix(state.rows, state.cols, false)
    if in_bounds(start_r, start_c) then visited[start_r][start_c] = true end
    while head <= #q_r do
        local r, c = q_r[head], q_c[head]
        head = head + 1
        for i = 1, #DIRS do
            local d = DIRS[i]
            local nr, nc = apply_tunnel(r + d.dr, c + d.dc)
            if in_bounds(nr, nc) and not visited[nr][nc] then
                visited[nr][nc] = true
                if is_walkable(nr, nc) and (not is_door(nr, nc)) then return nr, nc end
                q_r[#q_r + 1], q_c[#q_c + 1] = nr, nc
            end
        end
    end
    return 2, 2
end

local function init_positions_from_map()
    local center_r, center_c = math.floor(state.rows * 0.75), math.floor(state.cols / 2)
    local pr, pc = find_nearest_walkable(center_r, center_c)
    state.player_start = { r = pr, c = pc }

    if #state.door_cells > 0 then
        local d = state.door_cells[math.floor((#state.door_cells + 1) / 2)]
        local sr, sc = find_nearest_walkable(d.r + 1, d.c)
        local fr, fc = find_nearest_walkable(d.r + 3, d.c)
        state.ghost_spawn, state.fruit.r, state.fruit.c = { r = sr, c = sc }, fr, fc
    else
        local sr, sc = find_nearest_walkable(math.floor(state.rows / 2), math.floor(state.cols / 2))
        state.ghost_spawn, state.fruit.r, state.fruit.c = { r = sr, c = sc }, sr, sc
    end
end

local function randomize_fruit_spawn_for_level()
    local candidates = {}
    for r = 2, state.rows - 1 do
        for c = 2, state.cols - 1 do
            local ch = state.base_map[r][c]
            local walkable = (not is_wall(r, c)) and (not is_door(r, c))
            if walkable and ch ~= "<" and ch ~= ">" then
                local is_player_spawn = (r == state.player_start.r and c == state.player_start.c)
                local is_ghost_spawn = (r == state.ghost_spawn.r and c == state.ghost_spawn.c)
                if (not is_player_spawn) and (not is_ghost_spawn) then
                    candidates[#candidates + 1] = { r = r, c = c }
                end
            end
        end
    end

    if #candidates == 0 then return end

    local idx = 1
    if type(random) == "function" then
        idx = random(#candidates) + 1
    end
    local pick = candidates[idx] or candidates[1]
    state.fruit.r, state.fruit.c = pick.r, pick.c
end

local function load_best_score()
    state.best_score = 0
    if type(load_data) ~= "function" then return end
    local ok, v = pcall(load_data, "pacman_best_score")
    if not ok then return end
    if type(v) == "number" then state.best_score = math.floor(v)
    elseif type(v) == "table" and type(v.value) == "number" then state.best_score = math.floor(v.value) end
end

local function save_best_score()
    if type(save_data) == "function" then pcall(save_data, "pacman_best_score", { value = state.best_score }) end
end

local function commit_stats_once()
    if state.stats_committed then return end
    if state.score > state.best_score then state.best_score = state.score; save_best_score() end
    if type(update_game_stats) == "function" then pcall(update_game_stats, "pacman", state.score, elapsed_seconds()) end
    state.stats_committed = true
end

local function set_info_message(text, color, duration_sec)
    state.info_message, state.info_color = text, (color or "dark_gray")
    if duration_sec ~= nil and duration_sec > 0 then
        state.info_message_until = state.frame + math.floor(duration_sec * FPS)
    else
        state.info_message_until = nil
    end
end

local function start_round_countdown(seconds)
    local sec = math.max(0, math.floor(seconds or 0))
    state.countdown_until = state.frame + sec * FPS
    state.last_countdown_sec = nil
    if state.global_pause_until < state.countdown_until then
        state.global_pause_until = state.countdown_until
    end
    if type(clear_input_buffer) == "function" then
        pcall(clear_input_buffer)
    end
end

local function countdown_seconds_left()
    if state.phase ~= "playing" then return 0 end
    if state.countdown_until <= state.frame then return 0 end
    return math.max(1, math.ceil((state.countdown_until - state.frame) / FPS))
end

local function current_banner_message()
    if state.confirm_mode == "restart" then
        return tr("game.pacman.confirm_restart", "Confirm restart? [Y] Yes / [N] No"), "yellow"
    end
    if state.confirm_mode == "exit" then
        return tr("game.pacman.confirm_exit", "Confirm exit? [Y] Yes / [N] No"), "yellow"
    end

    local countdown = countdown_seconds_left()
    if countdown > 0 then
        return tr("game.pacman.countdown", "Starting in") .. " " .. tostring(countdown), "yellow"
    end

    if state.info_message == "" then
        return "", state.info_color or "dark_gray"
    end
    return state.info_message, state.info_color or "dark_gray"
end

local function reset_power_cycle()
    state.power_until, state.power_chain, state.power_eaten = 0, 0, {}
    for i = 1, #state.ghosts do if state.ghosts[i].state == "frightened" then state.ghosts[i].state = "normal" end end
end

local function activate_power_cycle()
    state.power_until = state.frame + power_duration_sec(state.level) * FPS
    state.power_chain, state.power_eaten = 0, {}
    for i = 1, #state.ghosts do
        local g = state.ghosts[i]
        state.power_eaten[g.id] = false
        if g.state ~= "eyes" and g.state ~= "house" then g.state = "frightened" end
    end
end

local function add_score(delta)
    if delta <= 0 then return end
    state.score = state.score + delta
    if (not state.extra_life_granted) and state.score >= EXTRA_LIFE_SCORE then state.extra_life_granted = true; state.lives = state.lives + 1 end
end

local function create_ghost(id, color, home_r, home_c)
    return {
        id = id, color = color,
        r = state.ghost_spawn.r, c = state.ghost_spawn.c, dir = "left",
        state = "house", release_at = state.frame, next_step_at = state.frame,
        home_r = home_r, home_c = home_c,
    }
end

local function reset_player_auto_direction()
    local order = { "left", "up", "right", "down" }
    for i = 1, #order do
        local dir = order[i]
        local dr, dc = direction_delta(dir)
        local can_move = can_move_player(state.player.r + dr, state.player.c + dc)
        if can_move then
            state.player.dir = dir
            state.player.next_dir = dir
            return
        end
    end
    state.player.dir = "left"
    state.player.next_dir = "left"
end

local function reset_entities_for_level()
    state.player.r, state.player.c = state.player_start.r, state.player_start.c
    state.player.next_step_at = state.frame
    reset_player_auto_direction()

    local top, left, right, bottom = 2, 2, state.cols - 1, state.rows - 1
    state.ghosts = {
        create_ghost("blinky", "red", top, right),
        create_ghost("pinky", "magenta", top, left),
        create_ghost("inky", "cyan", bottom, right),
        create_ghost("clyde", "rgb(255,165,0)", bottom, left),
    }

    local delays = ghost_release_delays(state.level)
    for i = 1, #state.ghosts do
        local g = state.ghosts[i]
        if g.id == "blinky" then g.state, g.release_at = "normal", state.frame
        else g.state, g.release_at = "house", state.frame + math.floor((delays[g.id] or 0) * FPS) end
    end

    state.global_pause_until = state.frame + FPS
    reset_power_cycle()
    state.fruit.active, state.fruit.spawned = false, false
end

local function start_level(level)
    state.level, state.level_start_frame = level, state.frame
    parse_map(); init_positions_from_map()
    state.total_pellets = pellet_counts(); state.remaining_pellets = state.total_pellets
    reset_entities_for_level()
    randomize_fruit_spawn_for_level()
    start_round_countdown(3)
    set_info_message(tr("game.pacman.status_ready", "Ready!"), "yellow")
    state.dirty = true
end

local function start_new_run()
    state.score, state.lives, state.extra_life_granted = 0, 3, false
    state.phase, state.run_start_frame, state.end_frame = "playing", state.frame, nil
    state.stats_committed = false
    state.confirm_mode = nil
    state.collected_fruits = {}
    start_level(1)
end

local function is_power_active() return state.power_until > state.frame end

local function update_power_state()
    if state.power_until > 0 and state.frame >= state.power_until then reset_power_cycle(); state.dirty = true end
end

local function spawn_fruit_if_needed()
    if state.fruit.spawned then return end
    if state.remaining_pellets <= math.floor(state.total_pellets * 0.7) then state.fruit.spawned = true; state.fruit.active = true; state.dirty = true end
end

local function consume_current_cell()
    local r, c = state.player.r, state.player.c
    local pellet = state.pellets[r][c]
    if pellet == PELLET_CHAR then
        state.pellets[r][c], state.remaining_pellets = " ", state.remaining_pellets - 1
        add_score(10); state.dirty = true
    elseif pellet == POWER_CHAR then
        state.pellets[r][c], state.remaining_pellets = " ", state.remaining_pellets - 1
        add_score(50); activate_power_cycle()
        set_info_message(tr("game.pacman.status_power", "Power pellet active!"), "cyan", 3)
        state.dirty = true
    end

    if state.fruit.active and r == state.fruit.r and c == state.fruit.c then
        local fruit = fruit_for_level(state.level)
        add_score(fruit.points); state.fruit.active = false
        state.collected_fruits[#state.collected_fruits + 1] = fruit.symbol
        set_info_message(tr("game.pacman.status_fruit", "Fruit collected!"), "magenta", 3)
        state.dirty = true
    end

    if state.remaining_pellets <= 0 then
        if state.level >= MAX_LEVEL then
            state.phase, state.end_frame = "won", state.frame
            commit_stats_once()
            set_info_message(tr("game.pacman.win_banner", "You collected all pellets!") .. " " .. tr("game.pacman.result_controls", "[R] Restart  [Q]/[ESC] Exit"), "green")
            state.dirty = true
        else
            start_level(state.level + 1)
            set_info_message(tr("game.pacman.status_level_clear", "Level clear!") .. " " .. tostring(state.level), "green")
        end
    end
end

local function try_move_player()
    if state.frame < state.player.next_step_at then return end
    state.player.next_step_at = state.frame + PLAYER_STEP_FRAMES

    local dr, dc = direction_delta(state.player.next_dir)
    local can_turn = can_move_player(state.player.r + dr, state.player.c + dc)
    if can_turn then state.player.dir = state.player.next_dir end

    local mdr, mdc = direction_delta(state.player.dir)
    local can_move, nr, nc = can_move_player(state.player.r + mdr, state.player.c + mdc)
    if can_move then state.player.r, state.player.c = nr, nc; consume_current_cell(); state.dirty = true end
end

local function walkable_for_bfs(r, c) return in_bounds(r, c) and (not is_wall(r, c)) end

local function bfs_distance(sr, sc, tr, tc)
    if sr == tr and sc == tc then return 0 end
    if not in_bounds(sr, sc) then return 9999 end
    local visited = blank_matrix(state.rows, state.cols, false)
    local dist = blank_matrix(state.rows, state.cols, -1)
    local q_r, q_c, head = { sr }, { sc }, 1
    visited[sr][sc], dist[sr][sc] = true, 0
    while head <= #q_r do
        local r, c = q_r[head], q_c[head]
        head = head + 1
        for i = 1, #DIRS do
            local d = DIRS[i]
            local nr, nc = apply_tunnel(r + d.dr, c + d.dc)
            if in_bounds(nr, nc) and (not visited[nr][nc]) and walkable_for_bfs(nr, nc) then
                visited[nr][nc], dist[nr][nc] = true, dist[r][c] + 1
                if nr == tr and nc == tc then return dist[nr][nc] end
                q_r[#q_r + 1], q_c[#q_c + 1] = nr, nc
            end
        end
    end
    return 9999
end

local function choose_step_towards(g, target_r, target_c)
    local candidates = {}
    for i = 1, #DIRS do
        local d = DIRS[i]
        local ok, nr, nc = can_move_ghost(g.r + d.dr, g.c + d.dc)
        if ok then candidates[#candidates + 1] = { dir = d.name, r = nr, c = nc } end
    end
    if #candidates == 0 then return nil end

    if #candidates > 1 then
        local opp = opposite_dir(g.dir)
        local filtered = {}
        for i = 1, #candidates do if candidates[i].dir ~= opp then filtered[#filtered + 1] = candidates[i] end end
        if #filtered > 0 then candidates = filtered end
    end

    if g.state == "frightened" and (not state.power_eaten[g.id]) then
        return candidates[random(#candidates) + 1]
    end

    local best, best_dist = nil, 9999
    for i = 1, #candidates do
        local cand = candidates[i]
        local d = bfs_distance(cand.r, cand.c, target_r, target_c)
        if d < best_dist then best_dist, best = d, cand end
    end
    return best or candidates[random(#candidates) + 1]
end

local function projected_player_pos(steps)
    local r, c = state.player.r, state.player.c
    local dr, dc = direction_delta(state.player.dir)
    for _ = 1, steps do
        local nr, nc = apply_tunnel(r + dr, c + dc)
        if not in_bounds(nr, nc) or is_wall(nr, nc) then break end
        r, c = nr, nc
    end
    return r, c
end

local function blinky_enraged()
    if state.level < 5 then return false end
    return state.remaining_pellets <= math.max(20, math.floor(state.total_pellets * 0.15))
end

local function ghost_target(g)
    if g.state == "eyes" then return state.ghost_spawn.r, state.ghost_spawn.c end

    local mode = current_chase_mode()
    if g.id == "blinky" and blinky_enraged() then mode = "chase" end
    if mode == "scatter" then return g.home_r, g.home_c end

    if g.id == "blinky" then
        return state.player.r, state.player.c
    elseif g.id == "pinky" then
        return projected_player_pos(4)
    elseif g.id == "inky" then
        local ar, ac = projected_player_pos(2)
        local br, bc = state.player.r, state.player.c
        for i = 1, #state.ghosts do
            if state.ghosts[i].id == "blinky" then
                br = state.ghosts[i].r
                bc = state.ghosts[i].c
                break
            end
        end
        local tx = clamp(ar + (ar - br), 2, state.rows - 1)
        local ty = clamp(ac + (ac - bc), 2, state.cols - 1)
        return tx, ty
    else
        local dist = math.abs(state.player.r - g.r) + math.abs(state.player.c - g.c)
        if dist > 8 then return state.player.r, state.player.c end
        return g.home_r, g.home_c
    end
end

local function ghost_step_interval(g)
    if g.state == "eyes" then
        return math.max(1, math.floor(4 * GHOST_SLOW_FACTOR + 0.5))
    end
    local base = (state.level >= 11 and 6) or (state.level >= 5 and 7) or 8
    if g.id == "blinky" and blinky_enraged() then base = math.max(4, base - 1) end
    if g.state == "frightened" and (not state.power_eaten[g.id]) then base = base + 3 end
    return math.max(1, math.floor(base * GHOST_SLOW_FACTOR + 0.5))
end

local function ghost_enter_house(g)
    g.r, g.c = state.ghost_spawn.r, state.ghost_spawn.c
    g.state = "house"
    g.release_at = state.frame + ghost_revive_sec(state.level) * FPS
    g.next_step_at = state.frame + FPS
end

local function eat_ghost(g)
    if g.state ~= "frightened" or state.power_eaten[g.id] then return false end
    local reward = { 200, 400, 800, 1600 }
    local idx = math.min(4, state.power_chain + 1)
    add_score(reward[idx])
    state.power_chain = state.power_chain + 1
    state.power_eaten[g.id] = true
    g.state = "eyes"
    g.next_step_at = state.frame
    set_info_message(tr("game.pacman.status_ghost_eaten", "Ghost eaten!"), "cyan", 3)
    state.dirty = true
    return true
end

local function reset_after_player_death()
    state.player.r, state.player.c = state.player_start.r, state.player_start.c
    state.player.next_step_at = state.frame
    reset_player_auto_direction()

    local delays = ghost_release_delays(state.level)
    for i = 1, #state.ghosts do
        local g = state.ghosts[i]
        g.r, g.c = state.ghost_spawn.r, state.ghost_spawn.c
        g.dir, g.state = "left", "house"
        g.release_at = state.frame + 4 * FPS + math.floor((delays[g.id] or 0) * FPS)
        g.next_step_at = g.release_at
    end

    state.global_pause_until = state.frame + 4 * FPS
    reset_power_cycle()
    set_info_message(tr("game.pacman.status_wait", "Ghosts retreating..."), "yellow")
    state.dirty = true
end

local function player_lost_life()
    state.lives = state.lives - 1
    if state.lives <= 0 then
        state.phase, state.end_frame = "lost", state.frame
        commit_stats_once()
        set_info_message(tr("game.pacman.lose_banner", "You lost all lives!") .. " " .. tr("game.pacman.result_controls", "[R] Restart  [Q]/[ESC] Exit"), "red")
    else
        reset_after_player_death()
    end
end

local function check_collision_with_ghost(g)
    if state.player.r ~= g.r or state.player.c ~= g.c then return end
    if g.state == "eyes" or g.state == "house" then return end
    if is_power_active() and g.state == "frightened" and (not state.power_eaten[g.id]) then
        eat_ghost(g)
    else
        player_lost_life()
    end
end

local function move_ghost(g)
    if g.state == "house" then
        if state.frame >= g.release_at and state.frame >= state.global_pause_until then
            g.state = (is_power_active() and (not state.power_eaten[g.id])) and "frightened" or "normal"
            g.next_step_at = state.frame
        end
        return
    end

    if state.frame < g.next_step_at or state.frame < state.global_pause_until then return end
    g.next_step_at = state.frame + ghost_step_interval(g)

    local target_r, target_c = ghost_target(g)
    local step = choose_step_towards(g, target_r, target_c)
    if step then
        g.r, g.c, g.dir = step.r, step.c, step.dir
        state.dirty = true
    end

    if g.state == "eyes" and g.r == state.ghost_spawn.r and g.c == state.ghost_spawn.c then
        ghost_enter_house(g)
    end
end

local function update_ghosts()
    if state.phase ~= "playing" then return end

    for i = 1, #state.ghosts do
        local g = state.ghosts[i]
        if g.state ~= "eyes" then
            if is_power_active() and (not state.power_eaten[g.id]) and g.state ~= "house" then
                g.state = "frightened"
            elseif g.state == "frightened" and ((not is_power_active()) or state.power_eaten[g.id]) then
                g.state = "normal"
            end
        end
    end

    for i = 1, #state.ghosts do
        move_ghost(state.ghosts[i])
        check_collision_with_ghost(state.ghosts[i])
        if state.phase ~= "playing" then return end
    end
end

local function update_player_and_collisions()
    if state.phase ~= "playing" or countdown_seconds_left() > 0 then return end
    try_move_player()
    for i = 1, #state.ghosts do
        check_collision_with_ghost(state.ghosts[i])
        if state.phase ~= "playing" then return end
    end
end

local function handle_confirm_input(key)
    if state.confirm_mode == nil then return false end
    if key == "y" then
        if state.confirm_mode == "restart" then
            start_new_run()
        else
            commit_stats_once()
            exit_game()
        end
        return true
    end
    if key == "n" or key == "q" or key == "esc" then
        state.confirm_mode = nil
        state.dirty = true
        return true
    end
    return true
end

local function handle_playing_input(key)
    if handle_confirm_input(key) then return end
    if key == "up" or key == "down" or key == "left" or key == "right" then state.player.next_dir = key; return end
    if key == "r" then state.confirm_mode = "restart"; state.dirty = true; return end
    if key == "q" or key == "esc" then state.confirm_mode = "exit"; state.dirty = true; return end
end

local function handle_result_input(key)
    if key == "r" then start_new_run(); return end
    if key == "q" or key == "esc" then commit_stats_once(); exit_game() end
end

local function ghost_at(r, c)
    for i = 1, #state.ghosts do
        local g = state.ghosts[i]
        if g.r == r and g.c == c then return g end
    end
    return nil
end

local function cell_visual(r, c)
    if state.player.r == r and state.player.c == c then
        return "@", (is_power_active() and "light_cyan" or "light_yellow")
    end

    local g = ghost_at(r, c)
    if g then
        if g.state == "eyes" then return "&", "white" end
        if g.state == "frightened" and (not state.power_eaten[g.id]) then return "&", "light_blue" end
        return "&", g.color
    end

    if state.fruit.active and r == state.fruit.r and c == state.fruit.c then
        return fruit_for_level(state.level).symbol, "magenta"
    end

    local pellet = state.pellets[r][c]
    if pellet == PELLET_CHAR then return PELLET_CHAR, "yellow" end
    if pellet == POWER_CHAR then return POWER_CHAR, "rgb(255,165,0)" end

    local ch = state.base_map[r][c]
    if ch == PELLET_CHAR or ch == POWER_CHAR then return " ", "white" end
    if WALL_SET[ch] then return ch, "blue" end
    if ch == DOOR_CHAR then return DOOR_CHAR, "white" end
    if ch == "<" or ch == ">" then return " ", "white" end
    return ch, "white"
end

local function centered_x(text, area_x, area_w)
    local x = area_x + math.floor((area_w - key_width(text)) / 2)
    if x < area_x then x = area_x end
    return x
end

local function fill_rect(x, y, w, h)
    local line = string.rep(" ", w)
    for i = 0, h - 1 do draw_text(x, y + i, line, "white", "black") end
end

local function collected_fruit_symbols()
    if #state.collected_fruits == 0 then
        return "-"
    end
    return table.concat(state.collected_fruits, " ")
end

local function build_info_width()
    local best_line = tr("game.pacman.best_score", "Best Score") .. ": " .. tostring(state.best_score)
    local score_line = tr("game.pacman.current_score", "Current Score") .. ": " .. tostring(state.score)
    local time_line = tr("game.pacman.game_time", "Game Time") .. ": " .. format_duration(elapsed_seconds())
    local level_line = tr("game.pacman.level", "Level") .. ": " .. tostring(state.level)
    local power_left = is_power_active() and math.max(0, math.ceil((state.power_until - state.frame) / FPS)) or 0
    local power_line = tr("game.pacman.power_left", "Power") .. " " .. tostring(power_left) .. tr("game.pacman.seconds_unit", "s")
    local lives_label = tr("game.pacman.lives", "Lives") .. ": "
    local lives_icons = string.rep("@", math.max(0, state.lives))
    if lives_icons == "" then lives_icons = "-" end
    local fruits_line = collected_fruit_symbols()

    local max_w = 18
    local candidates = { best_line, score_line, time_line, level_line, power_line, fruits_line, lives_label .. lives_icons }
    for i = 1, #candidates do
        local w = key_width(candidates[i])
        if w > max_w then max_w = w end
    end
    return max_w + 1
end

local function board_geometry()
    local term_w, term_h = terminal_size()
    local map_w, map_h = state.cols, state.rows
    local info_w = build_info_width()
    local gap = 4

    local content_w = map_w + gap + info_w
    local content_h = map_h

    local controls = tr("game.pacman.controls", "[?]/[?]/[?]/[?] Move  [R] Restart  [Q]/[ESC] Exit")
    local controls_w = min_width_for_lines(controls, 3, 24)
    local result_w = key_width(tr("game.pacman.lose_banner", "You lost all lives!") .. " " .. tr("game.pacman.result_controls", "[R] Restart  [Q]/[ESC] Exit"))
    local confirm_w = math.max(
        key_width(tr("game.pacman.confirm_restart", "Confirm restart? [Y] Yes / [N] No")),
        key_width(tr("game.pacman.confirm_exit", "Confirm exit? [Y] Yes / [N] No"))
    )
    local countdown_w = key_width(tr("game.pacman.countdown", "Starting in") .. " 3")

    local total_w = math.max(content_w, controls_w, result_w, confirm_w, countdown_w)
    local total_h = content_h + 1 + 3

    local x = math.floor((term_w - total_w) / 2) + 1
    local y = math.floor((term_h - total_h) / 2) + 1
    if x < 1 then x = 1 end
    if y < 1 then y = 1 end

    local map_x = x
    local map_y = y
    local info_x = map_x + map_w + gap
    if info_x + info_w - 1 > x + total_w - 1 then info_x = x + total_w - info_w end

    local mid_y = map_y + math.floor(map_h / 2) - 1
    if mid_y < map_y then mid_y = map_y end
    if mid_y > map_y + map_h - 3 then mid_y = map_y + map_h - 3 end

    local fruits_y = mid_y + 5
    if fruits_y > map_y + map_h - 1 then fruits_y = map_y + map_h - 1 end

    return {
        x = x, y = y, total_w = total_w, total_h = total_h,
        map_x = map_x, map_y = map_y, map_w = map_w, map_h = map_h,
        info_x = info_x, info_y = map_y, info_w = info_w,
        info_mid_y = mid_y, info_fruits_y = fruits_y,
        message_y = y + content_h,
        controls_y = y + content_h + 1,
    }
end

local function draw_map(layout)
    for r = 1, state.rows do
        for c = 1, state.cols do
            local ch, fg = cell_visual(r, c)
            draw_text(layout.map_x + c - 1, layout.map_y + r - 1, ch, fg, "black")
        end
    end
end

local function draw_info(layout)
    fill_rect(layout.info_x, layout.info_y, layout.info_w, layout.map_h)

    local top_y = layout.info_y
    local mid_y = layout.info_mid_y

    local best_line = tr("game.pacman.best_score", "Best Score") .. ": " .. tostring(state.best_score)
    local score_line = tr("game.pacman.current_score", "Current Score") .. ": " .. tostring(state.score)
    local time_line = tr("game.pacman.game_time", "Game Time") .. ": " .. format_duration(elapsed_seconds())

    draw_text(layout.info_x, top_y, best_line, "dark_gray", "black")
    draw_text(layout.info_x, top_y + 1, score_line, "white", "black")
    draw_text(layout.info_x, top_y + 2, time_line, "light_cyan", "black")

    local level_line = tr("game.pacman.level", "Level") .. ": " .. tostring(state.level)
    draw_text(layout.info_x, mid_y, level_line, "white", "black")

    local lives_label = tr("game.pacman.lives", "Lives") .. ": "
    local lives_icons = string.rep("@", math.max(0, state.lives))
    if lives_icons == "" then lives_icons = "-" end
    draw_text(layout.info_x, mid_y + 1, lives_label, "white", "black")
    draw_text(layout.info_x + key_width(lives_label), mid_y + 1, lives_icons, "yellow", "black")

    local remain = is_power_active() and math.max(0, math.ceil((state.power_until - state.frame) / FPS)) or 0
    local power_line = tr("game.pacman.power_left", "Power") .. " " .. tostring(remain) .. tr("game.pacman.seconds_unit", "s")
    draw_text(layout.info_x, mid_y + 2, power_line, "white", "black")

    draw_text(layout.info_x, layout.info_fruits_y, collected_fruit_symbols(), "light_magenta", "black")
end

local function draw_message(layout)
    local term_w, _ = terminal_size()
    draw_text(1, layout.message_y, string.rep(" ", term_w), "white", "black")

    local msg, color = current_banner_message()
    if msg ~= "" then
        draw_text(centered_x(msg, 1, term_w), layout.message_y, msg, color or "dark_gray", "black")
    end
end

local function draw_controls(layout)
    local controls = tr("game.pacman.controls", "[?]/[?]/[?]/[?] Move  [R] Restart  [Q]/[ESC] Exit")
    local term_w, _ = terminal_size()
    local lines = wrap_words(controls, math.max(10, term_w - 2))
    if #lines > 3 then lines = { lines[1], lines[2], lines[3] } end

    for i = 0, 2 do draw_text(1, layout.controls_y + i, string.rep(" ", term_w), "white", "black") end
    local offset = (#lines < 3) and math.floor((3 - #lines) / 2) or 0
    for i = 1, #lines do draw_text(centered_x(lines[i], 1, term_w), layout.controls_y + offset + i - 1, lines[i], "white", "black") end
end

local function clear_last_area_if_needed(layout)
    local area = { x = layout.x, y = layout.y, w = layout.total_w, h = layout.total_h }
    if state.last_area == nil then
        fill_rect(area.x, area.y, area.w, area.h)
    elseif state.last_area.x ~= area.x or state.last_area.y ~= area.y or state.last_area.w ~= area.w or state.last_area.h ~= area.h then
        fill_rect(state.last_area.x, state.last_area.y, state.last_area.w, state.last_area.h)
        fill_rect(area.x, area.y, area.w, area.h)
    end
    state.last_area = area
end

local function render()
    local layout = board_geometry()
    clear_last_area_if_needed(layout)
    draw_map(layout)
    draw_info(layout)
    draw_message(layout)
    draw_controls(layout)
end

local function minimum_required_size()
    local map_w, map_h = state.cols, state.rows
    local info_w = build_info_width()
    local content_w = map_w + 4 + info_w

    local controls_text = tr("game.pacman.controls", "[?]/[?]/[?]/[?] Move  [R] Restart  [Q]/[ESC] Exit")
    local controls_w = min_width_for_lines(controls_text, 3, 24)
    local result_w = key_width(tr("game.pacman.lose_banner", "You lost all lives!") .. " " .. tr("game.pacman.result_controls", "[R] Restart  [Q]/[ESC] Exit"))
    local confirm_w = math.max(
        key_width(tr("game.pacman.confirm_restart", "Confirm restart? [Y] Yes / [N] No")),
        key_width(tr("game.pacman.confirm_exit", "Confirm exit? [Y] Yes / [N] No"))
    )

    local min_w = math.max(content_w, controls_w, result_w, confirm_w) + 2
    local min_h = map_h + 1 + 3 + 2
    return min_w, min_h
end
local function draw_terminal_size_warning(term_w, term_h, min_w, min_h)
    clear()
    local lines = {
        tr("warning.size_title", "Terminal Too Small"),
        string.format("%s: %dx%d", tr("warning.required", "Required size"), min_w, min_h),
        string.format("%s: %dx%d", tr("warning.current", "Current size"), term_w, term_h),
        tr("warning.enlarge_hint", "Please enlarge terminal window to continue."),
    }
    local top = math.floor((term_h - #lines) / 2)
    if top < 1 then top = 1 end
    for i = 1, #lines do
        local x = math.floor((term_w - key_width(lines[i])) / 2)
        if x < 1 then x = 1 end
        draw_text(x, top + i - 1, lines[i], "white", "black")
    end
end

local function ensure_terminal_size_ok()
    local term_w, term_h = terminal_size()
    local min_w, min_h = minimum_required_size()
    if term_w >= min_w and term_h >= min_h then
        if state.size_warning_active then clear(); state.last_area = nil; state.dirty = true end
        state.size_warning_active = false
        return true
    end

    local changed = (not state.size_warning_active)
        or state.last_warn_term_w ~= term_w
        or state.last_warn_term_h ~= term_h
        or state.last_warn_min_w ~= min_w
        or state.last_warn_min_h ~= min_h
    if changed then
        draw_terminal_size_warning(term_w, term_h, min_w, min_h)
        state.last_warn_term_w, state.last_warn_term_h = term_w, term_h
        state.last_warn_min_w, state.last_warn_min_h = min_w, min_h
    end
    state.size_warning_active = true
    return false
end

local function sync_resize()
    local w, h = terminal_size()
    if w ~= state.last_term_w or h ~= state.last_term_h then
        state.last_term_w, state.last_term_h = w, h
        clear(); state.last_area = nil; state.dirty = true
    end
end

local function init_game()
    local w, h = terminal_size()
    state.last_term_w, state.last_term_h = w, h
    clear()
    load_best_score()
    parse_map(); init_positions_from_map(); start_new_run()
    if type(clear_input_buffer) == "function" then pcall(clear_input_buffer) end
end

local function update_logic(key)
    if state.phase == "playing" then handle_playing_input(key) else handle_result_input(key) end

    local countdown = countdown_seconds_left()
    if countdown > 0 then
        if state.last_countdown_sec ~= countdown then
            state.last_countdown_sec = countdown
            state.dirty = true
        end
    elseif state.last_countdown_sec ~= nil then
        state.last_countdown_sec = nil
        state.dirty = true
    end

    if state.info_message_until ~= nil and state.frame >= state.info_message_until then
        state.info_message = ""
        state.info_message_until = nil
        state.dirty = true
    end

    if state.confirm_mode ~= nil then return end
    if state.phase ~= "playing" then return end

    update_power_state(); spawn_fruit_if_needed(); update_player_and_collisions(); update_ghosts()
end

local function game_loop()
    while true do
        local key = normalize_key(get_key(false))
        if ensure_terminal_size_ok() then
            update_logic(key)
            sync_resize()
            if state.dirty then render(); state.dirty = false end
            state.frame = state.frame + 1
        else
            if key == "q" or key == "esc" then commit_stats_once(); exit_game() end
        end
        sleep(FRAME_MS)
    end
end

init_game()
game_loop()

