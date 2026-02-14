GAME_META = {
    name = "Lights Out",
    description = "Light all tiles by toggling cross patterns."
}

local DEFAULT_SIZE = 5
local MIN_SIZE = 2
local MAX_SIZE = 10

local FPS = 60
local FRAME_MS = 16

local CELL_W = 4
local CELL_H = 3
local CELL_STEP_X = 5
local CELL_STEP_Y = 2
local LABEL_W = 3

local state = {
    size = DEFAULT_SIZE,
    board = {},
    cursor_r = 1,
    cursor_c = 1,
    steps = 0,
    frame = 0,
    start_frame = 0,
    end_frame = nil,
    won = false,
    confirm_mode = nil,
    input_mode = nil,
    input_buffer = "",
    toast_text = nil,
    toast_until = 0,
    last_auto_save_sec = 0,
    dirty = true,
    last_elapsed_sec = -1,
    last_toast_visible = false,
    last_key = "",
    last_key_frame = -100,
    launch_mode = "new",
    last_area = nil,
    best = nil,
    best_committed = false,
    last_term_w = 0,
    last_term_h = 0,
    size_warning_active = false,
    last_warn_term_w = 0,
    last_warn_term_h = 0,
    last_warn_min_w = 0,
    last_warn_min_h = 0
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

local function key_width(text)
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
            if key_width(candidate) <= max_width then
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
    local full = key_width(text)
    local width = hard_min
    while width <= full do
        if #wrap_words(text, width) <= max_lines then
            return width
        end
        width = width + 1
    end
    return full
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

local function elapsed_seconds()
    local end_frame = state.end_frame
    if end_frame == nil then
        end_frame = state.frame
    end
    return math.floor((end_frame - state.start_frame) / FPS)
end

local function format_duration(sec)
    local h = math.floor(sec / 3600)
    local m = math.floor((sec % 3600) / 60)
    local s = sec % 60
    return string.format("%02d:%02d:%02d", h, m, s)
end

local function clamp(v, lo, hi)
    if v < lo then return lo end
    if v > hi then return hi end
    return v
end

local function normalize_key(key)
    if key == nil then return "" end
    if type(key) == "string" then return string.lower(key) end
    return tostring(key):lower()
end

local function new_board(size, value)
    local board = {}
    for r = 1, size do
        board[r] = {}
        for c = 1, size do
            board[r][c] = value
        end
    end
    return board
end

local function all_lit_board(board, size)
    for r = 1, size do
        for c = 1, size do
            if not board[r][c] then
                return false
            end
        end
    end
    return true
end

local function all_lit()
    return all_lit_board(state.board, state.size)
end

local function toggle_cell(board, size, r, c)
    if r < 1 or r > size or c < 1 or c > size then
        return
    end
    board[r][c] = not board[r][c]
end

local function toggle_cross_on(board, size, r, c)
    toggle_cell(board, size, r, c)
    toggle_cell(board, size, r - 1, c)
    toggle_cell(board, size, r + 1, c)
    toggle_cell(board, size, r, c - 1)
    toggle_cell(board, size, r, c + 1)
end

local function randomize_board(size)
    local board = new_board(size, true)
    for _ = 1, size * size do
        local rr = random(size) + 1
        local cc = random(size) + 1
        toggle_cross_on(board, size, rr, cc)
    end
    if all_lit_board(board, size) then
        toggle_cross_on(board, size, random(size) + 1, random(size) + 1)
    end
    return board
end

local function load_best_record()
    if type(load_data) ~= "function" then
        return nil
    end
    local ok, data = pcall(load_data, "lights_out_best")
    if not ok or type(data) ~= "table" then
        return nil
    end

    local max_size = tonumber(data.max_size)
    local min_steps = tonumber(data.min_steps)
    local min_time_sec = tonumber(data.min_time_sec)
    if max_size == nil or min_steps == nil or min_time_sec == nil then
        return nil
    end

    return {
        max_size = math.floor(max_size),
        min_steps = math.floor(min_steps),
        min_time_sec = math.floor(min_time_sec)
    }
end

local function should_replace_best(old, new)
    if old == nil then
        return true
    end
    if new.max_size ~= old.max_size then
        return new.max_size > old.max_size
    end
    if new.min_steps ~= old.min_steps then
        return new.min_steps < old.min_steps
    end
    return new.min_time_sec < old.min_time_sec
end

local function save_best_record(record)
    if type(save_data) ~= "function" then
        return
    end
    pcall(save_data, "lights_out_best", record)
end

local function commit_best_if_needed()
    if state.best_committed then
        return
    end
    local record = {
        max_size = state.size,
        min_steps = state.steps,
        min_time_sec = elapsed_seconds()
    }
    if should_replace_best(state.best, record) then
        state.best = record
        save_best_record(record)
    end
    state.best_committed = true
end

local function mark_won()
    if state.won then
        return
    end
    state.won = true
    state.end_frame = state.frame
    state.confirm_mode = nil
    commit_best_if_needed()
    state.dirty = true
end

local function make_snapshot()
    return {
        size = state.size,
        board = state.board,
        cursor_r = state.cursor_r,
        cursor_c = state.cursor_c,
        steps = state.steps,
        elapsed_sec = elapsed_seconds(),
        won = state.won,
        last_auto_save_sec = state.last_auto_save_sec
    }
end

local function save_game_state(show_toast)
    local ok = false
    local snapshot = make_snapshot()
    if type(save_game_slot) == "function" then
        local s, ret = pcall(save_game_slot, "lights_out", snapshot)
        ok = s and ret ~= false
    elseif type(save_data) == "function" then
        local s, ret = pcall(save_data, "lights_out", snapshot)
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

local function restore_snapshot(snapshot)
    if type(snapshot) ~= "table" then
        return false
    end

    local size = tonumber(snapshot.size)
    if size == nil then
        return false
    end
    size = clamp(math.floor(size), MIN_SIZE, MAX_SIZE)

    if type(snapshot.board) ~= "table" then
        return false
    end

    local board = new_board(size, false)
    for r = 1, size do
        if type(snapshot.board[r]) ~= "table" then
            return false
        end
        for c = 1, size do
            board[r][c] = not not snapshot.board[r][c]
        end
    end

    state.size = size
    state.board = board
    state.cursor_r = clamp(math.floor(tonumber(snapshot.cursor_r) or 1), 1, size)
    state.cursor_c = clamp(math.floor(tonumber(snapshot.cursor_c) or 1), 1, size)
    state.steps = math.max(0, math.floor(tonumber(snapshot.steps) or 0))

    local elapsed = math.max(0, math.floor(tonumber(snapshot.elapsed_sec) or 0))
    state.start_frame = state.frame - elapsed * FPS
    state.last_auto_save_sec = math.max(0, math.floor(tonumber(snapshot.last_auto_save_sec) or elapsed))

    state.won = not not snapshot.won
    state.end_frame = nil
    if state.won then
        state.end_frame = state.frame
    end

    state.confirm_mode = nil
    state.input_mode = nil
    state.input_buffer = ""
    state.toast_text = nil
    state.toast_until = 0
    state.best_committed = state.won
    state.last_area = nil
    state.dirty = true
    return true
end

local function load_game_state()
    local ok = false
    local snapshot = nil
    if type(load_game_slot) == "function" then
        local s, ret = pcall(load_game_slot, "lights_out")
        ok = s and ret ~= nil
        snapshot = ret
    elseif type(load_data) == "function" then
        local s, ret = pcall(load_data, "lights_out")
        ok = s and ret ~= nil
        snapshot = ret
    end

    if ok then
        return restore_snapshot(snapshot)
    end
    return false
end

local function reset_game(new_size)
    if new_size ~= nil then
        state.size = clamp(new_size, MIN_SIZE, MAX_SIZE)
    end

    -- Start from a fully unlit board (all false) instead of random state.
    state.board = new_board(state.size, false)
    state.cursor_r = 1
    state.cursor_c = 1
    state.steps = 0
    state.start_frame = state.frame
    state.end_frame = nil
    state.won = false
    state.confirm_mode = nil
    state.input_mode = nil
    state.input_buffer = ""
    state.toast_text = nil
    state.toast_until = 0
    state.last_auto_save_sec = 0
    state.best_committed = false
    state.last_area = nil
    state.dirty = true
end

local function init_game()
    clear()
    local w, h = 120, 40
    if type(get_terminal_size) == "function" then
        local tw, th = get_terminal_size()
        if type(tw) == "number" and type(th) == "number" then
            w, h = tw, th
        end
    end
    state.last_term_w, state.last_term_h = w, h
    state.best = load_best_record()
    state.launch_mode = read_launch_mode()
    if state.launch_mode == "continue" then
        if not load_game_state() then
            reset_game(DEFAULT_SIZE)
        end
    else
        reset_game(DEFAULT_SIZE)
    end
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

local function board_geometry()
    local w, h = terminal_size()
    local grid_w = (state.size - 1) * CELL_STEP_X + CELL_W
    local grid_h = (state.size - 1) * CELL_STEP_Y + CELL_H

    local status_w = key_width(tr("game.lights_out.time", "Time") .. " 00:00:00")
        + 2
        + key_width(tr("game.lights_out.steps", "Steps") .. " 9999")
    local win_line_w = key_width(
        tr("game.lights_out.win_banner", "You lit all lights!")
            .. tr("game.lights_out.win_controls", "[R] Restart  [Q]/[ESC] Exit")
    )
    local content_w = math.max(LABEL_W + grid_w, status_w, win_line_w)
    local content_h = 1 + grid_h
    local frame_w = content_w + 2
    local frame_h = content_h + 2

    local x = math.floor((w - frame_w) / 2)
    local y = math.floor((h - frame_h) / 2)
    if x < 1 then x = 1 end
    if y < 6 then y = 6 end

    return x, y, frame_w, frame_h, content_w, content_h
end

local function fill_rect(x, y, w, h, bg)
    if w <= 0 or h <= 0 then
        return
    end
    local line = string.rep(" ", w)
    for row = 0, h - 1 do
        draw_text(x, y + row, line, "white", bg or "black")
    end
end

local function draw_outer_frame(x, y, frame_w, frame_h)
    draw_text(x, y, "╔" .. string.rep("═", frame_w - 2) .. "╗", "white", "black")
    for i = 1, frame_h - 2 do
        draw_text(x, y + i, "║", "white", "black")
        draw_text(x + frame_w - 1, y + i, "║", "white", "black")
    end
    draw_text(x, y + frame_h - 1, "╚" .. string.rep("═", frame_w - 2) .. "╝", "white", "black")
end

local function draw_lamp(x, y, lit, selected)
    local lamp_color = lit and "rgb(255,255,0)" or "rgb(210,210,210)"

    if selected then
        draw_text(x, y, "┌──┐", "green", "black")
        draw_text(x, y + 1, "│", "green", "black")
        draw_text(x + 1, y + 1, "██", lamp_color, "black")
        draw_text(x + 3, y + 1, "│", "green", "black")
        draw_text(x, y + 2, "└──┘", "green", "black")
    else
        draw_text(x, y, "    ", "white", "black")
        draw_text(x, y + 1, " ██ ", lamp_color, "black")
        draw_text(x, y + 2, "    ", "white", "black")
    end
end

local function draw_board(x, y, frame_w, frame_h)
    draw_outer_frame(x, y, frame_w, frame_h)
    local inner_x = x + 1
    local inner_y = y + 1

    draw_text(inner_x, inner_y, string.rep(" ", frame_w - 2), "white", "black")

    local grid_w = (state.size - 1) * CELL_STEP_X + CELL_W
    local grid_total_w = LABEL_W + grid_w
    local pad_x = math.floor((frame_w - 2 - grid_total_w) / 2)
    if pad_x < 0 then pad_x = 0 end
    local grid_block_x = inner_x + pad_x
    local grid_x = grid_block_x + LABEL_W
    for c = 1, state.size do
        local cx = grid_x + (c - 1) * CELL_STEP_X + 1
        draw_text(cx, inner_y, string.format("%2d", c), "dark_gray", "black")
    end

    for r = 1, state.size do
        local row_base = inner_y + 1 + (r - 1) * CELL_STEP_Y
        draw_text(grid_block_x, row_base + 1, string.format("%2d", r), "dark_gray", "black")

        for c = 1, state.size do
            local cx = grid_x + (c - 1) * CELL_STEP_X
            local selected = (r == state.cursor_r and c == state.cursor_c)
            draw_lamp(cx, row_base, state.board[r][c], selected)
        end
    end

    -- Redraw selected lamp last so its border is never erased by overlapping row spacing.
    local sr = state.cursor_r
    local sc = state.cursor_c
    if sr >= 1 and sr <= state.size and sc >= 1 and sc <= state.size then
        local sel_y = inner_y + 1 + (sr - 1) * CELL_STEP_Y
        local sel_x = grid_x + (sc - 1) * CELL_STEP_X
        draw_lamp(sel_x, sel_y, state.board[sr][sc], true)
    end
end

local function best_line()
    if state.best == nil then
        return tr("game.lights_out.best_none", "Best: none")
    end

    return string.format(
        "%s %dx%d  %s %d  %s %s",
        tr("game.lights_out.best_size", "Max"),
        state.best.max_size,
        state.best.max_size,
        tr("game.lights_out.best_steps", "Steps"),
        state.best.min_steps,
        tr("game.lights_out.best_time", "Time"),
        format_duration(state.best.min_time_sec)
    )
end

local function draw_status(x, y, frame_w)
    local elapsed = elapsed_seconds()
    local time_text = tr("game.lights_out.time", "Time") .. " " .. format_duration(elapsed)
    local steps_text = tr("game.lights_out.steps", "Steps") .. " " .. tostring(state.steps)
    local term_w = terminal_size()
    local right_x = x + frame_w - key_width(steps_text)
    if right_x < 1 then right_x = 1 end

    draw_text(1, y - 3, string.rep(" ", term_w), "white", "black")
    draw_text(1, y - 2, string.rep(" ", term_w), "white", "black")
    draw_text(1, y - 1, string.rep(" ", term_w), "white", "black")

    draw_text(x, y - 3, best_line(), "dark_gray", "black")
    draw_text(x, y - 2, time_text, "light_cyan", "black")
    draw_text(right_x, y - 2, steps_text, "light_cyan", "black")

    if state.input_mode == "size" then
        if state.input_buffer == "" then
            draw_text(x, y - 1, tr("game.lights_out.input_size_hint", "Input 2-10 to resize board."), "dark_gray", "black")
        else
            draw_text(x, y - 1, state.input_buffer, "white", "black")
        end
    elseif state.input_mode == "jump" then
        if state.input_buffer == "" then
            draw_text(x, y - 1, tr("game.lights_out.input_jump_hint", "Input xx xx to jump to coordinates."), "dark_gray", "black")
        else
            draw_text(x, y - 1, state.input_buffer, "white", "black")
        end
    elseif state.won then
        local line = tr("game.lights_out.win_banner", "You lit all lights!")
            .. tr("game.lights_out.win_controls", "[R] Restart  [Q]/[ESC] Exit")
        draw_text(x, y - 1, line, "yellow", "black")
    elseif state.confirm_mode == "restart" then
        draw_text(x, y - 1, tr("game.2048.confirm_restart", "Confirm restart? [Y] Yes / [N] No"), "yellow", "black")
    elseif state.confirm_mode == "exit" then
        draw_text(x, y - 1, tr("game.2048.confirm_exit", "Confirm exit? [Y] Yes / [N] No"), "yellow", "black")
    elseif state.toast_text ~= nil and state.frame <= state.toast_until then
        draw_text(x, y - 1, state.toast_text, "green", "black")
    end
end

local function draw_controls(x, y, frame_h)
    local term_w = terminal_size()
    local text = tr(
        "game.lights_out.controls",
        "[↑]/[↓]/[←]/[→] Move  [Space] Toggle  [P] Resize  [D] Jump  [R] Restart  [S] Save  [Q]/[ESC] Exit"
    )
    local max_w = math.max(10, term_w - 2)
    local lines = wrap_words(text, max_w)
    if #lines > 3 then
        lines = { lines[1], lines[2], lines[3] }
    end

    for i = 1, 3 do
        draw_text(1, y + frame_h + i, string.rep(" ", term_w), "white", "black")
    end

    local offset = 0
    if #lines < 3 then
        offset = math.floor((3 - #lines) / 2)
    end
    for i = 1, #lines do
        local line = lines[i]
        local line_x = math.floor((term_w - key_width(line)) / 2)
        if line_x < 1 then line_x = 1 end
        draw_text(line_x, y + frame_h + 1 + offset + i - 1, line, "white", "black")
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
    draw_board(x, y, frame_w, frame_h)
    draw_controls(x, y, frame_h)
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
    local grid_w = (state.size - 1) * CELL_STEP_X + CELL_W
    local grid_h = (state.size - 1) * CELL_STEP_Y + CELL_H
    local frame_w = LABEL_W + grid_w + 2
    local frame_h = 1 + grid_h + 2

    local controls_w = min_width_for_lines(
        tr(
            "game.lights_out.controls",
            "[↑]/[↓]/[←]/[→] Move  [Space] Toggle  [P] Resize  [D] Jump  [R] Restart  [S] Save  [Q]/[ESC] Exit"
        ),
        3,
        24
    )
    local status_w = key_width(tr("game.lights_out.time", "Time") .. " 00:00:00")
        + 2
        + key_width(tr("game.lights_out.steps", "Steps") .. " 9999")
    local hint_w = key_width(tr("game.lights_out.input_jump_hint", "Input xx xx to jump to coordinates."))

    local min_w = math.max(frame_w, controls_w, status_w, hint_w) + 2
    -- Render range is [y-3, y+frame_h+3], and y is clamped to >= 6.
    -- So minimum height must be at least frame_h + 9.
    local min_h = frame_h + 9
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
        local x = math.floor((term_w - key_width(line)) / 2)
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

local function start_input_mode(mode)
    state.input_mode = mode
    state.input_buffer = ""
    state.dirty = true
end

local function parse_size_input()
    local value = tonumber(state.input_buffer)
    if value == nil then
        return nil
    end
    value = math.floor(value)
    if value < MIN_SIZE or value > MAX_SIZE then
        return nil
    end
    return value
end

local function parse_jump_input()
    local a, b = state.input_buffer:match("^(%d+)%s+(%d+)$")
    if a == nil or b == nil then
        return nil, nil
    end
    local r = math.floor(tonumber(a) or 0)
    local c = math.floor(tonumber(b) or 0)
    if r < 1 or r > state.size or c < 1 or c > state.size then
        return nil, nil
    end
    return r, c
end

local function handle_input_mode_key(key)
    if key == "esc" or key == "q" then
        state.input_mode = nil
        state.input_buffer = ""
        state.dirty = true
        return "changed"
    end

    if key == "enter" then
        if state.input_mode == "size" then
            local size = parse_size_input()
            state.input_mode = nil
            state.input_buffer = ""
            if size ~= nil then
                if size ~= state.size then
                    clear()
                    state.last_area = nil
                end
                reset_game(size)
            else
                state.dirty = true
            end
            return "changed"
        end

        if state.input_mode == "jump" then
            local r, c = parse_jump_input()
            state.input_mode = nil
            state.input_buffer = ""
            if r ~= nil and c ~= nil then
                state.cursor_r = r
                state.cursor_c = c
            end
            state.dirty = true
            return "changed"
        end
    end

    if key == "backspace" then
        if #state.input_buffer > 0 then
            state.input_buffer = string.sub(state.input_buffer, 1, #state.input_buffer - 1)
            state.dirty = true
            return "changed"
        end
        return "none"
    end

    if state.input_mode == "size" then
        if key:match("^%d$") then
            if #state.input_buffer < 2 then
                state.input_buffer = state.input_buffer .. key
                state.dirty = true
                return "changed"
            end
        end
        return "none"
    end

    if state.input_mode == "jump" then
        if key:match("^%d$") or key == "space" then
            local token = key
            if key == "space" then
                token = " "
            end
            if #state.input_buffer < 6 then
                state.input_buffer = state.input_buffer .. token
                state.dirty = true
                return "changed"
            end
        end
        return "none"
    end

    return "none"
end

local function handle_confirm_key(key)
    if key == "y" or key == "enter" then
        if state.confirm_mode == "restart" then
            reset_game(state.size)
            return "changed"
        end
        if state.confirm_mode == "exit" then
            return "exit"
        end
    end

    if key == "n" or key == "q" or key == "esc" then
        state.confirm_mode = nil
        state.dirty = true
        return "changed"
    end

    return "none"
end

local function should_debounce(key)
    if not (key == "up" or key == "down" or key == "left" or key == "right") then
        return false
    end
    if key == state.last_key and (state.frame - state.last_key_frame) <= 2 then
        return true
    end
    state.last_key = key
    state.last_key_frame = state.frame
    return false
end

local function handle_input(key)
    if key == nil or key == "" then
        return "none"
    end

    if should_debounce(key) then
        return "none"
    end

    if state.input_mode ~= nil then
        return handle_input_mode_key(key)
    end

    if state.confirm_mode ~= nil then
        return handle_confirm_key(key)
    end

    if state.won then
        if key == "r" then
            reset_game(state.size)
            return "changed"
        end
        if key == "q" or key == "esc" then
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

    if key == "p" then
        start_input_mode("size")
        return "changed"
    end

    if key == "d" then
        start_input_mode("jump")
        return "changed"
    end

    if key == "up" then
        state.cursor_r = clamp(state.cursor_r - 1, 1, state.size)
        state.dirty = true
        return "changed"
    end

    if key == "down" then
        state.cursor_r = clamp(state.cursor_r + 1, 1, state.size)
        state.dirty = true
        return "changed"
    end

    if key == "left" then
        state.cursor_c = clamp(state.cursor_c - 1, 1, state.size)
        state.dirty = true
        return "changed"
    end

    if key == "right" then
        state.cursor_c = clamp(state.cursor_c + 1, 1, state.size)
        state.dirty = true
        return "changed"
    end

    if key == "space" then
        toggle_cross_on(state.board, state.size, state.cursor_r, state.cursor_c)
        state.steps = state.steps + 1
        if all_lit() then
            mark_won()
        else
            state.dirty = true
        end
        return "changed"
    end

    return "none"
end

local function auto_save_if_needed()
    if state.won then
        return
    end
    local elapsed = elapsed_seconds()
    if elapsed - state.last_auto_save_sec >= 60 then
        save_game_state(false)
        state.last_auto_save_sec = elapsed
    end
end

local function refresh_dirty_flags()
    local elapsed = elapsed_seconds()
    if elapsed ~= state.last_elapsed_sec then
        state.last_elapsed_sec = elapsed
        state.dirty = true
    end

    local toast_visible = state.toast_text ~= nil and state.frame <= state.toast_until
    if toast_visible ~= state.last_toast_visible then
        state.last_toast_visible = toast_visible
        state.dirty = true
    end
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
