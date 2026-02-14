GAME_META = {
    name = "Color Memory",
    description = "Repeat the color sequence exactly as the system presents it."
}

local FPS = 60
local FRAME_MS = 16
local SHOW_ON_MS = 1200
local SHOW_OFF_MS = 800

local BOX_W = 4
local BOX_H = 3
local BOX_GAP = 3
local INPUT_GAP = 1
local FRAME_H = 12

local COLORS = {
    { bg = "rgb(255,0,0)" },
    { bg = "rgb(255,255,0)" },
    { bg = "rgb(0,120,255)" },
    { bg = "rgb(0,200,0)" }
}

local state = {
    score = 0,
    round = 1,
    sequence = {},
    input_colors = {},
    highlight_idx = 0,

    best_score = 0,
    best_time_sec = 0,

    phase = "input",
    lost = false,
    confirm_mode = nil,
    committed = false,

    frame = 0,
    start_frame = 0,
    end_frame = nil,
    running = true,
    dirty = true,

    last_elapsed_sec = -1,
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

local function normalize_key(key)
    if key == nil then return "" end
    if type(key) == "string" then return string.lower(key) end
    return tostring(key):lower()
end

local function flush_input_buffer()
    if type(clear_input_buffer) == "function" then
        pcall(clear_input_buffer)
    end
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

local function fill_line(y, width)
    draw_text(1, y, string.rep(" ", width), "white", "black")
end

local function fill_rect(x, y, w, h, bg)
    if w <= 0 or h <= 0 then return end
    local line = string.rep(" ", w)
    for i = 0, h - 1 do
        draw_text(x, y + i, line, "white", bg or "black")
    end
end

local function draw_outer_frame(x, y, w, h)
    draw_text(x, y, "╔" .. string.rep("═", w - 2) .. "╗", "white", "black")
    for i = 1, h - 2 do
        draw_text(x, y + i, "║", "white", "black")
        draw_text(x + w - 1, y + i, "║", "white", "black")
    end
    draw_text(x, y + h - 1, "╚" .. string.rep("═", w - 2) .. "╝", "white", "black")
end

local function draw_color_fill_slot(x, y, color_idx)
    local bg = COLORS[color_idx].bg
    fill_rect(x, y, BOX_W, BOX_H, "black")
    draw_text(x + 1, y + 1, "  ", "white", bg)
end

local function draw_highlight_box(x, y, color_idx)
    local bg = COLORS[color_idx].bg
    draw_text(x, y, "┌──┐", "white", "black")
    draw_text(x, y + 1, "│", "white", "black")
    draw_text(x + 1, y + 1, "  ", "white", bg)
    draw_text(x + 3, y + 1, "│", "white", "black")
    draw_text(x, y + 2, "└──┘", "white", "black")
end

local function load_best_record()
    if type(load_data) ~= "function" then
        return
    end
    local ok, data = pcall(load_data, "color_memory_best")
    if not ok or type(data) ~= "table" then
        return
    end
    local bs = tonumber(data.best_score)
    local bt = tonumber(data.best_time_sec)
    if bs ~= nil and bs >= 0 then
        state.best_score = math.floor(bs)
    end
    if bt ~= nil and bt >= 0 then
        state.best_time_sec = math.floor(bt)
    end
end

local function save_best_record()
    if type(save_data) ~= "function" then
        return
    end
    pcall(save_data, "color_memory_best", {
        best_score = state.best_score,
        best_time_sec = state.best_time_sec
    })
end

local function commit_stats_if_needed()
    if state.committed then
        return
    end
    local dur = elapsed_seconds()
    if state.score > state.best_score then
        state.best_score = state.score
    end
    if dur > state.best_time_sec then
        state.best_time_sec = dur
    end
    save_best_record()
    if type(update_game_stats) == "function" then
        pcall(update_game_stats, "color_memory", state.score, dur)
    end
    state.committed = true
end

local function advance_time(ms)
    local delta = math.max(1, math.floor(ms / FRAME_MS))
    state.frame = state.frame + delta
end

local function centered_x(text, left_x, right_x)
    local width = key_width(text)
    local x = left_x + math.floor(((right_x - left_x + 1) - width) / 2)
    if x < left_x then x = left_x end
    if x > right_x - width + 1 then
        x = math.max(left_x, right_x - width + 1)
    end
    return x
end

local function minimum_required_size()
    local controls = tr(
        "game.color_memory.controls",
        "[1][2][3][4] Input Color  [Enter] Submit  [Backspace]/[Delete] Remove  [R] Restart  [Q]/[ESC] Exit"
    )
    local controls_w = min_width_for_lines(controls, 3, 40)

    local best_line = tr("game.color_memory.best_score", "Best Score") .. " 99999  "
        .. tr("game.color_memory.best_time", "Longest Play") .. " 00:00:00"
    local curr_line = tr("game.color_memory.time", "Time") .. " 00:00:00  "
        .. tr("game.color_memory.score", "Score") .. " 99999"

    local info_w = math.max(
        key_width(tr("game.color_memory.confirm_restart", "Confirm restart? [Y] Yes / [N] No")),
        key_width(tr("game.color_memory.confirm_exit", "Confirm exit? [Y] Yes / [N] No")),
        key_width(
            tr("game.color_memory.lose_banner", "Wrong color order. Game over!")
            .. " "
            .. tr("game.color_memory.lose_controls", "[R] Restart  [Q]/[ESC] Exit")
        )
    )

    local boxes_w = 4 * BOX_W + 3 * BOX_GAP
    local frame_w = math.max(48, boxes_w + 10, info_w + 2)
    local min_w = math.max(frame_w + 2, controls_w + 2, key_width(best_line) + 2, key_width(curr_line) + 2)
    local min_h = FRAME_H + 7
    return min_w, min_h
end

local function draw_terminal_size_warning(term_w, term_h, min_w, min_h)
    clear()
    local lines = {
        tr("warning.size_title", "Terminal Too Small"),
        string.format("%s: %dx%d", tr("warning.required", "Required size"), min_w, min_h),
        string.format("%s: %dx%d", tr("warning.current", "Current size"), term_w, term_h),
        tr("warning.enlarge_hint", "Please enlarge terminal window to continue.")
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
        local resized = (term_w ~= state.last_term_w) or (term_h ~= state.last_term_h)
        state.last_term_w = term_w
        state.last_term_h = term_h
        if state.size_warning_active or resized then
            clear()
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
        draw_terminal_size_warning(term_w, term_h, min_w, min_h)
        state.last_warn_term_w = term_w
        state.last_warn_term_h = term_h
        state.last_warn_min_w = min_w
        state.last_warn_min_h = min_h
    end
    state.size_warning_active = true
    return false
end

local function frame_geometry()
    local term_w, term_h = terminal_size()
    local frame_w = math.max(48, 4 * BOX_W + 3 * BOX_GAP + 10)
    local top_h = 3
    local bottom_h = 3
    local block_h = top_h + FRAME_H + bottom_h

    local top = math.floor((term_h - block_h) / 2) + 1
    if top < 1 then top = 1 end

    local x = math.floor((term_w - frame_w) / 2)
    if x < 1 then x = 1 end

    return {
        best_y = top,
        current_y = top + 1,
        info_y = top + 2,
        game_x = x,
        game_y = top + 3,
        frame_w = frame_w,
        frame_h = FRAME_H,
        controls_y = top + 3 + FRAME_H + 1,
        term_w = term_w
    }
end

local function game_inner(g)
    return g.game_x + 1, g.game_y + 1, g.frame_w - 2
end

local function format_round_text()
    local tmpl = tr("game.color_memory.round", "Round {n}")
    if string.find(tmpl, "{n}", 1, true) ~= nil then
        return string.gsub(tmpl, "{n}", tostring(state.round))
    end
    if tmpl == "第几局" then
        return "第" .. tostring(state.round) .. "局"
    end
    return tmpl .. " " .. tostring(state.round)
end

local function draw_show_section(g)
    local inner_x = g.game_x + 1
    local inner_y = g.game_y + 1
    local inner_w = g.frame_w - 2
    fill_rect(inner_x, inner_y, inner_w, 7, "black")

    local round_text = format_round_text()
    draw_text(centered_x(round_text, inner_x, inner_x + inner_w - 1), inner_y, round_text, "yellow", "black")

    local total_boxes_w = 4 * BOX_W + 3 * BOX_GAP
    local row_x = inner_x + math.floor((inner_w - total_boxes_w) / 2)
    local show_y = inner_y + 2
    for i = 1, 4 do
        local bx = row_x + (i - 1) * (BOX_W + BOX_GAP)
        if state.highlight_idx == i then
            draw_highlight_box(bx, show_y, i)
        else
            draw_color_fill_slot(bx, show_y, i)
        end
    end

    local status_text = ""
    if state.phase == "show" then
        status_text = tr("game.color_memory.status_observe", "Drawing colors, watch carefully......")
    elseif state.phase == "input" then
        status_text = tr("game.color_memory.status_input", "Please input the color sequence.")
    end
    draw_text(centered_x(status_text, inner_x, inner_x + inner_w - 1), inner_y + 6, status_text, "dark_gray", "black")
end

local function draw_input_section(g)
    local inner_x = g.game_x + 1
    local inner_y = g.game_y + 1
    local inner_w = g.frame_w - 2
    local input_y = inner_y + 7
    fill_rect(inner_x, input_y, inner_w, 3, "black")

    local max_slots = math.max(1, math.floor((inner_w + INPUT_GAP) / (BOX_W + INPUT_GAP)))
    local start_idx = 1
    if #state.input_colors > max_slots then
        start_idx = #state.input_colors - max_slots + 1
    end
    local visible = #state.input_colors - start_idx + 1
    local input_w = visible * BOX_W + math.max(0, visible - 1) * INPUT_GAP
    local input_x = inner_x + math.floor((inner_w - input_w) / 2)
    for i = start_idx, #state.input_colors do
        local slot = i - start_idx
        local bx = input_x + slot * (BOX_W + INPUT_GAP)
        draw_color_fill_slot(bx, input_y, state.input_colors[i])
    end

    -- Keep the bottom double-line border intact during partial redraws.
    draw_text(
        g.game_x,
        g.game_y + g.frame_h - 1,
        "╚" .. string.rep("═", g.frame_w - 2) .. "╝",
        "white",
        "black"
    )
end

local function draw_header(g)
    fill_line(g.best_y, g.term_w)
    fill_line(g.current_y, g.term_w)
    fill_line(g.info_y, g.term_w)

    local best_line = tr("game.color_memory.best_score", "Best Score") .. ": " .. tostring(state.best_score)
        .. "  "
        .. tr("game.color_memory.best_time", "Longest Play") .. ": " .. format_duration(state.best_time_sec)
    draw_text(centered_x(best_line, 1, g.term_w), g.best_y, best_line, "dark_gray", "black")

    local current_line = tr("game.color_memory.time", "Time") .. ": " .. format_duration(elapsed_seconds())
        .. "  "
        .. tr("game.color_memory.score", "Score") .. ": " .. tostring(state.score)
    draw_text(centered_x(current_line, 1, g.term_w), g.current_y, current_line, "light_cyan", "black")

    local info = ""
    local info_color = "yellow"
    if state.confirm_mode == "restart" then
        info = tr("game.color_memory.confirm_restart", "Confirm restart? [Y] Yes / [N] No")
    elseif state.confirm_mode == "exit" then
        info = tr("game.color_memory.confirm_exit", "Confirm exit? [Y] Yes / [N] No")
    elseif state.lost then
        info = tr("game.color_memory.lose_banner", "Wrong color order. Game over!")
            .. " "
            .. tr("game.color_memory.lose_controls", "[R] Restart  [Q]/[ESC] Exit")
        info_color = "red"
    end
    if info ~= "" then
        draw_text(centered_x(info, 1, g.term_w), g.info_y, info, info_color, "black")
    end
end

local function draw_controls(g)
    local controls = tr(
        "game.color_memory.controls",
        "[1][2][3][4] Input Color  [Enter] Submit  [Backspace]/[Delete] Remove  [R] Restart  [Q]/[ESC] Exit"
    )
    local lines = wrap_words(controls, math.max(10, g.term_w - 2))
    if #lines > 3 then
        lines = { lines[1], lines[2], lines[3] }
    end

    for i = 0, 2 do
        fill_line(g.controls_y + i, g.term_w)
    end

    local offset = 0
    if #lines < 3 then
        offset = math.floor((3 - #lines) / 2)
    end

    for i = 1, #lines do
        local line = lines[i]
        draw_text(centered_x(line, 1, g.term_w), g.controls_y + offset + i - 1, line, "white", "black")
    end
end

local function render_full(g)
    draw_header(g)
    draw_outer_frame(g.game_x, g.game_y, g.frame_w, g.frame_h)
    local inner_x, inner_y, inner_w = game_inner(g)
    fill_rect(inner_x, inner_y, inner_w, g.frame_h - 2, "black")
    draw_show_section(g)
    draw_input_section(g)
    draw_controls(g)
end

local function render_header_only()
    if not ensure_terminal_size_ok() then
        return
    end
    if state.dirty then
        local g_full = frame_geometry()
        state.dirty = false
        render_full(g_full)
        return
    end
    local g = frame_geometry()
    draw_header(g)
end

local function render_show_only()
    if not ensure_terminal_size_ok() then
        return
    end
    if state.dirty then
        local g_full = frame_geometry()
        state.dirty = false
        render_full(g_full)
        return
    end
    local g = frame_geometry()
    draw_show_section(g)
end

local function render_input_only()
    if not ensure_terminal_size_ok() then
        return
    end
    if state.dirty then
        local g_full = frame_geometry()
        state.dirty = false
        render_full(g_full)
        return
    end
    local g = frame_geometry()
    draw_input_section(g)
end

local function render_if_needed(force)
    if not ensure_terminal_size_ok() then
        return
    end
    if force or state.dirty then
        state.dirty = false
        local g = frame_geometry()
        render_full(g)
    end
end

local function pause_with_render(ms)
    render_show_only()
    sleep(ms)
    advance_time(ms)
    render_header_only()
end

local function generate_sequence(round_no)
    local out = {}
    for _ = 1, round_no do
        out[#out + 1] = random(4) + 1
    end
    return out
end

local function show_sequence_blocking()
    state.phase = "show"
    state.highlight_idx = 0
    render_show_only()
    pause_with_render(SHOW_OFF_MS)

    for i = 1, #state.sequence do
        state.highlight_idx = state.sequence[i]
        pause_with_render(SHOW_ON_MS)

        state.highlight_idx = 0
        pause_with_render(SHOW_OFF_MS)
    end

    flush_input_buffer()
    state.phase = "input"
    state.highlight_idx = 0
    render_show_only()
end

local function start_next_round()
    state.input_colors = {}
    render_input_only()
    state.sequence = generate_sequence(state.round)
    show_sequence_blocking()
end

local function start_new_run()
    state.score = 0
    state.round = 1
    state.sequence = {}
    state.input_colors = {}
    state.highlight_idx = 0

    state.phase = "input"
    state.lost = false
    state.confirm_mode = nil
    state.committed = false

    state.start_frame = state.frame
    state.end_frame = nil
    state.dirty = true

    start_next_round()
end

local function mark_lost()
    if state.lost then
        return
    end
    state.lost = true
    state.phase = "lost"
    state.end_frame = state.frame
    state.confirm_mode = nil
    commit_stats_if_needed()
    state.dirty = true
end

local function on_round_success()
    state.score = state.score + state.round
    state.round = state.round + 1
    start_next_round()
end

local function refresh_dirty_flags()
    local elapsed = elapsed_seconds()
    if elapsed ~= state.last_elapsed_sec then
        state.last_elapsed_sec = elapsed
        render_header_only()
    end
end

local function handle_confirm_key(key)
    if key == "y" or key == "enter" then
        if state.confirm_mode == "restart" then
            commit_stats_if_needed()
            start_new_run()
            return "changed"
        end
        if state.confirm_mode == "exit" then
            commit_stats_if_needed()
            return "exit"
        end
    elseif key == "n" or key == "q" or key == "esc" then
        state.confirm_mode = nil
        state.dirty = true
        return "changed"
    end
    return "none"
end

local function handle_input(key)
    if key == nil or key == "" then
        return "none"
    end

    if state.confirm_mode ~= nil then
        return handle_confirm_key(key)
    end

    if state.lost then
        if key == "r" then
            start_new_run()
            return "changed"
        end
        if key == "q" or key == "esc" then
            commit_stats_if_needed()
            return "exit"
        end
        return "none"
    end

    if key == "q" or key == "esc" then
        state.confirm_mode = "exit"
        state.dirty = true
        return "changed"
    end
    if key == "r" then
        state.confirm_mode = "restart"
        state.dirty = true
        return "changed"
    end

    if state.phase ~= "input" then
        return "none"
    end

    if key == "backspace" or key == "delete" then
        if #state.input_colors > 0 then
            table.remove(state.input_colors)
            render_input_only()
        end
        return "changed"
    end

    local color_idx = nil
    if key == "1" then color_idx = 1 end
    if key == "2" then color_idx = 2 end
    if key == "3" then color_idx = 3 end
    if key == "4" then color_idx = 4 end

    if color_idx ~= nil then
        state.input_colors[#state.input_colors + 1] = color_idx
        render_input_only()
        return "changed"
    end

    if key == "enter" then
        local ok = #state.input_colors == #state.sequence
        if ok then
            for i = 1, #state.sequence do
                if state.input_colors[i] ~= state.sequence[i] then
                    ok = false
                    break
                end
            end
        end

        if not ok then
            mark_lost()
        else
            on_round_success()
        end
        return "changed"
    end

    return "none"
end

local function init_game()
    clear()
    flush_input_buffer()
    local w, h = terminal_size()
    state.last_term_w, state.last_term_h = w, h
    load_best_record()
    start_new_run()
end

local function game_loop()
    while state.running do
        local key = normalize_key(get_key(false))
        local action = handle_input(key)
        if action == "exit" then
            return
        end

        refresh_dirty_flags()
        render_if_needed(false)

        state.frame = state.frame + 1
        sleep(FRAME_MS)
    end
end

init_game()
game_loop()
