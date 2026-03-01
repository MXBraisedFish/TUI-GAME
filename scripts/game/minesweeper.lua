GAME_META = {
    name = "Minesweeper",
    description = "Reveal safe cells and mark all hidden mines."
}

local OFFICIAL = {
    [1] = { rows = 9, cols = 9, mines = 10 },
    [2] = { rows = 16, cols = 16, mines = 40 },
    [3] = { rows = 16, cols = 30, mines = 99 }
}

local MIN_DIFFICULTY = 1
local MAX_DIFFICULTY = 3
local MAX_ROWS = 22
local MAX_COLS = 32
local MIN_ROWS = 2
local MIN_COLS = 2

local FPS = 60
local FRAME_MS = 16
local ROW_LABEL_W = 4
local FACE_ACTION_FRAMES = 18

local COLOR_HIDDEN = "white"
local COLOR_FLAG = "rgb(255,165,0)"
local COLOR_QUESTION = "rgb(0,140,255)"
local COLOR_MINE = "rgb(255,0,0)"
local COLOR_EMPTY = "rgb(180,180,180)"
local COLOR_CURSOR = "yellow"

local NUMBER_COLORS = {
    [1] = "rgb(0,0,255)",
    [2] = "rgb(0,130,0)",
    [3] = "rgb(255,0,0)",
    [4] = "rgb(0,0,132)",
    [5] = "rgb(132,0,0)",
    [6] = "rgb(0,130,132)",
    [7] = "rgb(105,105,105)",
    [8] = "rgb(128,128,128)"
}

