GAME_META = {
    name = "Wordle",
    description = "Guess the hidden word using color hints from each attempt."
}

local FPS, FRAME_MS = 60, 16
local MAX_ATTEMPTS = 5

local S = {
    words = {},
    secret = "",
    word_len = 5,
    guesses = {},
    marks = {},
    input = "",
    mode = "input", -- input | action
    confirm = nil, -- restart | exit
    settled = false,
    won = false,

    streak = 0,
    best_time_sec = 0,

    frame = 0,
    start_frame = 0,
    end_frame = nil,

    toast = nil,
    toast_color = "green",
    toast_until = 0,

    dirty = true,
    time_dirty = false,
    last_elapsed = -1,
    last_time_line = "",

    tw = 0,
    th = 0,
    warn = false,
    lw = 0,
    lh = 0,
    lmw = 0,
    lmh = 0,
}

local function tr(k, d)
    if type(translate) ~= "function" then return d end
    local ok, v = pcall(translate, k)
    if (not ok) or v == nil or v == "" or v == k then return d end
    if type(v) == "string" and string.find(v, "missing-i18n-key", 1, true) then return d end
    return v
end

local function key(k)
    if k == nil then return "" end
    if type(k) == "string" then return string.lower(k) end
    if type(k) == "table" and type(k.code) == "string" then return string.lower(k.code) end
    return tostring(k):lower()
end

local function wid(t)
    if type(get_text_width) == "function" then
        local ok, w = pcall(get_text_width, t)
        if ok and type(w) == "number" then return w end
    end
    return #t
end

local function ts()
    local w, h = 120, 40
    if type(get_terminal_size) == "function" then
        local tw, th = get_terminal_size()
        if type(tw) == "number" and type(th) == "number" then w, h = tw, th end
    end
    return w, h
end

local function sec()
    local ef = S.end_frame or S.frame
    return math.max(0, math.floor((ef - S.start_frame) / FPS))
end

local function fmt(s)
    local h = math.floor(s / 3600)
    local m = math.floor((s % 3600) / 60)
    local x = s % 60
    return string.format("%02d:%02d:%02d", h, m, x)
end

local function rnd(n)
    if n <= 0 then return 0 end
    if type(random) == "function" then return random(n) end
    return math.random(0, n - 1)
end

local function cx(text, l, r)
    local x = l + math.floor(((r - l + 1) - wid(text)) / 2)
    if x < l then x = l end
    return x
end

local function clear_line(y, tw)
    draw_text(1, y, string.rep(" ", tw), "white", "black")
end

local function read_file(path)
    if not io or not io.open then return nil end
    local f = io.open(path, "r")
    if not f then return nil end
    local data = f:read("*a")
    f:close()
    return data
end

