
GAME_META = {
    name = "24 Points",
    description = "Use A/J/Q/K and + - * / () to form an expression equal to 24."
}

local FPS, FRAME_MS, EPS = 60, 16, 1e-6
local M_CLASSIC, M_FIXED_NEG, M_FLEX_NEG = 1, 2, 3
local OP_EMPTY = "_"
local PAREN_COLORS = { "magenta", "#d8b4fe", "light_blue", "light_green" }

local S = {
    mode = M_CLASSIC,
    base_nums = { 1, 2, 3, 4 },
    nums = { 1, 2, 3, 4 },
    ops = { OP_EMPTY, OP_EMPTY, OP_EMPTY, OP_EMPTY },
    pairs = {},
    cursor = 1,
    frame = 0,
    start_frame = 0,
    end_frame = nil,
    steps = 0,
    ready = false,
    value = nil,
    win = false,
    confirm = nil,
    input_mode = nil,
    input_buf = "",
    toast = nil,
    toast_color = "green",
    toast_until = 0,
    best_time = 0,
    committed = false,
    dirty = true,
    last_elapsed = -1,
    last_toast = false,
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
    local e = S.end_frame or S.frame
    return math.max(0, math.floor((e - S.start_frame) / FPS))
end

local function fmt(s)
    local h = math.floor(s / 3600)
    local m = math.floor((s % 3600) / 60)
    local x = s % 60
    return string.format("%02d:%02d:%02d", h, m, x)
end

local function rnd(n)
    if n <= 0 or type(random) ~= "function" then return 0 end
    return random(n)
end

local function cx(text, l, r)
    local x = l + math.floor(((r - l + 1) - wid(text)) / 2)
    if x < l then x = l end
    return x
end

local function wrap(t, mw)
    if mw <= 1 then return { t } end
    local ls, cur, had = {}, "", false
    for tok in string.gmatch(t, "%S+") do
        had = true
        if cur == "" then cur = tok else
            local c = cur .. " " .. tok
            if wid(c) <= mw then cur = c else ls[#ls + 1] = cur; cur = tok end
        end
    end
    if not had then return { "" } end
    if cur ~= "" then ls[#ls + 1] = cur end
    return ls
end

local function minw(t, ml, hm)
    local f, w = wid(t), hm
    while w <= f do if #wrap(t, w) <= ml then return w end; w = w + 1 end
    return f
end

local function mode_name(m)
    if m == M_FIXED_NEG then return tr("game.twenty_four.mode.fixed_negative", "Fixed Negative") end
    if m == M_FLEX_NEG then return tr("game.twenty_four.mode.flex_negative", "Flexible Negative") end
    return tr("game.twenty_four.mode.classic", "Classic")
end

local function active_list()
    if S.mode == M_FLEX_NEG then
        return {
            { k = "op", i = 1 }, { k = "num", i = 1 }, { k = "op", i = 2 }, { k = "num", i = 2 },
            { k = "op", i = 3 }, { k = "num", i = 3 }, { k = "op", i = 4 }, { k = "num", i = 4 },
        }
    end
    return { { k = "op", i = 1 }, { k = "op", i = 2 }, { k = "op", i = 3 }, { k = "op", i = 4 } }
end

local function focus()
    local ls = active_list()
    if S.cursor < 1 then S.cursor = 1 end
    if S.cursor > #ls then S.cursor = #ls end
    return ls[S.cursor], #ls
end

local function can24(nums)
    if #nums == 1 then return math.abs(nums[1] - 24) < EPS end
    for i = 1, #nums do
        for j = i + 1, #nums do
            local a, b, rest = nums[i], nums[j], {}
            for k = 1, #nums do if k ~= i and k ~= j then rest[#rest + 1] = nums[k] end end
            local cand = { a + b, a - b, b - a, a * b }
            if math.abs(b) > EPS then cand[#cand + 1] = a / b end
            if math.abs(a) > EPS then cand[#cand + 1] = b / a end
            for c = 1, #cand do
                local n = { table.unpack(rest) }
                n[#n + 1] = cand[c]
                if can24(n) then return true end
            end
        end
    end
    return false
end

local function has_solution(nums, mode)
    if mode == M_FLEX_NEG then
        local absn = { math.abs(nums[1]), math.abs(nums[2]), math.abs(nums[3]), math.abs(nums[4]) }
        for mask = 0, 15 do
            local t = {}
            for i = 1, 4 do
                local bit = math.floor(mask / (2 ^ (i - 1))) % 2
                t[i] = (bit == 1) and -absn[i] or absn[i]
            end
            if can24(t) then return true end
        end
        return false
    end
    return can24({ nums[1], nums[2], nums[3], nums[4] })
end

local function rand_num(mode)
    local v = rnd(13) + 1
    if mode == M_CLASSIC then return v end
    return (rnd(100) < 50) and -v or v
end

local function load_best()
    S.best_time = 0
    if type(load_data) ~= "function" then return end
    local ok, d = pcall(load_data, "twenty_four_best_time")
    if not ok then return end
    if type(d) == "number" then S.best_time = math.max(0, math.floor(d)); return end
    if type(d) == "table" then
        local s = tonumber(d.time_sec) or tonumber(d.best_time_sec) or 0
        S.best_time = math.max(0, math.floor(s))
    end
end

local function save_best()
    if type(save_data) == "function" then pcall(save_data, "twenty_four_best_time", { time_sec = S.best_time }) end
end

local function commit_once()
    if S.committed then return end
    S.committed = true
    local t = sec()
    if S.best_time <= 0 or t < S.best_time then S.best_time = t; save_best() end
    if type(update_game_stats) == "function" then
        local score = math.max(0, 1000000 - t * 100 - S.steps)
        pcall(update_game_stats, "twenty_four", score, t)
    end
end

local function reset_round(mode)
    S.mode = mode or S.mode
    local guard = 0
    while true do
        guard = guard + 1
        local n = { rand_num(S.mode), rand_num(S.mode), rand_num(S.mode), rand_num(S.mode) }
        if has_solution(n, S.mode) or guard > 2000 then
            S.base_nums = { n[1], n[2], n[3], n[4] }
            S.nums = { n[1], n[2], n[3], n[4] }
            break
        end
    end
    S.ops = { OP_EMPTY, OP_EMPTY, OP_EMPTY, OP_EMPTY }
    S.pairs = {}
    S.cursor = 1
    S.steps = 0
    S.ready = false
    S.value = nil
    S.win = false
    S.confirm = nil
    S.input_mode = nil
    S.input_buf = ""
    S.end_frame = nil
    S.start_frame = S.frame
    S.committed = false
    S.dirty = true
end
local function cross(l1, r1, l2, r2)
    return (l1 < l2 and l2 < r1 and r1 < r2) or (l2 < l1 and l1 < r2 and r2 < r1)
end

local function pair_map()
    local L, R = {}, {}
    for i = 1, 9 do L[i], R[i] = {}, {} end
    for c = 1, 4 do
        local p = S.pairs[c]
        if p then
            L[p.l][#L[p.l] + 1] = { c = c, l = p.l, r = p.r }
            R[p.r][#R[p.r] + 1] = { c = c, l = p.l, r = p.r }
        end
    end
    for i = 1, 9 do
        table.sort(L[i], function(a, b) if a.l ~= b.l then return a.l < b.l end return a.r > b.r end)
        table.sort(R[i], function(a, b) if a.r ~= b.r then return a.r < b.r end return a.l > b.l end)
    end
    return L, R
end

local function add_pair(l, r)
    if l < 1 or r > 9 or l >= r then
        S.toast, S.toast_color, S.toast_until = tr("game.twenty_four.err_paren_order", "Use different coordinates: left < right."), "red", S.frame + FPS * 2
        S.dirty = true; return false
    end
    local nums, ops = 0, 0
    for p = l, r - 1 do if p % 2 == 0 then nums = nums + 1 else ops = ops + 1 end end
    if nums < 2 or ops < 1 then
        S.toast, S.toast_color, S.toast_until = tr("game.twenty_four.err_paren_single", "Brackets must cover at least two numbers and one operator."), "red", S.frame + FPS * 2
        S.dirty = true; return false
    end
    for i = 1, 4 do
        local p = S.pairs[i]
        if p and cross(l, r, p.l, p.r) then
            S.toast, S.toast_color, S.toast_until = tr("game.twenty_four.err_paren_cross", "Crossed brackets are not allowed."), "red", S.frame + FPS * 2
            S.dirty = true; return false
        end
        if p and p.l == l and p.r == r then
            S.toast, S.toast_color, S.toast_until = tr("game.twenty_four.err_paren_duplicate", "Same bracket pair already exists."), "red", S.frame + FPS * 2
            S.dirty = true; return false
        end
    end
    for i = 1, 4 do
        if S.pairs[i] == nil then
            S.pairs[i] = { l = l, r = r }
            S.steps = S.steps + 1
            S.dirty = true
            return true
        end
    end
    S.toast, S.toast_color, S.toast_until = tr("game.twenty_four.err_paren_full", "Only 4 bracket colors are available."), "red", S.frame + FPS * 2
    S.dirty = true
    return false
end

local function eval_expr()
    S.ready, S.value = false, nil
    for i = 1, 4 do if S.ops[i] == OP_EMPTY then return end end
    local toks = { S.ops[1], tostring(S.nums[1]), S.ops[2], tostring(S.nums[2]), S.ops[3], tostring(S.nums[3]), S.ops[4], tostring(S.nums[4]) }
    local L, R = pair_map()
    local parts = {}
    for b = 1, 9 do
        for i = 1, #R[b] do parts[#parts + 1] = ")" end
        for i = 1, #L[b] do parts[#parts + 1] = "(" end
        if b <= 8 then parts[#parts + 1] = toks[b] end
    end
    local expr = table.concat(parts, "")
    S.ready = true
    local fn = load("return " .. expr)
    if fn == nil then return end
    local ok, v = pcall(fn)
    if (not ok) or type(v) ~= "number" or v ~= v or v == math.huge or v == -math.huge then return end
    S.value = v
    if math.abs(v - 24) < EPS then S.win = true; S.end_frame = S.frame; commit_once() end
end

local function set_op(i, op)
    if S.ops[i] ~= op then S.ops[i] = op; S.steps = S.steps + 1; eval_expr(); S.dirty = true end
end

local function set_num_sign(i, sign)
    local v = math.abs(S.nums[i])
    local t = (sign < 0) and -v or v
    if S.nums[i] ~= t then S.nums[i] = t; S.steps = S.steps + 1; eval_expr(); S.dirty = true end
end

local function msg()
    if S.confirm == "restart" then return tr("game.twenty_four.confirm_restart", "Confirm restart? [Y] Yes / [N] No"), "yellow" end
    if S.confirm == "exit" then return tr("game.twenty_four.confirm_exit", "Confirm exit? [Y] Yes / [N] No"), "yellow" end
    if S.input_mode == "paren_add" then
        local t = tr("game.twenty_four.prompt_add_paren", "Add brackets: input X Y (1-9), Enter confirm")
        if S.input_buf ~= "" then t = t .. "  " .. S.input_buf end
        return t, "yellow"
    end
    if S.input_mode == "paren_remove" then
        local d = {}
        for i = 1, 4 do if S.pairs[i] then d[#d + 1] = tostring(i) .. "()" end end
        local t = tr("game.twenty_four.prompt_remove_paren", "Remove brackets: input color index 1-4")
        if #d > 0 then t = t .. "  " .. table.concat(d, " ") end
        if S.input_buf ~= "" then t = t .. "  " .. S.input_buf end
        return t, "yellow"
    end
    if S.input_mode == "difficulty" then
        local t = tr("game.twenty_four.prompt_difficulty", "Set difficulty: 1 Classic / 2 Fixed Negative / 3 Flexible Negative")
        if S.input_buf ~= "" then t = t .. "  " .. S.input_buf end
        return t, "yellow"
    end
    if S.win then return tr("game.twenty_four.win_banner", "You found the best expression!") .. "  " .. tr("game.twenty_four.result_controls", "[R] Restart  [Q]/[ESC] Exit"), "green" end
    if S.toast and S.frame <= S.toast_until then return S.toast, S.toast_color end
    return tr("game.twenty_four.ready", "Fill all operators to evaluate."), "dark_gray"
end

local function result_text()
    if not S.ready then return "?", "blue" end
    if S.value == nil then return tr("game.twenty_four.invalid", "ERR"), "red" end
    local iv = math.floor(S.value + 0.5)
    local t = (math.abs(S.value - iv) < 1e-9) and tostring(iv) or string.format("%.6g", S.value)
    return t, (math.abs(S.value - 24) < EPS) and "green" or "red"
end

local function render_mid(y, tw)
    local f = focus()
    local toks = { S.ops[1], tostring(S.nums[1]), S.ops[2], tostring(S.nums[2]), S.ops[3], tostring(S.nums[3]), S.ops[4], tostring(S.nums[4]) }
    local L, R = pair_map()
    local seg, tx, cur = {}, {}, 1
    for b = 1, 9 do
        for i = 1, #R[b] do seg[#seg + 1] = { t = ")", fg = PAREN_COLORS[R[b][i].c], bg = "black" }; cur = cur + 1 end
        for i = 1, #L[b] do seg[#seg + 1] = { t = "(", fg = PAREN_COLORS[L[b][i].c], bg = "black" }; cur = cur + 1 end
        if b <= 8 then
            tx[b] = cur
            local fg, bg = "white", "black"
            local is_op = (b % 2 == 1)
            local oi, ni = math.floor((b + 1) / 2), math.floor(b / 2)
            local hit = (is_op and f.k == "op" and f.i == oi) or ((not is_op) and f.k == "num" and f.i == ni)
            if is_op then
                if toks[b] == OP_EMPTY then fg = "yellow" else fg = hit and "#3f48cc" or "cyan" end
                if hit then bg = "light_yellow" end
            else
                fg = hit and "black" or "white"
                if hit then bg = "light_yellow" end
            end
            seg[#seg + 1] = { t = toks[b], fg = fg, bg = bg }
            cur = cur + wid(toks[b])
            if b < 8 then seg[#seg + 1] = { t = "  ", fg = "white", bg = "black" }; cur = cur + 2 end
        end
    end
    tx[9] = cur + 2
    local rv, rc = result_text()
    seg[#seg + 1] = { t = "  = ", fg = "white", bg = "black" }
    seg[#seg + 1] = { t = rv, fg = rc, bg = "black" }
    local sw = 0; for i = 1, #seg do sw = sw + wid(seg[i].t) end
    local sx = cx(string.rep(" ", sw), 1, tw)

    draw_text(1, y, string.rep(" ", tw), "white", "black")
    draw_text(1, y + 1, string.rep(" ", tw), "white", "black")
    draw_text(1, y + 2, string.rep(" ", tw), "white", "black")
    for i = 1, 9 do
        local x = sx + tx[i] - 1
        draw_text(x, y, tostring(i), "white", "black")
        draw_text(x, y + 1, "|", "white", "black")
    end
    local x = sx
    for i = 1, #seg do draw_text(x, y + 2, seg[i].t, seg[i].fg, seg[i].bg); x = x + wid(seg[i].t) end
end
local function controls()
    return tr("game.twenty_four.controls", "[Left]/[Right] Move  [1/+][2/-][3/*][4//] Edit  [Space] Clear  [Z] Add Brackets  [X] Remove Brackets  [P] Difficulty  [R] Restart  [Q]/[ESC] Exit")
end

local function min_size()
    local cw = minw(controls(), 3, 56)
    local mw = math.max(
        wid(tr("game.twenty_four.confirm_restart", "Confirm restart? [Y] Yes / [N] No")),
        wid(tr("game.twenty_four.confirm_exit", "Confirm exit? [Y] Yes / [N] No")),
        wid(tr("game.twenty_four.prompt_difficulty", "Set difficulty: 1 Classic / 2 Fixed Negative / 3 Flexible Negative"))
    )
    local tw = math.max(wid(tr("game.twenty_four.best_time", "Best Time") .. " " .. fmt(0)), wid(tr("game.twenty_four.time", "Time") .. " " .. fmt(0) .. "  " .. tr("game.twenty_four.steps", "Steps") .. " 9999"))
    return math.max(cw, mw, tw, 64) + 2, 13
end

local function draw_warn(tw, th, mw, mh)
    clear()
    local ls = {
        tr("warning.size_title", "Terminal Too Small"),
        string.format("%s: %dx%d", tr("warning.required", "Required size"), mw, mh),
        string.format("%s: %dx%d", tr("warning.current", "Current size"), tw, th),
        tr("warning.enlarge_hint", "Please enlarge terminal window to continue.")
    }
    local top = math.floor((th - #ls) / 2); if top < 1 then top = 1 end
    for i = 1, #ls do draw_text(cx(ls[i], 1, tw), top + i - 1, ls[i], "white", "black") end
end

local function size_ok()
    local tw, th = ts(); local mw, mh = min_size()
    if tw >= mw and th >= mh then
        if S.warn then clear(); S.dirty = true end
        if tw ~= S.tw or th ~= S.th then clear(); S.dirty = true end
        S.tw, S.th, S.warn = tw, th, false
        return true
    end
    local chg = (not S.warn) or S.lw ~= tw or S.lh ~= th or S.lmw ~= mw or S.lmh ~= mh
    if chg then draw_warn(tw, th, mw, mh); S.lw, S.lh, S.lmw, S.lmh = tw, th, mw, mh end
    S.warn = true
    return false
end

local function render()
    local tw, th = ts()
    local lines = wrap(controls(), math.max(20, tw - 2)); if #lines > 3 then lines = { lines[1], lines[2], lines[3] } end
    local top = math.floor((th - 10 - #lines) / 2); if top < 1 then top = 1 end
    local best = tr("game.twenty_four.best_time", "Best Time") .. "  " .. ((S.best_time > 0) and fmt(S.best_time) or tr("game.twenty_four.none", "--:--:--"))
    local stat = tr("game.twenty_four.time", "Time") .. " " .. fmt(sec()) .. "  " .. tr("game.twenty_four.steps", "Steps") .. " " .. tostring(S.steps)
    local m, mc = msg()
    for i = 0, 2 do draw_text(1, top + i, string.rep(" ", tw), "white", "black") end
    draw_text(cx(best, 1, tw), top, best, "dark_gray", "black")
    draw_text(cx(stat, 1, tw), top + 1, stat, "light_cyan", "black")
    draw_text(cx(m, 1, tw), top + 2, m, mc, "black")
    render_mid(top + 4, tw)
    local cy = top + 8
    for i = 0, 2 do draw_text(1, cy + i, string.rep(" ", tw), "white", "black") end
    local off = math.floor((3 - #lines) / 2); if off < 0 then off = 0 end
    for i = 1, #lines do draw_text(cx(lines[i], 1, tw), cy + off + i - 1, lines[i], "white", "black") end
end

local function handle_confirm(k)
    if k == "y" or k == "enter" then
        if S.confirm == "restart" then S.confirm = nil; reset_round(S.mode); return "changed" end
        if S.confirm == "exit" then return "exit" end
    end
    if k == "n" or k == "q" or k == "esc" then S.confirm = nil; S.dirty = true; return "changed" end
    return "none"
end

local function handle_input_mode(k)
    if k == "esc" or k == "q" then S.input_mode = nil; S.input_buf = ""; S.dirty = true; return "changed" end
    if k == "backspace" or k == "delete" then if #S.input_buf > 0 then S.input_buf = string.sub(S.input_buf, 1, #S.input_buf - 1); S.dirty = true end; return "changed" end
    if S.input_mode == "difficulty" then
        if k:match("^[1-3]$") and #S.input_buf < 1 then S.input_buf = k; S.dirty = true; return "changed" end
        if k == "enter" then local d = tonumber(S.input_buf); S.input_mode = nil; S.input_buf = ""; if d then reset_round(d) else S.dirty = true end; return "changed" end
        return "changed"
    end
    if S.input_mode == "paren_add" then
        if (k:match("^%d$") or k == "space") and #S.input_buf < 5 then S.input_buf = S.input_buf .. ((k == "space") and " " or k); S.dirty = true; return "changed" end
        if k == "enter" then
            local a, b = S.input_buf:match("^(%d+)%s+(%d+)$")
            S.input_mode, S.input_buf = nil, ""
            if a and b then add_pair(math.min(tonumber(a), tonumber(b)), math.max(tonumber(a), tonumber(b))); eval_expr() else S.toast, S.toast_color, S.toast_until = tr("game.twenty_four.err_input", "Invalid input."), "red", S.frame + FPS * 2 end
            S.dirty = true
            return "changed"
        end
        return "changed"
    end
    if S.input_mode == "paren_remove" then
        if k:match("^[1-4]$") and #S.input_buf < 1 then S.input_buf = k; S.dirty = true; return "changed" end
        if k == "enter" then
            local i = tonumber(S.input_buf)
            S.input_mode, S.input_buf = nil, ""
            if i and S.pairs[i] then S.pairs[i] = nil; S.steps = S.steps + 1; eval_expr() else S.toast, S.toast_color, S.toast_until = tr("game.twenty_four.err_remove_paren", "No brackets for this color index."), "red", S.frame + FPS * 2 end
            S.dirty = true
            return "changed"
        end
        return "changed"
    end
    return "none"
end

local function handle_active(k)
    local f, n = focus()
    if k == "left" then if S.cursor > 1 then S.cursor = S.cursor - 1; S.dirty = true end; return "changed" end
    if k == "right" then if S.cursor < n then S.cursor = S.cursor + 1; S.dirty = true end; return "changed" end
    if S.win then if k == "r" then reset_round(S.mode); return "changed" end; if k == "q" or k == "esc" then return "exit" end; return "none" end
    if k == "r" then S.confirm = "restart"; S.dirty = true; return "changed" end
    if k == "q" or k == "esc" then S.confirm = "exit"; S.dirty = true; return "changed" end
    if k == "p" then S.input_mode, S.input_buf = "difficulty", ""; S.dirty = true; return "changed" end
    if k == "z" then S.input_mode, S.input_buf = "paren_add", ""; S.dirty = true; return "changed" end
    if k == "x" then S.input_mode, S.input_buf = "paren_remove", ""; S.dirty = true; return "changed" end
    if k == "space" and f.k == "op" then set_op(f.i, OP_EMPTY); return "changed" end
    if k == "1" or k == "+" then if f.k == "op" then set_op(f.i, "+") elseif f.k == "num" and S.mode == M_FLEX_NEG then set_num_sign(f.i, 1) end; return "changed" end
    if k == "2" or k == "-" then if f.k == "op" then set_op(f.i, "-") elseif f.k == "num" and S.mode == M_FLEX_NEG then set_num_sign(f.i, -1) end; return "changed" end
    if k == "3" or k == "*" then if f.k == "op" then set_op(f.i, "*") end; return "changed" end
    if k == "4" or k == "/" then if f.k == "op" then set_op(f.i, "/") end; return "changed" end
    return "none"
end

local function refresh_flags()
    local e = sec(); if e ~= S.last_elapsed then S.last_elapsed = e; S.dirty = true end
    local tv = S.toast ~= nil and S.frame <= S.toast_until
    if tv ~= S.last_toast then S.last_toast = tv; S.dirty = true end
    if (not tv) and S.toast ~= nil then S.toast = nil; S.dirty = true end
end

local function init()
    clear()
    load_best()
    S.tw, S.th = ts()
    reset_round(M_CLASSIC)
    S.dirty = true
    if type(clear_input_buffer) == "function" then pcall(clear_input_buffer) end
end

local function loop()
    while true do
        local k = key(get_key(false))
        if size_ok() then
            local a = "none"
            if S.confirm then a = handle_confirm(k) elseif S.input_mode then a = handle_input_mode(k) else a = handle_active(k) end
            if a == "exit" then return end
            refresh_flags()
            if S.dirty then render(); S.dirty = false end
            S.frame = S.frame + 1
        else
            if k == "q" or k == "esc" then return end
        end
        sleep(FRAME_MS)
    end
end

init()
loop()