local state = {
    rows = OFFICIAL[1].rows,
    cols = OFFICIAL[1].cols,
    mines = OFFICIAL[1].mines,
    difficulty = 1,
    mine_map = {},
    adj = {},
    revealed = {},
    marks = {},
    mines_placed = false,
    first_move = true,
    cursor_r = 1,
    cursor_c = 1,
    won = false,
    lost = false,
    frame = 0,
    start_frame = 0,
    end_frame = nil,
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
    best = {},
    best_committed = false,
    action_face_until = 0,
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

local function clamp(v, lo, hi)
    if v < lo then return lo end
    if v > hi then return hi end
    return v
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

local function normalize_key(key)
    if key == nil then return "" end
    if type(key) == "string" then return string.lower(key) end
    return tostring(key):lower()
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

local function new_bool_matrix(rows, cols, value)
    local m = {}
    for r = 1, rows do
        m[r] = {}
        for c = 1, cols do
            m[r][c] = value
        end
    end
    return m
end

local function new_num_matrix(rows, cols, value)
    local m = {}
    for r = 1, rows do
        m[r] = {}
        for c = 1, cols do
            m[r][c] = value
        end
    end
    return m
end

local function copy_matrix(matrix, rows, cols)
    local out = {}
    for r = 1, rows do
        out[r] = {}
        for c = 1, cols do
            out[r][c] = matrix[r][c]
        end
    end
    return out
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

local function count_flags()
    local total = 0
    for r = 1, state.rows do
        for c = 1, state.cols do
            if state.marks[r][c] == 1 then
                total = total + 1
            end
        end
    end
    return total
end

local function face_text()
    if state.won then
        return "$o$"
    end
    if state.lost then
        return "X_X"
    end
    if state.frame <= state.action_face_until then
        return "oOo"
    end
    return "ovo"
end

local function trigger_action_face()
    state.action_face_until = state.frame + FACE_ACTION_FRAMES
end

local function load_best_record()
    if type(load_data) ~= "function" then
        return {}
    end
    local ok, data = pcall(load_data, "minesweeper_best")
    if not ok or type(data) ~= "table" then
        return {}
    end
    local out = {}
    for d = MIN_DIFFICULTY, MAX_DIFFICULTY do
        local k = tostring(d)
        local value = tonumber(data[k])
        if value ~= nil and value >= 0 then
            out[d] = math.floor(value)
        end
    end
    return out
end

local function save_best_record()
    if type(save_data) ~= "function" then
        return
    end
    local payload = {}
    for d = MIN_DIFFICULTY, MAX_DIFFICULTY do
        if state.best[d] ~= nil then
            payload[tostring(d)] = state.best[d]
        end
    end
    pcall(save_data, "minesweeper_best", payload)
end

local function commit_best_if_needed()
    if state.best_committed then
        return
    end
    if state.difficulty >= MIN_DIFFICULTY and state.difficulty <= MAX_DIFFICULTY then
        local elapsed = elapsed_seconds()
        local old = state.best[state.difficulty]
        if old == nil or elapsed < old then
            state.best[state.difficulty] = elapsed
            save_best_record()
        end
    end
    state.best_committed = true
end

local function best_line()
    local d1 = state.best[1]
    local d2 = state.best[2]
    local d3 = state.best[3]
    if d1 == nil and d2 == nil and d3 == nil then
        return tr("game.minesweeper.best_none", "Best: none")
    end
    local function fmt(v)
        if v == nil then return "-" end
        return format_duration(v)
    end
    return string.format(
        "%s  1:%s  2:%s  3:%s",
        tr("game.minesweeper.best_title", "Best"),
        fmt(d1), fmt(d2), fmt(d3)
    )
end

local function make_snapshot()
    return {
        rows = state.rows,
        cols = state.cols,
        mines = state.mines,
        difficulty = state.difficulty,
        mine_map = copy_matrix(state.mine_map, state.rows, state.cols),
        adj = copy_matrix(state.adj, state.rows, state.cols),
        revealed = copy_matrix(state.revealed, state.rows, state.cols),
        marks = copy_matrix(state.marks, state.rows, state.cols),
        mines_placed = state.mines_placed,
        first_move = state.first_move,
        cursor_r = state.cursor_r,
        cursor_c = state.cursor_c,
        elapsed_sec = elapsed_seconds(),
        won = state.won,
        lost = state.lost,
        last_auto_save_sec = state.last_auto_save_sec
    }
end

local function save_game_state(show_toast)
    local ok = false
    local snapshot = make_snapshot()
    if type(save_game_slot) == "function" then
        local s, ret = pcall(save_game_slot, "minesweeper", snapshot)
        ok = s and ret ~= false
    elseif type(save_data) == "function" then
        local s, ret = pcall(save_data, "minesweeper", snapshot)
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

    local rows = tonumber(snapshot.rows)
    local cols = tonumber(snapshot.cols)
    local mines = tonumber(snapshot.mines)
    if rows == nil or cols == nil or mines == nil then
        return false
    end
    rows = clamp(math.floor(rows), MIN_ROWS, MAX_ROWS)
    cols = clamp(math.floor(cols), MIN_COLS, MAX_COLS)
    local max_mines = rows * cols - 1
    mines = clamp(math.floor(mines), 1, max_mines)

    if type(snapshot.mine_map) ~= "table" or type(snapshot.adj) ~= "table"
        or type(snapshot.revealed) ~= "table" or type(snapshot.marks) ~= "table" then
        return false
    end

    local mine_map = new_bool_matrix(rows, cols, false)
    local adj = new_num_matrix(rows, cols, 0)
    local revealed = new_bool_matrix(rows, cols, false)
    local marks = new_num_matrix(rows, cols, 0)

    for r = 1, rows do
        if type(snapshot.mine_map[r]) ~= "table" or type(snapshot.adj[r]) ~= "table"
            or type(snapshot.revealed[r]) ~= "table" or type(snapshot.marks[r]) ~= "table" then
            return false
        end
        for c = 1, cols do
            mine_map[r][c] = not not snapshot.mine_map[r][c]
            adj[r][c] = math.max(0, math.floor(tonumber(snapshot.adj[r][c]) or 0))
            revealed[r][c] = not not snapshot.revealed[r][c]
            marks[r][c] = clamp(math.floor(tonumber(snapshot.marks[r][c]) or 0), 0, 2)
        end
    end

    state.rows = rows
    state.cols = cols
    state.mines = mines
    state.difficulty = clamp(math.floor(tonumber(snapshot.difficulty) or 0), 0, 3)
    state.mine_map = mine_map
    state.adj = adj
    state.revealed = revealed
    state.marks = marks
    state.mines_placed = not not snapshot.mines_placed
    state.first_move = not not snapshot.first_move
    state.cursor_r = clamp(math.floor(tonumber(snapshot.cursor_r) or 1), 1, rows)
    state.cursor_c = clamp(math.floor(tonumber(snapshot.cursor_c) or 1), 1, cols)

    local elapsed = math.max(0, math.floor(tonumber(snapshot.elapsed_sec) or 0))
    state.start_frame = state.frame - elapsed * FPS
    state.last_auto_save_sec = math.max(0, math.floor(tonumber(snapshot.last_auto_save_sec) or elapsed))
    state.won = not not snapshot.won
    state.lost = not not snapshot.lost
    state.end_frame = nil
    if state.won or state.lost then
        state.end_frame = state.frame
    end

    state.confirm_mode = nil
    state.input_mode = nil
    state.input_buffer = ""
    state.toast_text = nil
    state.toast_until = 0
    state.best_committed = state.won
    state.action_face_until = 0
    state.last_area = nil
    state.dirty = true
    return true
end

local function load_game_state()
    local ok = false
    local snapshot = nil
    if type(load_game_slot) == "function" then
        local s, ret = pcall(load_game_slot, "minesweeper")
        ok = s and ret ~= nil
        snapshot = ret
    elseif type(load_data) == "function" then
        local s, ret = pcall(load_data, "minesweeper")
        ok = s and ret ~= nil
        snapshot = ret
    end
    if ok then
        return restore_snapshot(snapshot)
    end
    return false
end

local function reset_game(rows, cols, mines, difficulty)
    state.rows = clamp(rows, MIN_ROWS, MAX_ROWS)
    state.cols = clamp(cols, MIN_COLS, MAX_COLS)
    state.mines = clamp(mines, 1, state.rows * state.cols - 1)
    state.difficulty = difficulty or 0
    state.mine_map = new_bool_matrix(state.rows, state.cols, false)
    state.adj = new_num_matrix(state.rows, state.cols, 0)
    state.revealed = new_bool_matrix(state.rows, state.cols, false)
    state.marks = new_num_matrix(state.rows, state.cols, 0)
    state.mines_placed = false
    state.first_move = true
    state.cursor_r = 1
    state.cursor_c = 1
    state.won = false
    state.lost = false
    state.start_frame = state.frame
    state.end_frame = nil
    state.confirm_mode = nil
    state.input_mode = nil
    state.input_buffer = ""
    state.toast_text = nil
    state.toast_until = 0
    state.last_auto_save_sec = 0
    state.best_committed = false
    state.action_face_until = 0
    state.last_area = nil
    state.dirty = true
end

local function reset_official(difficulty)
    local d = clamp(difficulty, MIN_DIFFICULTY, MAX_DIFFICULTY)
    local cfg = OFFICIAL[d]
    reset_game(cfg.rows, cfg.cols, cfg.mines, d)
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
            reset_official(1)
        end
    else
        reset_official(1)
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

local function marker_positions(n)
    local a = 1
    local b = math.floor((n + 1) / 2)
    local c = n
    local out = {}
    local seen = {}
    for _, v in ipairs({ a, b, c }) do
        if not seen[v] then
            out[#out + 1] = v
            seen[v] = true
        end
    end
    return out, seen
end

local function board_geometry()
    local w, h = terminal_size()
    local grid_w = ROW_LABEL_W + state.cols
    local grid_h = 2 + state.rows

    local time_text = tr("game.minesweeper.time", "Time") .. " 00:00:00"
    local mines_text = tr("game.minesweeper.mines_left", "Mines") .. " -999"
    local status_w = key_width(time_text) + 2 + key_width("ovo") + 2 + key_width(mines_text)
    local message_w = math.max(
        key_width(tr("game.2048.confirm_restart", "Confirm restart? [Y] Yes / [N] No")),
        key_width(tr("game.2048.confirm_exit", "Confirm exit? [Y] Yes / [N] No")),
        key_width(tr("game.minesweeper.win_banner", "All mines cleared!")),
        key_width(tr("game.minesweeper.lose_banner", "You hit a mine!")),
        key_width(tr("game.minesweeper.input_config_hint", "Input 1/2/3 or row col mines.")),
        key_width(tr("game.minesweeper.input_jump_hint", "Input row col to jump."))
    )

    local controls_text = tr(
        "game.minesweeper.controls",
        "[↑]/[↓]/[←]/[→] Move  [Space] Open  [Z] Flag  [X] Question  [P] Config  [D] Jump  [R] Restart  [S] Save  [Q]/[ESC] Exit"
    )
    local controls_w = min_width_for_lines(controls_text, 3, 26)

    local frame_w = math.max(grid_w, status_w, message_w, controls_w, key_width(best_line())) + 2
    local frame_h = grid_h + 2
    local x = math.floor((w - frame_w) / 2)
    local y = math.floor((h - frame_h) / 2)
    if x < 1 then x = 1 end
    if y < 6 then y = 6 end
    return x, y, frame_w, frame_h
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

local function cell_char_and_style(r, c)
    local is_cursor = (r == state.cursor_r and c == state.cursor_c)
    local char = "#"
    local fg = COLOR_HIDDEN
    local bg = "black"

    if state.lost and state.mine_map[r][c] then
        char = "@"
        fg = COLOR_MINE
    elseif state.revealed[r][c] then
        if state.mine_map[r][c] then
            char = "@"
            fg = COLOR_MINE
        else
            local n = state.adj[r][c]
            if n <= 0 then
                char = "."
                fg = COLOR_EMPTY
            else
                char = tostring(n)
                fg = NUMBER_COLORS[n] or COLOR_EMPTY
            end
        end
    else
        local mark = state.marks[r][c]
        if mark == 1 then
            char = "!"
            fg = COLOR_FLAG
        elseif mark == 2 then
            char = "?"
            fg = COLOR_QUESTION
        else
            char = "#"
            fg = COLOR_HIDDEN
        end
    end

    if is_cursor then
        bg = COLOR_CURSOR
        if char == "!" or char == "#" then
            fg = "black"
        end
    end
    return char, fg, bg
end

local function draw_board(x, y, frame_w, frame_h)
    draw_outer_frame(x, y, frame_w, frame_h)

    local inner_x = x + 1
    local inner_y = y + 1
    local inner_w = frame_w - 2
    local grid_w = ROW_LABEL_W + state.cols
    local pad_x = math.floor((inner_w - grid_w) / 2)
    if pad_x < 0 then pad_x = 0 end
    local base_x = inner_x + pad_x

    local col_markers = marker_positions(state.cols)
    local _, row_mark_set = marker_positions(state.rows)

    draw_text(base_x, inner_y, string.rep(" ", ROW_LABEL_W + state.cols), "dark_gray", "black")
    draw_text(base_x, inner_y + 1, string.rep(" ", ROW_LABEL_W + state.cols), "dark_gray", "black")

    for _, c in ipairs(col_markers) do
        local text = tostring(c)
        local text_x = base_x + ROW_LABEL_W + c - math.floor(#text / 2) - 1
        draw_text(text_x, inner_y, text, "dark_gray", "black")
        draw_text(base_x + ROW_LABEL_W + c - 1, inner_y + 1, "|", "dark_gray", "black")
    end

    for r = 1, state.rows do
        local row_y = inner_y + 1 + r
        if row_mark_set[r] then
            draw_text(base_x, row_y, string.format("%2d -", r), "dark_gray", "black")
        else
            draw_text(base_x, row_y, "    ", "dark_gray", "black")
        end

        for c = 1, state.cols do
            local ch, fg, bg = cell_char_and_style(r, c)
            draw_text(base_x + ROW_LABEL_W + c - 1, row_y, ch, fg, bg)
        end
    end
end
local function draw_status(x, y, frame_w)
    local elapsed = elapsed_seconds()
    local term_w = terminal_size()
    local left = tr("game.minesweeper.time", "Time") .. " " .. format_duration(elapsed)
    local center = face_text()
    local right = tr("game.minesweeper.mines_left", "Mines") .. " " .. tostring(state.mines - count_flags())
    draw_text(1, y - 3, string.rep(" ", term_w), "white", "black")
    draw_text(1, y - 2, string.rep(" ", term_w), "white", "black")
    draw_text(1, y - 1, string.rep(" ", term_w), "white", "black")

    local left_x = x
    local center_x = x + math.floor((frame_w - key_width(center)) / 2)
    local right_x = x + frame_w - key_width(right)
    if center_x < left_x + key_width(left) + 1 then
        center_x = left_x + key_width(left) + 1
    end
    if right_x < center_x + key_width(center) + 1 then
        right_x = center_x + key_width(center) + 1
    end

    draw_text(x, y - 3, best_line(), "dark_gray", "black")
    draw_text(left_x, y - 2, left, "light_cyan", "black")
    draw_text(center_x, y - 2, center, "yellow", "black")
    draw_text(right_x, y - 2, right, "light_cyan", "black")

    if state.input_mode == "config" then
        if state.input_buffer == "" then
            draw_text(x, y - 1, tr("game.minesweeper.input_config_hint", "Input 1/2/3 or row col mines."), "dark_gray", "black")
        else
            draw_text(x, y - 1, state.input_buffer, "white", "black")
        end
    elseif state.input_mode == "jump" then
        if state.input_buffer == "" then
            draw_text(x, y - 1, tr("game.minesweeper.input_jump_hint", "Input row col to jump."), "dark_gray", "black")
        else
            draw_text(x, y - 1, state.input_buffer, "white", "black")
        end
    elseif state.won then
        local line = tr("game.minesweeper.win_banner", "All mines cleared!")
            .. tr("game.minesweeper.win_controls", "[R] Restart  [Q]/[ESC] Exit")
        draw_text(x, y - 1, line, "yellow", "black")
    elseif state.lost then
        local line = tr("game.minesweeper.lose_banner", "You hit a mine!")
            .. tr("game.minesweeper.lose_controls", "[R] Restart  [Q]/[ESC] Exit")
        draw_text(x, y - 1, line, "red", "black")
    elseif state.confirm_mode == "restart" then
        draw_text(x, y - 1, tr("game.2048.confirm_restart", "Confirm restart? [Y] Yes / [N] No"), "yellow", "black")
    elseif state.confirm_mode == "exit" then
        draw_text(x, y - 1, tr("game.2048.confirm_exit", "Confirm exit? [Y] Yes / [N] No"), "yellow", "black")
    elseif state.toast_text ~= nil and state.frame <= state.toast_until then
        draw_text(x, y - 1, state.toast_text, "green", "black")
    end
end

local function draw_controls(x, y, frame_h)
    local text = tr(
        "game.minesweeper.controls",
        "[↑]/[↓]/[←]/[→] Move  [Space] Open  [Z] Flag  [X] Question  [P] Config  [D] Jump  [R] Restart  [S] Save  [Q]/[ESC] Exit"
    )
    local term_w = terminal_size()
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

local function force_full_refresh()
    clear()
    state.last_area = nil
    state.dirty = true
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
    local grid_w = ROW_LABEL_W + state.cols
    local grid_h = 2 + state.rows
    local frame_w = grid_w + 2
    local frame_h = grid_h + 2

    local controls_text = tr(
        "game.minesweeper.controls",
        "[↑]/[↓]/[←]/[→] Move  [Space] Open  [Z] Flag  [X] Question  [P] Config  [D] Jump  [R] Restart  [S] Save  [Q]/[ESC] Exit"
    )
    local controls_w = min_width_for_lines(controls_text, 3, 26)
    local status_w = key_width(tr("game.minesweeper.time", "Time") .. " 00:00:00")
        + 2 + key_width("ovo")
        + 2 + key_width(tr("game.minesweeper.mines_left", "Mines") .. " -999")
    local hint_w = math.max(
        key_width(tr("game.minesweeper.input_config_hint", "Input 1/2/3 or row col mines.")),
        key_width(tr("game.minesweeper.input_jump_hint", "Input row col to jump.")),
        key_width(tr("game.minesweeper.win_banner", "All mines cleared!") .. tr("game.minesweeper.win_controls", "[R] Restart  [Q]/[ESC] Exit")),
        key_width(tr("game.minesweeper.lose_banner", "You hit a mine!") .. tr("game.minesweeper.lose_controls", "[R] Restart  [Q]/[ESC] Exit")),
        key_width(tr("game.2048.confirm_restart", "Confirm restart? [Y] Yes / [N] No")),
        key_width(tr("game.2048.confirm_exit", "Confirm exit? [Y] Yes / [N] No"))
    )

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
        local px = math.floor((term_w - key_width(line)) / 2)
        if px < 1 then px = 1 end
        draw_text(px, top + i - 1, line, "white", "black")
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
local function place_mines(exclude_r, exclude_c)
    local candidates = {}
    for r = 1, state.rows do
        for c = 1, state.cols do
            if not (r == exclude_r and c == exclude_c) then
                candidates[#candidates + 1] = { r = r, c = c }
            end
        end
    end

    for i = #candidates, 2, -1 do
        local j = random(i) + 1
        candidates[i], candidates[j] = candidates[j], candidates[i]
    end

    for i = 1, state.mines do
        local pick = candidates[i]
        state.mine_map[pick.r][pick.c] = true
    end

    state.adj = new_num_matrix(state.rows, state.cols, 0)
    for r = 1, state.rows do
        for c = 1, state.cols do
            if state.mine_map[r][c] then
                for dr = -1, 1 do
                    for dc = -1, 1 do
                        if not (dr == 0 and dc == 0) then
                            local nr = r + dr
                            local nc = c + dc
                            if nr >= 1 and nr <= state.rows and nc >= 1 and nc <= state.cols then
                                state.adj[nr][nc] = state.adj[nr][nc] + 1
                            end
                        end
                    end
                end
            end
        end
    end

    state.mines_placed = true
    state.first_move = false
end

local function all_non_mines_revealed()
    local need = state.rows * state.cols - state.mines
    local opened = 0
    for r = 1, state.rows do
        for c = 1, state.cols do
            if not state.mine_map[r][c] and state.revealed[r][c] then
                opened = opened + 1
            end
        end
    end
    return opened == need
end

local function all_mines_flagged()
    if count_flags() ~= state.mines then
        return false
    end
    for r = 1, state.rows do
        for c = 1, state.cols do
            if state.mine_map[r][c] and state.marks[r][c] ~= 1 then
                return false
            end
        end
    end
    return true
end

local function check_victory()
    if state.won or state.lost then
        return false
    end
    if not state.mines_placed then
        return false
    end
    if all_non_mines_revealed() then
        state.won = true
        state.end_frame = state.frame
        state.confirm_mode = nil
        commit_best_if_needed()
        state.dirty = true
        return true
    end
    return false
end

local function reveal_flood(start_r, start_c)
    local q_r = { start_r }
    local q_c = { start_c }
    local head = 1

    while head <= #q_r do
        local r = q_r[head]
        local c = q_c[head]
        head = head + 1

        if not state.revealed[r][c] and state.marks[r][c] == 0 then
            state.revealed[r][c] = true
            if state.adj[r][c] == 0 then
                for dr = -1, 1 do
                    for dc = -1, 1 do
                        if not (dr == 0 and dc == 0) then
                            local nr = r + dr
                            local nc = c + dc
                            if nr >= 1 and nr <= state.rows and nc >= 1 and nc <= state.cols then
                                if not state.revealed[nr][nc] and state.marks[nr][nc] == 0 then
                                    if not state.mine_map[nr][nc] then
                                        q_r[#q_r + 1] = nr
                                        q_c[#q_c + 1] = nc
                                    end
                                end
                            end
                        end
                    end
                end
            end
        end
    end
end

local function open_current_cell()
    local r = state.cursor_r
    local c = state.cursor_c

    if state.revealed[r][c] then
        return "none"
    end
    if state.marks[r][c] ~= 0 then
        return "none"
    end

    if not state.mines_placed then
        place_mines(r, c)
    end

    trigger_action_face()
    if state.mine_map[r][c] then
        state.revealed[r][c] = true
        state.lost = true
        state.end_frame = state.frame
        state.confirm_mode = nil
        state.dirty = true
        return "changed"
    end

    reveal_flood(r, c)
    check_victory()
    state.dirty = true
    return "changed"
end

local function parse_numbers(input)
    local nums = {}
    for token in string.gmatch(input, "%d+") do
        nums[#nums + 1] = math.floor(tonumber(token) or 0)
    end
    return nums
end

local function handle_config_input()
    local nums = parse_numbers(state.input_buffer)
    if #nums == 1 then
        local d = nums[1]
        if d >= 1 and d <= 3 then
            reset_official(d)
            force_full_refresh()
            return true
        end
        return false
    end

    if #nums == 3 then
        local rows = nums[1]
        local cols = nums[2]
        local mines = nums[3]
        if rows < MIN_ROWS or rows > MAX_ROWS or cols < MIN_COLS or cols > MAX_COLS then
            return false
        end
        local max_mines = rows * cols - 1
        if mines < 1 or mines > max_mines then
            return false
        end
        reset_game(rows, cols, mines, 0)
        force_full_refresh()
        return true
    end
    return false
end

local function parse_jump_input()
    local nums = parse_numbers(state.input_buffer)
    if #nums ~= 2 then
        return nil, nil
    end
    local r = nums[1]
    local c = nums[2]
    if r < 1 or r > state.rows or c < 1 or c > state.cols then
        return nil, nil
    end
    return r, c
end

local function start_input_mode(mode)
    state.input_mode = mode
    state.input_buffer = ""
    state.dirty = true
end

local function handle_input_mode_key(key)
    if key == "esc" or key == "q" then
        state.input_mode = nil
        state.input_buffer = ""
        state.dirty = true
        return "changed"
    end

    if key == "enter" then
        if state.input_mode == "config" then
            local applied = handle_config_input()
            state.input_mode = nil
            state.input_buffer = ""
            if not applied then
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

    if key:match("^%d$") or key == "space" then
        local token = key
        if key == "space" then
            token = " "
        end
        if #state.input_buffer < 12 then
            state.input_buffer = state.input_buffer .. token
            state.dirty = true
            return "changed"
        end
    end
    return "none"
end

local function handle_confirm_key(key)
    if key == "y" or key == "enter" then
        if state.confirm_mode == "restart" then
            if state.difficulty >= 1 and state.difficulty <= 3 then
                reset_official(state.difficulty)
            else
                reset_game(state.rows, state.cols, state.mines, 0)
            end
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

    if state.won or state.lost then
        if key == "r" then
            if state.difficulty >= 1 and state.difficulty <= 3 then
                reset_official(state.difficulty)
            else
                reset_game(state.rows, state.cols, state.mines, 0)
            end
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
        start_input_mode("config")
        return "changed"
    end
    if key == "d" then
        start_input_mode("jump")
        return "changed"
    end

    if key == "up" then
        state.cursor_r = clamp(state.cursor_r - 1, 1, state.rows)
        state.dirty = true
        return "changed"
    end
    if key == "down" then
        state.cursor_r = clamp(state.cursor_r + 1, 1, state.rows)
        state.dirty = true
        return "changed"
    end
    if key == "left" then
        state.cursor_c = clamp(state.cursor_c - 1, 1, state.cols)
        state.dirty = true
        return "changed"
    end
    if key == "right" then
        state.cursor_c = clamp(state.cursor_c + 1, 1, state.cols)
        state.dirty = true
        return "changed"
    end

    local r = state.cursor_r
    local c = state.cursor_c

    if key == "space" then
        return open_current_cell()
    end

    if state.revealed[r][c] then
        return "none"
    end

    if key == "z" then
        if state.marks[r][c] == 1 then
            state.marks[r][c] = 0
        else
            state.marks[r][c] = 1
        end
        trigger_action_face()
        check_victory()
        state.dirty = true
        return "changed"
    end

    if key == "x" then
        if state.marks[r][c] == 2 then
            state.marks[r][c] = 0
        else
            state.marks[r][c] = 2
        end
        trigger_action_face()
        check_victory()
        state.dirty = true
        return "changed"
    end

    return "none"
end

local function auto_save_if_needed()
    if state.won or state.lost then
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
