-- 2048 for TUI GAME
GAME_META = {
    name = "2048",
    description = "Merge equal tiles to reach 131072!"
}

local SIZE = 4
local TARGET_TILE = 131072
local MAX_TILE = 2147483647
local FPS = 60
local FRAME_MS = 16
local CELL_W = 8
local CELL_H = 4

local BORDER_TL = "\u{2554}"
local BORDER_TR = "\u{2557}"
local BORDER_BL = "\u{255A}"
local BORDER_BR = "\u{255D}"
local BORDER_H = "\u{2550}"
local BORDER_V = "\u{2551}"

local state = {
    board = {},
    score = 0,
    game_over = false,
    won = false,
    confirm_mode = nil,
    frame = 0,
    start_frame = 0,
    win_message_until = 0,
    last_auto_save_sec = 0,
    toast_text = nil,
    toast_until = 0,
    dirty = true,
    last_elapsed_sec = -1,
    last_win_visible = false,
    last_toast_visible = false,
    last_key = "",
    last_key_frame = -100,
    launch_mode = "new",
    last_area = nil,
    end_frame = nil,
    last_term_w = 0,
    last_term_h = 0,
    size_warning_active = false,
    last_warn_term_w = 0,
    last_warn_term_h = 0,
    last_warn_min_w = 0,
    last_warn_min_h = 0,
    best_score = 0,
    best_time_sec = 0
}

local function tr(key, fallback)
    if type(translate) ~= "function" then
        return fallback
    end
    local ok, value = pcall(translate, key)
    if not ok or value == nil or value == "" or value == key then
        return fallback
    end
    return value
end

local function deep_copy_board(board)
    local out = {}
    for r = 1, SIZE do
        out[r] = {}
        for c = 1, SIZE do
            out[r][c] = board[r][c]
        end
    end
    return out
end

local function init_empty_board()
    local board = {}
    for r = 1, SIZE do
        board[r] = {}
        for c = 1, SIZE do
            board[r][c] = 0
        end
    end
    return board
end

local function random_tile_value()
    if random(10) == 0 then
        return 4
    end
    return 2
end

