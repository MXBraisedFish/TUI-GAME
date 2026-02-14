GAME_META = {
    name = "Rock Paper Scissors",
    description = "Challenge the computer in classic rock-paper-scissors rounds."
}

local FRAME_MS = 16

local CHOICES = {
    [1] = { symbol = "Y", key = "game.rock_paper_scissors.choice.scissors", fallback = "Scissors" },
    [2] = { symbol = "O", key = "game.rock_paper_scissors.choice.rock", fallback = "Rock" },
    [3] = { symbol = "U", key = "game.rock_paper_scissors.choice.paper", fallback = "Paper" }
}

local state = {
    player_pick = nil,
    ai_pick = nil,
    current_streak = 0,
    best_streak = 0,
    message = "",
    message_color = "dark_gray",
    dirty = true,
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

local function normalize_key(key)
    if key == nil then return "" end
    if type(key) == "string" then return string.lower(key) end
    return tostring(key):lower()
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

local function wrap_words(text, max_width)
    if max_width <= 1 then
        return { text }
    end
    local lines = {}
    local current = ""
    local had = false
    for token in string.gmatch(text, "%S+") do
        had = true
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
    if not had then
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

local function centered_x(text, area_x, area_w)
    local x = area_x + math.floor((area_w - key_width(text)) / 2)
    if x < area_x then x = area_x end
    return x
end

local function save_best()
    if type(save_data) == "function" then
        pcall(save_data, "rock_paper_scissors_best", { best_streak = state.best_streak })
    end
    if type(update_game_stats) == "function" then
        pcall(update_game_stats, "rock_paper_scissors", state.best_streak, 0)
    end
end

local function load_best()
    if type(load_data) ~= "function" then
        return
    end
    local ok, data = pcall(load_data, "rock_paper_scissors_best")
    if not ok or type(data) ~= "table" then
        return
    end
    local v = tonumber(data.best_streak)
    if v ~= nil and v >= 0 then
        state.best_streak = math.floor(v)
    end
end

local function choice_text(index)
    if index == nil or CHOICES[index] == nil then
        return "-"
    end
    local info = CHOICES[index]
    return info.symbol .. " " .. tr(info.key, info.fallback)
end

local function resolve_round(player_idx, ai_idx)
    if player_idx == ai_idx then
        return 0
    end
    if (player_idx == 1 and ai_idx == 3)
        or (player_idx == 2 and ai_idx == 1)
        or (player_idx == 3 and ai_idx == 2) then
        return 1
    end
    return -1
end

local function play_round(player_idx)
    local ai_idx = random(3) + 1
    state.player_pick = player_idx
    state.ai_pick = ai_idx

    local result = resolve_round(player_idx, ai_idx)
    local controls = tr("game.rock_paper_scissors.result_controls", "[1][2][3] Next  [R] Restart  [Q]/[ESC] Exit")
    if result > 0 then
        state.current_streak = state.current_streak + 1
        if state.current_streak > state.best_streak then
            state.best_streak = state.current_streak
            save_best()
        end
        state.message = tr("game.rock_paper_scissors.win_banner", "You win!") .. " " .. controls
        state.message_color = "green"
    elseif result < 0 then
        state.current_streak = 0
        state.message = tr("game.rock_paper_scissors.lose_banner", "You lose!") .. " " .. controls
        state.message_color = "red"
    else
        state.current_streak = 0
        state.message = tr("game.rock_paper_scissors.draw_banner", "Draw!") .. " " .. controls
        state.message_color = "yellow"
    end

    state.dirty = true
end

local function reset_round()
    state.player_pick = nil
    state.ai_pick = nil
    state.current_streak = 0
    state.message = tr("game.rock_paper_scissors.ready_banner", "Make your move.")
    state.message_color = "dark_gray"
    state.dirty = true
end

local function minimum_required_size()
    local top1 = tr("game.rock_paper_scissors.best_streak", "Best Win Streak") .. ": 9999"
    local top2 = tr("game.rock_paper_scissors.current_streak", "Current Streak") .. ": 9999"
    local header = tr("game.rock_paper_scissors.player", "Player") .. "   |   " .. tr("game.rock_paper_scissors.system", "System")
    local picks = "Y " .. tr("game.rock_paper_scissors.choice.scissors", "Scissors") .. "   |   O " .. tr("game.rock_paper_scissors.choice.rock", "Rock")
    local msg = tr("game.rock_paper_scissors.win_banner", "You win!") .. " "
        .. tr("game.rock_paper_scissors.result_controls", "[1][2][3] Next  [R] Restart  [Q]/[ESC] Exit")
    local controls = tr(
        "game.rock_paper_scissors.controls",
        "[1]=Scissors  [2]=Rock  [3]=Paper  [R] Restart  [Q]/[ESC] Exit"
    )
    local controls_w = min_width_for_lines(controls, 3, 24)
    local min_w = math.max(key_width(top1), key_width(top2), key_width(header), key_width(picks), key_width(msg), controls_w) + 2
    local min_h = 10
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

local function draw_controls(y)
    local controls = tr(
        "game.rock_paper_scissors.controls",
        "[1]=Scissors  [2]=Rock  [3]=Paper  [R] Restart  [Q]/[ESC] Exit"
    )
    local term_w = terminal_size()
    local lines = wrap_words(controls, math.max(10, term_w - 2))
    if #lines > 3 then
        lines = { lines[1], lines[2], lines[3] }
    end
    for i = 1, 3 do
        draw_text(1, y + i - 1, string.rep(" ", term_w), "white", "black")
    end
    local offset = 0
    if #lines < 3 then
        offset = math.floor((3 - #lines) / 2)
    end
    for i = 1, #lines do
        local line = lines[i]
        local x = math.floor((term_w - key_width(line)) / 2)
        if x < 1 then x = 1 end
        draw_text(x, y + offset + i - 1, line, "white", "black")
    end
end

local function render()
    local term_w, term_h = terminal_size()
    local total_h = 8
    local y0 = math.floor((term_h - total_h) / 2) + 1
    if y0 < 1 then y0 = 1 end

    clear()

    local top1 = tr("game.rock_paper_scissors.best_streak", "Best Win Streak") .. ": " .. tostring(state.best_streak)
    local top2 = tr("game.rock_paper_scissors.current_streak", "Current Streak") .. ": " .. tostring(state.current_streak)
    draw_text(centered_x(top1, 1, term_w), y0, top1, "dark_gray", "black")
    draw_text(centered_x(top2, 1, term_w), y0 + 1, top2, "light_cyan", "black")

    if state.message ~= "" then
        draw_text(centered_x(state.message, 1, term_w), y0 + 2, state.message, state.message_color, "black")
    end

    local header = tr("game.rock_paper_scissors.player", "Player") .. "   |   " .. tr("game.rock_paper_scissors.system", "System")
    local line = choice_text(state.player_pick) .. "   |   " .. choice_text(state.ai_pick)
    draw_text(centered_x(header, 1, term_w), y0 + 4, header, "white", "black")
    draw_text(centered_x(line, 1, term_w), y0 + 5, line, "white", "black")

    draw_controls(y0 + 7)
end

local function handle_input(key)
    if key == nil or key == "" then
        return "none"
    end
    if key == "q" or key == "esc" then
        return "exit"
    end
    if key == "r" then
        reset_round()
        return "changed"
    end
    if key == "1" or key == "2" or key == "3" then
        play_round(tonumber(key))
        return "changed"
    end
    return "none"
end

local function init_game()
    local w, h = terminal_size()
    state.last_term_w = w
    state.last_term_h = h
    load_best()
    reset_round()
    if type(clear_input_buffer) == "function" then
        pcall(clear_input_buffer)
    end
end

local function sync_terminal_resize()
    local w, h = terminal_size()
    if w ~= state.last_term_w or h ~= state.last_term_h then
        state.last_term_w = w
        state.last_term_h = h
        state.dirty = true
    end
end

local function game_loop()
    while true do
        local key = normalize_key(get_key(false))
        if ensure_terminal_size_ok() then
            local action = handle_input(key)
            if action == "exit" then
                save_best()
                return
            end
            sync_terminal_resize()
            if state.dirty then
                render()
                state.dirty = false
            end
        else
            if key == "q" or key == "esc" then
                save_best()
                return
            end
        end
        sleep(FRAME_MS)
    end
end

init_game()
game_loop()