local function load_words()
    local raw = read_file("assets/wordle/word.json")
    local words, seen = {}, {}

    if type(raw) == "string" then
        for w in raw:gmatch('"([A-Za-z]+)"') do
            local lw = string.lower(w)
            if #lw >= 2 and not seen[lw] then
                seen[lw] = true
                words[#words + 1] = lw
            end
        end
    end

    if #words == 0 then
        words = { "apple", "water", "green", "house", "sound", "light", "story", "music", "table", "clock" }
    end

    S.words = words
end

local function pick_word()
    if #S.words == 0 then load_words() end
    local idx = rnd(#S.words) + 1
    local w = S.words[idx]
    S.secret = string.lower(w)
    S.word_len = #S.secret
end

local function char_at(str, i)
    return string.sub(str, i, i)
end

local function evaluate_guess(secret, guess)
    local n = #secret
    local marks, pool = {}, {}

    for i = 1, n do
        local c = char_at(secret, i)
        pool[c] = (pool[c] or 0) + 1
        marks[i] = "absent"
    end

    for i = 1, n do
        local g = char_at(guess, i)
        local s = char_at(secret, i)
        if g == s then
            marks[i] = "correct"
            pool[g] = (pool[g] or 0) - 1
        end
    end

    for i = 1, n do
        if marks[i] ~= "correct" then
            local g = char_at(guess, i)
            local cnt = pool[g] or 0
            if cnt > 0 then
                marks[i] = "present"
                pool[g] = cnt - 1
            else
                marks[i] = "absent"
            end
        end
    end

    return marks
end

local function save_best_time()
    if type(save_data) == "function" then
        pcall(save_data, "wordle_best_time_sec", S.best_time_sec)
    end
end

local function save_streak()
    if type(save_data) == "function" then
        pcall(save_data, "wordle_streak", S.streak)
    end
end

local function load_meta()
    if type(load_data) ~= "function" then return end

    local ok1, bt = pcall(load_data, "wordle_best_time_sec")
    if ok1 and type(bt) == "number" and bt > 0 then
        S.best_time_sec = math.floor(bt)
    end

    local ok2, st = pcall(load_data, "wordle_streak")
    if ok2 and type(st) == "number" and st >= 0 then
        S.streak = math.floor(st)
    end
end

local function save_slot()
    if type(save_game_slot) ~= "function" then return end
    local payload = {
        secret = S.secret,
        guesses = S.guesses,
        input = S.input,
        mode = S.mode,
        streak = S.streak,
        best_time_sec = S.best_time_sec,
        elapsed_sec = sec(),
        settled = S.settled,
        won = S.won,
    }
    pcall(save_game_slot, "wordle", payload)
    S.toast = tr("game.wordle.saved", "Saved.")
    S.toast_color = "green"
    S.toast_until = S.frame + FPS * 2
end

local function load_slot_if_continue()
    if type(get_launch_mode) ~= "function" or type(load_game_slot) ~= "function" then return false end
    local mode = string.lower(tostring(get_launch_mode()))
    if mode ~= "continue" then return false end

    local ok, slot = pcall(load_game_slot, "wordle")
    if not ok or type(slot) ~= "table" then return false end
    if type(slot.secret) ~= "string" or slot.secret == "" then return false end
    if slot.settled then return false end

    S.secret = string.lower(slot.secret)
    S.word_len = #S.secret
    S.guesses = {}
    S.marks = {}

    if type(slot.guesses) == "table" then
        for i = 1, #slot.guesses do
            local g = tostring(slot.guesses[i]):lower()
            if #g == S.word_len then
                S.guesses[#S.guesses + 1] = g
                S.marks[#S.marks + 1] = evaluate_guess(S.secret, g)
            end
        end
    end

    S.input = type(slot.input) == "string" and string.lower(slot.input) or ""
    if #S.input > S.word_len then
        S.input = string.sub(S.input, 1, S.word_len)
    end

    S.mode = (slot.mode == "action") and "action" or "input"
    if type(slot.streak) == "number" and slot.streak >= 0 then
        S.streak = math.floor(slot.streak)
    end
    if type(slot.best_time_sec) == "number" and slot.best_time_sec > 0 then
        S.best_time_sec = math.floor(slot.best_time_sec)
    end

    local elapsed = 0
    if type(slot.elapsed_sec) == "number" and slot.elapsed_sec >= 0 then
        elapsed = math.floor(slot.elapsed_sec)
    end
    S.start_frame = S.frame - elapsed * FPS
    S.end_frame = nil
    S.settled = false
    S.won = false
    return true
end

local function new_round(preserve_streak)
    if not preserve_streak then
        S.streak = 0
        save_streak()
    end

    pick_word()
    S.guesses = {}
    S.marks = {}
    S.input = ""
    S.mode = "input"
    S.confirm = nil
    S.settled = false
    S.won = false
    S.start_frame = S.frame
    S.end_frame = nil
    S.toast = nil
    S.dirty = true
end

local function settle(win)
    S.settled = true
    S.won = win
    S.end_frame = S.frame

    if win then
        S.streak = S.streak + 1
        local t = sec()
        if S.best_time_sec <= 0 or t < S.best_time_sec then
            S.best_time_sec = t
            save_best_time()
        end
        save_streak()
        if type(update_game_stats) == "function" then
            pcall(update_game_stats, "wordle", S.streak, t)
        end
    else
        S.streak = 0
        save_streak()
    end
end

local function status_text()
    if S.confirm == "restart" then
        return tr("game.wordle.confirm_restart", "Confirm restart? [Y] Yes / [N] No"), "yellow"
    end
    if S.confirm == "exit" then
        return tr("game.wordle.confirm_exit", "Confirm exit? [Y] Yes / [N] No"), "yellow"
    end
    if S.settled then
        if S.won then
            return tr("game.wordle.win", "Guessed the correct word!") .. "  " .. tr("game.wordle.result_controls", "[R] Restart  [Q]/[ESC] Exit"), "green"
        end
        return tr("game.wordle.lose", "You did not guess the word.") .. "  " .. tr("game.wordle.result_controls", "[R] Restart  [Q]/[ESC] Exit"), "red"
    end
    if S.toast and S.frame <= S.toast_until then
        return S.toast, S.toast_color
    end
    if S.mode == "action" then
        return tr("game.wordle.mode_action", "Action Mode"), "yellow"
    end
    return tr("game.wordle.mode_input", "Letter Input Mode"), "dark_gray"
end

local function controls_text()
    if S.settled then
        return tr("game.wordle.controls_result", "[R] Restart  [Q]/[ESC] Exit")
    end
    if S.mode == "action" then
        return tr("game.wordle.controls_action", "[Tab] Input Mode  [S] Save  [R] Restart  [Q]/[ESC] Exit")
    end
    return tr("game.wordle.controls_input", "[A-Z] Input  [Backspace]/[Delete] Remove  [Enter] Submit  [Tab] Action Mode")
end

local function min_size()
    local cw = wid(controls_text())
    local row_w = wid("-> ") + S.word_len * 2 + 2
    local top_w = math.max(
        wid(tr("game.wordle.best_time", "Best Time") .. " " .. fmt(0)),
        wid(tr("game.wordle.time", "Time") .. " " .. fmt(0) .. "  " .. tr("game.wordle.streak", "Streak") .. " 999")
    )
    local need_w = math.max(60, cw + 2, row_w + 8, top_w + 2)
    return need_w, 14
end

local function draw_warn(tw, th, mw, mh)
    clear()
    local ls = {
        tr("warning.size_title", "Terminal Too Small"),
        string.format("%s: %dx%d", tr("warning.required", "Required size"), mw, mh),
        string.format("%s: %dx%d", tr("warning.current", "Current size"), tw, th),
        tr("warning.enlarge_hint", "Please enlarge terminal window to continue.")
    }
    local top = math.floor((th - #ls) / 2)
    if top < 1 then top = 1 end
    for i = 1, #ls do
        draw_text(cx(ls[i], 1, tw), top + i - 1, ls[i], "white", "black")
    end
end

local function size_ok()
    local tw, th = ts()
    local mw, mh = min_size()
    if tw >= mw and th >= mh then
        if S.warn then clear(); S.dirty = true end
        if tw ~= S.tw or th ~= S.th then clear(); S.dirty = true end
        S.tw, S.th, S.warn = tw, th, false
        return true
    end
    local changed = (not S.warn) or S.lw ~= tw or S.lh ~= th or S.lmw ~= mw or S.lmh ~= mh
    if changed then
        draw_warn(tw, th, mw, mh)
        S.lw, S.lh, S.lmw, S.lmh = tw, th, mw, mh
    end
    S.warn = true
    return false
end

local function top_time_line()
    return tr("game.wordle.time", "Time") .. " " .. fmt(sec()) .. "  " .. tr("game.wordle.streak", "Streak") .. " " .. tostring(S.streak)
end

local function draw_guess_row(y, tw, idx)
    local prefix = "-> "
    local guess = S.guesses[idx]
    local marks = S.marks[idx]

    local width = wid(prefix) + S.word_len * 2
    local x = cx(string.rep(" ", width), 1, tw)

    draw_text(x, y, prefix, "white", "black")
    x = x + wid(prefix)

    for i = 1, S.word_len do
        local ch = " "
        local fg, bg = "white", "black"

        if type(guess) == "string" then
            ch = string.upper(char_at(guess, i))
            local mark = marks and marks[i] or "absent"
            if mark == "correct" then
                fg, bg = "black", "green"
            elseif mark == "present" then
                fg, bg = "black", "yellow"
            else
                fg, bg = "dark_gray", "black"
            end
        end

        draw_text(x, y, ch, fg, bg)
        draw_text(x + 1, y, " ", "white", "black")
        x = x + 2
    end
end

local function draw_input_row(y, tw)
    local prefix = "  "
    local width = wid(prefix) + S.word_len * 2
    local x = cx(string.rep(" ", width), 1, tw)

    draw_text(x, y, prefix, "white", "black")
    x = x + wid(prefix)

    local show = S.input
    if S.settled then
        show = S.secret
    end

    for i = 1, S.word_len do
        local ch, fg = "_", "dark_gray"
        if i <= #show then
            ch = string.upper(char_at(show, i))
            if S.settled then
                fg = S.won and "green" or "red"
            else
                fg = "white"
            end
        end

        draw_text(x, y, ch, fg, "black")
        draw_text(x + 1, y, " ", "white", "black")
        x = x + 2
    end
end

local function render()
    local tw, th = ts()
    local top = math.floor((th - 12) / 2)
    if top < 1 then top = 1 end

    local best = tr("game.wordle.best_time", "Best Time") .. " " .. ((S.best_time_sec > 0) and fmt(S.best_time_sec) or tr("game.twenty_four.none", "--:--:--"))
    local tline = top_time_line()
    local msg, mc = status_text()

    for i = 0, 2 do clear_line(top + i, tw) end
    draw_text(cx(best, 1, tw), top, best, "dark_gray", "black")
    draw_text(cx(tline, 1, tw), top + 1, tline, "light_cyan", "black")
    S.last_time_line = tline
    draw_text(cx(msg, 1, tw), top + 2, msg, mc, "black")

    local y0 = top + 4
    for i = 0, MAX_ATTEMPTS do clear_line(y0 + i, tw) end
    for i = 1, MAX_ATTEMPTS do
        draw_guess_row(y0 + i - 1, tw, i)
    end
    draw_input_row(y0 + MAX_ATTEMPTS, tw)

    local controls = controls_text()
    clear_line(y0 + MAX_ATTEMPTS + 2, tw)
    draw_text(cx(controls, 1, tw), y0 + MAX_ATTEMPTS + 2, controls, "white", "black")
end

local function render_time_only()
    local tw, th = ts()
    local top = math.floor((th - 12) / 2)
    if top < 1 then top = 1 end
    local tline = top_time_line()

    local cw = math.max(wid(S.last_time_line or ""), wid(tline))
    local x = cx(string.rep(" ", cw), 1, tw)
    draw_text(x, top + 1, string.rep(" ", cw), "white", "black")
    draw_text(cx(tline, 1, tw), top + 1, tline, "light_cyan", "black")
    S.last_time_line = tline
end

local function apply_guess()
    if #S.input ~= S.word_len then
        S.toast = tr("game.wordle.need_letters", "Not enough letters.")
        S.toast_color = "red"
        S.toast_until = S.frame + FPS * 2
        S.dirty = true
        return
    end

    local guess = string.lower(S.input)
    local marks = evaluate_guess(S.secret, guess)
    S.guesses[#S.guesses + 1] = guess
    S.marks[#S.marks + 1] = marks
    S.input = ""

    if guess == S.secret then
        settle(true)
    elseif #S.guesses >= MAX_ATTEMPTS then
        settle(false)
    end

    S.dirty = true
end

local function refresh_flags()
    local e = sec()
    if e ~= S.last_elapsed then
        S.last_elapsed = e
        S.time_dirty = true
    end

    local tv = S.toast ~= nil and S.frame <= S.toast_until
    if (not tv) and S.toast ~= nil then
        S.toast = nil
        S.dirty = true
    end
end

local function handle_confirm(k)
    if k == "y" or k == "enter" then
        if S.confirm == "restart" then
            S.confirm = nil
            new_round(false)
            return "changed"
        end
        if S.confirm == "exit" then
            return "exit"
        end
    end

    if k == "n" or k == "q" or k == "esc" then
        S.confirm = nil
        S.dirty = true
        return "changed"
    end
    return "none"
end

local function handle_playing_key(k)
    if S.mode == "input" then
        if k == "tab" then
            S.mode = "action"
            S.dirty = true
            return "changed"
        end
        if k == "backspace" or k == "delete" then
            if #S.input > 0 then
                S.input = string.sub(S.input, 1, #S.input - 1)
                S.dirty = true
            end
            return "changed"
        end
        if k == "enter" then
            apply_guess()
            return "changed"
        end
        if k:match("^[a-z]$") then
            if #S.input < S.word_len then
                S.input = S.input .. k
                S.dirty = true
            end
            return "changed"
        end
        return "none"
    end

    -- action mode
    if k == "tab" then
        S.mode = "input"
        S.dirty = true
        return "changed"
    end
    if k == "s" then
        save_slot()
        S.dirty = true
        return "changed"
    end
    if k == "r" then
        S.confirm = "restart"
        S.dirty = true
        return "changed"
    end
    if k == "q" or k == "esc" then
        S.confirm = "exit"
        S.dirty = true
        return "changed"
    end
    return "none"
end

local function handle_settled_key(k)
    if k == "r" then
        new_round(S.won)
        return "changed"
    end
    if k == "q" or k == "esc" then
        return "exit"
    end
    if k == "tab" then
        S.mode = (S.mode == "input") and "action" or "input"
        S.dirty = true
        return "changed"
    end
    return "none"
end

local function init()
    clear()
    if type(clear_input_buffer) == "function" then pcall(clear_input_buffer) end

    load_meta()
    load_words()

    if not load_slot_if_continue() then
        new_round(true)
    end

    S.frame = 0
    S.last_elapsed = sec()
    S.time_dirty = false
    S.dirty = true
end

local function loop()
    while true do
        if not size_ok() then
            sleep(FRAME_MS)
            S.frame = S.frame + 1
        else
            local k = key(get_key(false))
            local a = "none"

            if k ~= "" then
                if S.confirm then
                    a = handle_confirm(k)
                elseif S.settled then
                    a = handle_settled_key(k)
                else
                    a = handle_playing_key(k)
                end

                if a == "exit" then
                    exit_game()
                    return
                end
            end

            refresh_flags()
            if S.dirty then
                render()
                S.dirty = false
                S.time_dirty = false
            elseif S.time_dirty then
                render_time_only()
                S.time_dirty = false
            end

            sleep(FRAME_MS)
            S.frame = S.frame + 1
        end
    end
end

init()
loop()