local function list_empty_cells(board)
    local cells = {}
    for r = 1, SIZE do
        for c = 1, SIZE do
            if board[r][c] == 0 then
                cells[#cells + 1] = { r = r, c = c }
            end
        end
    end
    return cells
end

local function spawn_tile(board)
    local empty = list_empty_cells(board)
    if #empty == 0 then
        return false
    end
    local pick = empty[random(#empty) + 1]
    if pick == nil then
        return false
    end
    board[pick.r][pick.c] = random_tile_value()
    return true
end

local function normalize_key(key)
    if key == nil then
        return ""
    end
    if type(key) == "string" then
        return string.lower(key)
    end
    if type(key) == "table" and type(key.code) == "string" then
        return string.lower(key.code)
    end
    return tostring(key):lower()
end

local function format_duration(sec)
    local h = math.floor(sec / 3600)
    local m = math.floor((sec % 3600) / 60)
    local s = sec % 60
    return string.format("%02d:%02d:%02d", h, m, s)
end

local function format_cell_value(v)
    if v == 0 then
        return "."
    end
    local text = tostring(v)
    if #text > 4 then
        if v >= 1000000000 then
            text = tostring(math.floor(v / 1000000000)) .. "g"
        elseif v >= 1000000 then
            text = tostring(math.floor(v / 1000000)) .. "m"
        elseif v >= 1000 then
            text = tostring(math.floor(v / 1000)) .. "k"
        end
    end
    if #text > 4 then
        text = string.sub(text, 1, 4)
    end
    return text
end

local function tile_bg_color(v)
    if v == 0 then return "rgb(90,90,90)" end
    if v == 2 then return "rgb(255,255,255)" end
    if v == 4 then return "rgb(255,229,229)" end
    if v == 8 then return "rgb(255,204,204)" end
    if v == 16 then return "rgb(255,178,178)" end
    if v == 32 then return "rgb(255,153,153)" end
    if v == 64 then return "rgb(255,127,127)" end
    if v == 128 then return "rgb(255,102,102)" end
    if v == 256 then return "rgb(255,76,76)" end
    if v == 512 then return "rgb(255,50,50)" end
    if v == 1024 then return "rgb(255,25,25)" end
    if v == 2048 then return "rgb(255,0,0)" end
    if v == 4096 then return "rgb(212,0,0)" end
    if v == 8192 then return "rgb(170,0,0)" end
    if v == 16384 then return "rgb(127,0,0)" end
    if v == 32768 then return "rgb(85,0,0)" end
    if v == 65536 then return "rgb(42,0,0)" end
    return "rgb(0,0,0)"
end

local function text_width(text)
    if type(get_text_width) == "function" then
        local ok, w = pcall(get_text_width, text)
        if ok and type(w) == "number" then
            return w
        end
    end
    return #text
end

local function wrap_words(text, max_width)
    if max_width <= 1 then
        return { text }
    end
    local lines = {}
    local current = ""
    local had_token = false

    for token in string.gmatch(text, "%S+") do
        had_token = true
        if current == "" then
            current = token
        else
            local candidate = current .. " " .. token
            if text_width(candidate) <= max_width then
                current = candidate
            else
                lines[#lines + 1] = current
                current = token
            end
        end
    end

    if not had_token then
        return { "" }
    end
    if current ~= "" then
        lines[#lines + 1] = current
    end
    return lines
end

local function min_width_for_lines(text, max_lines, hard_min)
    local full = text_width(text)
    local width = hard_min
    while width <= full do
        if #wrap_words(text, width) <= max_lines then
            return width
        end
        width = width + 1
    end
    return full
end

local function text_color_for_value(v)
    if v == 0 then
        return "black"
    end
    if v <= 2048 then
        return "black"
    end
    return "white"
end

local function merge_line(values)
    local compact = {}
    for i = 1, #values do
        if values[i] ~= 0 then
            compact[#compact + 1] = values[i]
        end
    end

    local out = {}
    local gained = 0
    local i = 1
    while i <= #compact do
        if i < #compact and compact[i] == compact[i + 1] then
            local merged = compact[i] * 2
            if merged > MAX_TILE then merged = MAX_TILE end
            out[#out + 1] = merged
            gained = gained + merged
            i = i + 2
        else
            out[#out + 1] = compact[i]
            i = i + 1
        end
    end

    while #out < SIZE do
        out[#out + 1] = 0
    end
    return out, gained
end

local function get_row(board, r)
    local line = {}
    for c = 1, SIZE do line[c] = board[r][c] end
    return line
end

local function set_row(board, r, line)
    for c = 1, SIZE do board[r][c] = line[c] end
end

local function get_col(board, c)
    local line = {}
    for r = 1, SIZE do line[r] = board[r][c] end
    return line
end

local function set_col(board, c, line)
    for r = 1, SIZE do board[r][c] = line[r] end
end

local function reverse_line(line)
    local out = {}
    for i = 1, SIZE do out[i] = line[SIZE - i + 1] end
    return out
end

local function lines_equal(a, b)
    for i = 1, SIZE do
        if a[i] ~= b[i] then return false end
    end
    return true
end

local function apply_move(dir)
    local moved = false
    local gained = 0

    if dir == "left" or dir == "right" then
        for r = 1, SIZE do
            local old = get_row(state.board, r)
            local line = old
            local gained_line = 0
            if dir == "right" then line = reverse_line(line) end
            line, gained_line = merge_line(line)
            if dir == "right" then line = reverse_line(line) end
            set_row(state.board, r, line)
            if not lines_equal(old, line) then moved = true end
            gained = gained + gained_line
        end
    else
        for c = 1, SIZE do
            local old = get_col(state.board, c)
            local line = old
            local gained_line = 0
            if dir == "down" then line = reverse_line(line) end
            line, gained_line = merge_line(line)
            if dir == "down" then line = reverse_line(line) end
            set_col(state.board, c, line)
            if not lines_equal(old, line) then moved = true end
            gained = gained + gained_line
        end
    end

    if moved then state.score = state.score + gained end
    return moved
end

local function can_move_any()
    if #list_empty_cells(state.board) > 0 then return true end
    for r = 1, SIZE do
        for c = 1, SIZE do
            local v = state.board[r][c]
            if r < SIZE and state.board[r + 1][c] == v then return true end
            if c < SIZE and state.board[r][c + 1] == v then return true end
        end
    end
    return false
end

local function update_win_and_loss()
    local was_won = state.won
    state.won = false
    for r = 1, SIZE do
        for c = 1, SIZE do
            if state.board[r][c] >= TARGET_TILE then
                state.won = true
                state.win_message_until = state.frame + 3 * FPS
                if not was_won then
                    state.end_frame = state.frame
                    commit_stats()
                end
                return
            end
        end
    end
end

local function make_snapshot()
    return {
        board = deep_copy_board(state.board),
        score = state.score,
        elapsed_sec = math.floor((state.frame - state.start_frame) / FPS)
    }
end

local function elapsed_seconds()
    local end_frame = state.end_frame
    if end_frame == nil then
        end_frame = state.frame
    end
    return math.floor((end_frame - state.start_frame) / FPS)
end

local function commit_stats()
    local score = tonumber(state.score) or 0
    local duration = elapsed_seconds()
    if score > state.best_score or (score == state.best_score and score > 0 and (state.best_time_sec == 0 or duration < state.best_time_sec)) then
        state.best_score = score
        state.best_time_sec = duration
        if type(save_data) == "function" then
            pcall(save_data, "2048_best", { score = state.best_score, time_sec = state.best_time_sec })
        end
    end

    if type(update_game_stats) ~= "function" then
        return
    end
    pcall(update_game_stats, "2048", score, duration)
end

local function load_best_record()
    local data = nil
    if type(load_data) == "function" then
        local ok, ret = pcall(load_data, "2048_best")
        if ok and type(ret) == "table" then
            data = ret
        end
    end

    if data == nil then
        state.best_score = 0
        state.best_time_sec = 0
        return
    end

    state.best_score = math.max(0, math.floor(tonumber(data.score) or 0))
    state.best_time_sec = math.max(0, math.floor(tonumber(data.time_sec) or 0))
end

local function restore_snapshot(snapshot)
    if type(snapshot) ~= "table" or type(snapshot.board) ~= "table" then
        return false
    end
    local board = init_empty_board()
    for r = 1, SIZE do
        if type(snapshot.board[r]) ~= "table" then
            return false
        end
        for c = 1, SIZE do
            board[r][c] = tonumber(snapshot.board[r][c]) or 0
        end
    end

    state.board = board
    state.score = tonumber(snapshot.score) or 0
    local elapsed = tonumber(snapshot.elapsed_sec) or 0
    state.start_frame = state.frame - math.floor(elapsed * FPS)
    state.last_auto_save_sec = elapsed
    state.game_over = false
    state.won = false
    state.confirm_mode = nil
    state.win_message_until = 0
    state.toast_text = nil
    state.toast_until = 0
    state.end_frame = nil
    state.dirty = true
    return true
end

local function save_game_state(show_toast)
    local ok = false
    local snapshot = make_snapshot()
    if type(save_game_slot) == "function" then
        local s, ret = pcall(save_game_slot, "2048", snapshot)
        ok = s and ret ~= false
    elseif type(save_data) == "function" then
        local s, ret = pcall(save_data, "2048", snapshot)
        ok = s and ret ~= false
    elseif type(save_game) == "function" then
        local s, ret = pcall(save_game, snapshot)
        ok = s and ret ~= false
    end

    if show_toast then
        local key = ok and "game.2048.save_success" or "game.2048.save_unavailable"
        local def = ok and "Save successful!" or "Save API unavailable."
        state.toast_text = tr(key, def)
        state.toast_until = state.frame + 2 * FPS
        state.dirty = true
    end
end

local function load_game_state()
    local ok = false
    local snapshot = nil
    if type(load_game_slot) == "function" then
        local s, ret = pcall(load_game_slot, "2048")
        ok = s and ret ~= nil
        snapshot = ret
    elseif type(load_data) == "function" then
        local s, ret = pcall(load_data, "2048")
        ok = s and ret ~= nil
        snapshot = ret
    elseif type(load_game) == "function" then
        local s, ret = pcall(load_game)
        ok = s and ret ~= nil
        snapshot = ret
    end
    if ok then
        return restore_snapshot(snapshot)
    end
    return false
end

local function reset_game()
    state.board = init_empty_board()
    state.score = 0
    state.game_over = false
    state.won = false
    state.confirm_mode = nil
    state.start_frame = state.frame
    state.last_auto_save_sec = 0
    state.toast_text = nil
    state.toast_until = 0
    state.win_message_until = 0
    state.end_frame = nil
    spawn_tile(state.board)
    spawn_tile(state.board)
    state.dirty = true
end

local function read_launch_mode()
    if type(get_launch_mode) ~= "function" then
        return "new"
    end
    local ok, mode = pcall(get_launch_mode)
    if not ok or type(mode) ~= "string" then
        return "new"
    end
    mode = string.lower(mode)
    if mode == "continue" then
        return "continue"
    end
    return "new"
end

local function board_geometry()
    local w, h = 120, 40
    if type(get_terminal_size) == "function" then
        local tw, th = get_terminal_size()
        if type(tw) == "number" and type(th) == "number" then w, h = tw, th end
    end

    local grid_w = SIZE * CELL_W
    local grid_h = SIZE * CELL_H
    local status_w = text_width(tr("game.2048.time", "Time") .. " 00:00:00")
        + 2
        + text_width(tr("game.2048.score", "Score") .. " 999999999")
    local best_w = text_width(
        tr("game.2048.best_title", "Best")
            .. "  "
            .. tr("game.2048.best_score", "Score")
            .. " "
            .. tostring(math.max(0, state.best_score))
            .. "  "
            .. tr("game.2048.best_time", "Time")
            .. " "
            .. format_duration(math.max(0, state.best_time_sec))
    )
    local frame_w = math.max(grid_w, status_w, best_w) + 2
    local frame_h = grid_h + 2

    local x = math.floor((w - frame_w) / 2)
    local y = math.floor((h - frame_h) / 2)
    if x < 1 then x = 1 end
    if y < 5 then y = 5 end
    return x, y, frame_w, frame_h
end

local function terminal_size()
    local w, h = 120, 40
    if type(get_terminal_size) == "function" then
        local tw, th = get_terminal_size()
        if type(tw) == "number" and type(th) == "number" then
            w, h = tw, th
        end
    end
    return w, h
end

local function fill_rect(x, y, w, h, bg)
    if w <= 0 or h <= 0 then return end
    local line = string.rep(" ", w)
    for row = 0, h - 1 do
        draw_text(x, y + row, line, "white", bg or "black")
    end
end

local function draw_outer_frame(x, y, frame_w, frame_h)
    draw_text(x, y, BORDER_TL .. string.rep(BORDER_H, frame_w - 2) .. BORDER_TR, "white", "black")
    for i = 1, frame_h - 2 do
        draw_text(x, y + i, BORDER_V, "white", "black")
        draw_text(x + frame_w - 1, y + i, BORDER_V, "white", "black")
    end
    draw_text(x, y + frame_h - 1, BORDER_BL .. string.rep(BORDER_H, frame_w - 2) .. BORDER_BR, "white", "black")
end

local function draw_tile(tile_x, tile_y, value)
    local bg = tile_bg_color(value)
    local fg = text_color_for_value(value)

    for row = 0, CELL_H - 1 do
        draw_text(tile_x, tile_y + row, string.rep(" ", CELL_W), fg, bg)
    end

    local text = format_cell_value(value)
    local text_x = tile_x + math.floor((CELL_W - #text) / 2)
    local text_y = tile_y + math.floor(CELL_H / 2)
    draw_text(text_x, text_y, text, fg, bg)
end

local function draw_status(x, y, frame_w)
    local elapsed = elapsed_seconds()
    local left = tr("game.2048.time", "Time") .. " " .. format_duration(elapsed)
    local right = tr("game.2048.score", "Score") .. " " .. tostring(state.score)
    local term_w = terminal_size()
    local right_x = x + frame_w - text_width(right)
    if right_x < 1 then right_x = 1 end

    draw_text(1, y - 3, string.rep(" ", term_w), "white", "black")
    draw_text(1, y - 2, string.rep(" ", term_w), "white", "black")
    draw_text(1, y - 1, string.rep(" ", term_w), "white", "black")
    local best_line = tr("game.2048.best_title", "Best")
        .. "  "
        .. tr("game.2048.best_score", "Score")
        .. " "
        .. tostring(math.max(0, state.best_score))
        .. "  "
        .. tr("game.2048.best_time", "Time")
        .. " "
        .. format_duration(math.max(0, state.best_time_sec))
    draw_text(x, y - 3, best_line, "dark_gray", "black")
    draw_text(x, y - 2, left, "light_cyan", "black")
    draw_text(right_x, y - 2, right, "light_cyan", "black")

    if state.won then
        local line = tr("game.2048.win_banner", "Beyond machine limits!")
            .. tr("game.2048.win_controls", "[R] Restart  [Q]/[ESC] Exit")
        draw_text(x, y - 1, line, "yellow", "black")
    elseif state.confirm_mode == "game_over" then
        draw_text(x, y - 1, tr("game.2048.game_over", "Game over! [Y] Restart, [N] Back to game list."), "red", "black")
    elseif state.confirm_mode == "restart" then
        draw_text(x, y - 1, tr("game.2048.confirm_restart", "Confirm restart? [Y] Yes / [N] No"), "yellow", "black")
    elseif state.confirm_mode == "exit" then
        draw_text(x, y - 1, tr("game.2048.confirm_exit", "Confirm exit? [Y] Yes / [N] No"), "yellow", "black")
    elseif state.toast_text ~= nil and state.frame <= state.toast_until then
        draw_text(x, y - 1, state.toast_text, "green", "black")
    end
end

local function draw_controls(x, y, frame_h, frame_w)
    local term_w = terminal_size()
    local controls = tr("game.2048.controls", "[↑]/[↓]/[←]/[→] Move  [R] Restart  [S] Save  [Q]/[ESC] Exit")
    local max_w = math.max(10, term_w - 2)
    local lines = wrap_words(controls, max_w)
    if #lines > 3 then
        lines = { lines[1], lines[2], lines[3] }
    end

    draw_text(1, y + frame_h + 1, string.rep(" ", term_w), "white", "black")
    draw_text(1, y + frame_h + 2, string.rep(" ", term_w), "white", "black")
    draw_text(1, y + frame_h + 3, string.rep(" ", term_w), "white", "black")

    local offset = 0
    if #lines < 3 then
        offset = math.floor((3 - #lines) / 2)
    end
    for i = 1, #lines do
        local line = lines[i]
        local controls_x = math.floor((term_w - text_width(line)) / 2)
        if controls_x < 1 then controls_x = 1 end
        draw_text(controls_x, y + frame_h + 1 + offset + i - 1, line, "white", "black")
    end
end

local function clear_last_area()
    if state.last_area == nil then
        return
    end
    fill_rect(state.last_area.x, state.last_area.y, state.last_area.w, state.last_area.h, "black")
end

local function render()
    local x, y, frame_w, frame_h = board_geometry()
    local area = { x = x, y = y - 3, w = frame_w, h = frame_h + 7 }

    if state.last_area == nil then
        fill_rect(area.x, area.y, area.w, area.h, "black")
    elseif state.last_area.x ~= area.x or state.last_area.y ~= area.y or
        state.last_area.w ~= area.w or state.last_area.h ~= area.h then
        clear_last_area()
        fill_rect(area.x, area.y, area.w, area.h, "black")
    end
    state.last_area = area

    draw_status(x, y, frame_w)
    draw_outer_frame(x, y, frame_w, frame_h)

    local pad_x = math.floor((frame_w - 2 - SIZE * CELL_W) / 2)
    if pad_x < 0 then pad_x = 0 end
    local inner_x = x + 1 + pad_x
    local inner_y = y + 1
    for r = 1, SIZE do
        for c = 1, SIZE do
            local tx = inner_x + (c - 1) * CELL_W
            local ty = inner_y + (r - 1) * CELL_H
            draw_tile(tx, ty, state.board[r][c])
        end
    end

    draw_controls(x, y, frame_h, frame_w)
end

local function apply_direction_key(key)
    if key == "up" or key == "down" or key == "left" or key == "right" then
        return key
    end
    return nil
end

local function is_move_key(key)
    return key == "up" or key == "down" or key == "left" or key == "right"
end

local function should_debounce(key)
    if not is_move_key(key) then
        return false
    end
    if key == state.last_key and (state.frame - state.last_key_frame) <= 2 then
        return true
    end
    state.last_key = key
    state.last_key_frame = state.frame
    return false
end

local function handle_confirm_key(key)
    if key == "y" or key == "enter" then
        if state.confirm_mode == "game_over" then
            reset_game()
            return "changed"
        end
        if state.confirm_mode == "restart" then
            reset_game()
            return "changed"
        end
        if state.confirm_mode == "exit" then
            commit_stats()
            return "exit"
        end
    end

    if state.confirm_mode == "game_over" and key == "n" then
        commit_stats()
        return "exit"
    end

    if state.confirm_mode == "game_over" then
        return "none"
    end

    if key == "n" or key == "q" or key == "esc" then
        state.confirm_mode = nil
        state.dirty = true
        return "changed"
    end
    return "none"
end

local function reconcile_game_over_state()
    if state.confirm_mode == "game_over" and can_move_any() then
        state.game_over = false
        state.confirm_mode = nil
        state.end_frame = nil
        state.dirty = true
    end
end

local function handle_input(key)
    if key == nil or key == "" then
        return "none"
    end
    if should_debounce(key) then
        return "none"
    end

    reconcile_game_over_state()

    if state.confirm_mode ~= nil then
        return handle_confirm_key(key)
    end

    if state.won then
        if key == "r" then
            reset_game()
            return "changed"
        end
        if key == "q" or key == "esc" then
            commit_stats()
            return "exit"
        end
        return "none"
    end

    if key == "r" then
        state.confirm_mode = "restart"
        state.dirty = true
        return "changed"
    end
    if key == "q" or key == "esc" then
        state.confirm_mode = "exit"
        state.dirty = true
        return "changed"
    end
    if key == "s" then
        save_game_state(true)
        return "changed"
    end

    if state.game_over then
        return "none"
    end

    local dir = apply_direction_key(key)
    if dir == nil then
        return "none"
    end

    local moved = apply_move(dir)
    if moved then
        spawn_tile(state.board)
        update_win_and_loss()
        state.dirty = true
        return "changed"
    end

    if not can_move_any() and not state.game_over then
        state.game_over = true
        state.confirm_mode = "game_over"
        state.end_frame = state.frame
        state.dirty = true
        commit_stats()
        return "changed"
    end

    return "none"
end

local function auto_save_if_needed()
    local elapsed = elapsed_seconds()
    if elapsed - state.last_auto_save_sec >= 60 then
        save_game_state(false)
        state.last_auto_save_sec = elapsed
    end
end

local function refresh_dirty_flags()
    local elapsed = math.floor((state.frame - state.start_frame) / FPS)
    if elapsed ~= state.last_elapsed_sec then
        state.last_elapsed_sec = elapsed
        state.dirty = true
    end

    local win_visible = state.frame <= state.win_message_until
    if win_visible ~= state.last_win_visible then
        state.last_win_visible = win_visible
        state.dirty = true
    end

    local toast_visible = state.toast_text ~= nil and state.frame <= state.toast_until
    if toast_visible ~= state.last_toast_visible then
        state.last_toast_visible = toast_visible
        state.dirty = true
    end
end

local function sync_terminal_resize()
    local w, h = terminal_size()
    if w ~= state.last_term_w or h ~= state.last_term_h then
        state.last_term_w = w
        state.last_term_h = h
        clear()
        state.last_area = nil
        state.dirty = true
    end
end

local function minimum_required_size()
    local frame_w = SIZE * CELL_W + 2
    local frame_h = SIZE * CELL_H + 2

    local controls_w = min_width_for_lines(
        tr("game.2048.controls", "[↑]/[↓]/[←]/[→] Move  [R] Restart  [S] Save  [Q]/[ESC] Exit"),
        3,
        24
    )
    local status_w = text_width(tr("game.2048.time", "Time") .. " 00:00:00")
        + 2
        + text_width(tr("game.2048.score", "Score") .. " 999999999")
    local best_w = text_width(
        tr("game.2048.best_title", "Best")
            .. "  "
            .. tr("game.2048.best_score", "Score")
            .. " 999999999  "
            .. tr("game.2048.best_time", "Time")
            .. " 00:00:00"
    )
    local win_line_w = text_width(
        tr("game.2048.win_banner", "Beyond machine limits!")
            .. tr("game.2048.win_controls", "[R] Restart  [Q]/[ESC] Exit")
    )
    local tip_w = math.max(
        text_width(tr("game.2048.game_over", "Game over! [Y] Restart, [N] Back to game list.")),
        text_width(tr("game.2048.confirm_restart", "Confirm restart? [Y] Yes / [N] No")),
        text_width(tr("game.2048.confirm_exit", "Confirm exit? [Y] Yes / [N] No")),
        win_line_w
    )

    local min_w = math.max(frame_w, controls_w, status_w, best_w, tip_w) + 2
    -- Render range is [y-3, y+frame_h+3], and y is clamped to >= 5.
    -- So minimum height must be at least frame_h + 8.
    local min_h = frame_h + 8
    return min_w, min_h
end

local function draw_terminal_size_warning(term_w, term_h, min_w, min_h)
    local lines = {
        tr("warning.size_title", "Terminal Too Small"),
        string.format("%s: %dx%d", tr("warning.required", "Required size"), min_w, min_h),
        string.format("%s: %dx%d", tr("warning.current", "Current size"), term_w, term_h),
        tr("warning.enlarge_hint", "Please enlarge terminal window to continue.")
    }

    local top = math.floor((term_h - #lines) / 2)
    if top < 1 then top = 1 end

    for i = 1, #lines do
        local line = lines[i]
        local x = math.floor((term_w - text_width(line)) / 2)
        if x < 1 then x = 1 end
        draw_text(x, top + i - 1, line, "white", "black")
    end
end

local function ensure_terminal_size_ok()
    local term_w, term_h = terminal_size()
    local min_w, min_h = minimum_required_size()

    if term_w >= min_w and term_h >= min_h then
        if state.size_warning_active then
            clear()
            state.last_area = nil
            state.dirty = true
        end
        state.size_warning_active = false
        return true
    end

    local changed = (not state.size_warning_active)
        or state.last_warn_term_w ~= term_w
        or state.last_warn_term_h ~= term_h
        or state.last_warn_min_w ~= min_w
        or state.last_warn_min_h ~= min_h

    if changed then
        clear()
        draw_terminal_size_warning(term_w, term_h, min_w, min_h)
        state.last_warn_term_w = term_w
        state.last_warn_term_h = term_h
        state.last_warn_min_w = min_w
        state.last_warn_min_h = min_h
    end

    state.size_warning_active = true
    return false
end

local function init_game()
    clear()
    state.last_term_w, state.last_term_h = terminal_size()
    state.last_area = nil
    load_best_record()
    state.launch_mode = read_launch_mode()
    if state.launch_mode == "continue" then
        if not load_game_state() then
            reset_game()
        end
    else
        reset_game()
    end
    update_win_and_loss()
    state.dirty = true
end

local function game_loop()
    while true do
        local key = normalize_key(get_key(false))

        if ensure_terminal_size_ok() then
            local action = handle_input(key)
            if action == "exit" then
                return
            end

            sync_terminal_resize()
            auto_save_if_needed()
            refresh_dirty_flags()

            if state.dirty then
                render()
                state.dirty = false
            end

            state.frame = state.frame + 1
        else
            if key == "q" or key == "esc" then
                return
            end
        end

        sleep(FRAME_MS)
    end
end

init_game()
game_loop()
