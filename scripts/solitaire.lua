
GAME_META = {
    name = "Solitaire",
    description = "Arrange cards in order and suit to clear the table."
}

local FPS = 60
local FRAME_MS = 16

local MODE_TABLEAU = "tableau"
local MODE_FOUNDATION = "foundation"

local SUIT_HEART = 1
local SUIT_SPADE = 2
local SUIT_DIAMOND = 3
local SUIT_CLUB = 4

local MAX_UNDO = 100

local state = {
    mode = MODE_TABLEAU,
    tableau = {},
    stock = {},
    waste = {},
    foundation = {},
    cursor_col = 1,
    selected_col = nil,
    frame = 0,
    start_frame = 0,
    end_frame = nil,
    won = false,
    launch_mode = "new",
    last_auto_save_sec = 0,
    best_tableau_sec = 0,
    best_foundation_sec = 0,
    confirm_mode = nil,
    mode_input = false,
    msg_text = "",
    msg_color = "dark_gray",
    msg_until = 0,
    msg_persistent = false,
    dirty = true,
    last_elapsed_sec = -1,
    undo_stack = {},
    size_warning_active = false,
    last_warn_term_w = 0,
    last_warn_term_h = 0,
    last_warn_min_w = 0,
    last_warn_min_h = 0,
    last_term_w = 0,
    last_term_h = 0,
    top_dirty = false,
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

local function wrap_words(text, max_width)
    if max_width <= 1 then return { text } end
    local lines, current, had_token = {}, "", false
    for token in string.gmatch(text, "%S+") do
        had_token = true
        if current == "" then
            current = token
        else
            local candidate = current .. " " .. token
            if key_width(candidate) <= max_width then current = candidate else lines[#lines + 1] = current; current = token end
        end
    end
    if not had_token then return { "" } end
    if current ~= "" then lines[#lines + 1] = current end
    return lines
end

local function min_width_for_lines(text, max_lines, hard_min)
    local full = key_width(text)
    local width = hard_min
    while width <= full do
        if #wrap_words(text, width) <= max_lines then return width end
        width = width + 1
    end
    return full
end

local function terminal_size()
    local w, h = 120, 40
    if type(get_terminal_size) == "function" then
        local tw, th = get_terminal_size()
        if type(tw) == "number" and type(th) == "number" then w, h = tw, th end
    end
    return w, h
end

local function centered_x(text, x, w)
    local px = x + math.floor((w - key_width(text)) / 2)
    if px < x then px = x end
    return px
end

local function normalize_key(key)
    if key == nil then return "" end
    if type(key) == "string" then return string.lower(key) end
    return tostring(key):lower()
end

local function flush_input_buffer()
    if type(clear_input_buffer) == "function" then pcall(clear_input_buffer) end
end

local function clamp(v, lo, hi)
    if v < lo then return lo end
    if v > hi then return hi end
    return v
end

local function elapsed_seconds()
    local ending = state.end_frame or state.frame
    return math.max(0, math.floor((ending - state.start_frame) / FPS))
end

local function format_duration(sec)
    local h = math.floor(sec / 3600)
    local m = math.floor((sec % 3600) / 60)
    local s = sec % 60
    return string.format("%02d:%02d:%02d", h, m, s)
end

local function rand_int(n)
    if n <= 0 or type(random) ~= "function" then return 0 end
    return random(n)
end

local function rand_range(lo, hi)
    if hi <= lo then return lo end
    return lo + rand_int(hi - lo + 1)
end

local function show_message(text, color, dur_sec, persistent)
    state.msg_text = text or ""
    state.msg_color = color or "dark_gray"
    state.msg_persistent = persistent == true
    if dur_sec ~= nil and dur_sec > 0 then
        state.msg_until = state.frame + math.floor(dur_sec * FPS + 0.5)
    else
        state.msg_until = 0
    end
    state.dirty = true
end

local function clear_message()
    if state.msg_text ~= "" then
        state.msg_text = ""
        state.msg_color = "dark_gray"
        state.msg_until = 0
        state.msg_persistent = false
        state.dirty = true
    end
end

local function update_message_timer()
    if state.msg_persistent then return end
    if state.msg_until > 0 and state.frame >= state.msg_until then clear_message() end
end

local function rank_text(rank)
    if rank == 1 then return "A" end
    if rank == 11 then return "J" end
    if rank == 12 then return "Q" end
    if rank == 13 then return "K" end
    return tostring(rank)
end

local function is_red(card)
    return card.suit == SUIT_HEART or card.suit == SUIT_DIAMOND
end

local function card_color(card)
    if state.mode == MODE_FOUNDATION then
        if card.suit == SUIT_HEART then return "red" end
        if card.suit == SUIT_SPADE then return "white" end
        if card.suit == SUIT_DIAMOND then return "rgb(255,165,0)" end
        return "cyan"
    end
    if is_red(card) then return "red" end
    return "white"
end

local function clone_card(card)
    return { rank = card.rank, suit = card.suit, face_up = card.face_up == true }
end

local function deep_copy_cards(cards)
    local out = {}
    for i = 1, #cards do out[i] = clone_card(cards[i]) end
    return out
end

local function deep_copy_tableau(tableau)
    local out = {}
    for i = 1, 7 do out[i] = deep_copy_cards(tableau[i] or {}) end
    return out
end

local function deep_copy_foundation(foundation)
    local out = {}
    for i = 1, 4 do out[i] = deep_copy_cards(foundation[i] or {}) end
    return out
end

local function snapshot_state()
    return {
        mode = state.mode,
        tableau = deep_copy_tableau(state.tableau),
        stock = deep_copy_cards(state.stock),
        waste = deep_copy_cards(state.waste),
        foundation = deep_copy_foundation(state.foundation),
        cursor_col = state.cursor_col,
        selected_col = state.selected_col,
        frame = state.frame,
        start_frame = state.start_frame,
        end_frame = state.end_frame,
        won = state.won,
        last_auto_save_sec = state.last_auto_save_sec,
    }
end

local function restore_snapshot(snap, clear_undo)
    state.mode = snap.mode == MODE_FOUNDATION and MODE_FOUNDATION or MODE_TABLEAU
    state.tableau = deep_copy_tableau(snap.tableau or {})
    state.stock = deep_copy_cards(snap.stock or {})
    state.waste = deep_copy_cards(snap.waste or {})
    state.foundation = deep_copy_foundation(snap.foundation or {})
    state.cursor_col = clamp(math.floor(tonumber(snap.cursor_col) or 1), 1, 7)
    local sel = tonumber(snap.selected_col)
    if sel ~= nil then state.selected_col = clamp(math.floor(sel), 1, 7) else state.selected_col = nil end
    state.frame = math.max(0, math.floor(tonumber(snap.frame) or state.frame))
    state.start_frame = math.max(0, math.floor(tonumber(snap.start_frame) or state.start_frame))
    state.end_frame = snap.end_frame and math.max(0, math.floor(tonumber(snap.end_frame) or state.frame)) or nil
    state.won = snap.won == true
    state.last_auto_save_sec = math.max(0, math.floor(tonumber(snap.last_auto_save_sec) or 0))
    state.confirm_mode = nil
    state.mode_input = false
    if clear_undo == nil or clear_undo then
        state.undo_stack = {}
    end
    clear_message()
    state.dirty = true
end

local function push_undo()
    state.undo_stack[#state.undo_stack + 1] = snapshot_state()
    while #state.undo_stack > MAX_UNDO do table.remove(state.undo_stack, 1) end
end

local function pop_undo()
    if #state.undo_stack == 0 then
        show_message(tr("game.solitaire.undo_empty", "No more undo steps."), "dark_gray", 2, false)
        return false
    end
    local snap = state.undo_stack[#state.undo_stack]
    table.remove(state.undo_stack)
    -- Keep remaining undo history when reverting one step.
    restore_snapshot(snap, false)
    show_message(tr("game.solitaire.undo_done", "Undo successful."), "yellow", 2, false)
    return true
end
local fresh_empty_piles
local can_place_on_tableau

local function build_mode_ordered_sequences(mode)
    local seqs = { {}, {}, {}, {} }
    if mode == MODE_FOUNDATION then
        for suit = SUIT_HEART, SUIT_CLUB do
            for rank = 1, 13 do
                seqs[suit][#seqs[suit] + 1] = { rank = rank, suit = suit, face_up = true }
            end
        end
        return seqs
    end

    -- Tableau mode: 4 descending K..A sequences with strict red/black alternation.
    for rank = 13, 1, -1 do
        local idx = 14 - rank
        if idx % 2 == 1 then
            seqs[1][#seqs[1] + 1] = { rank = rank, suit = SUIT_HEART, face_up = true }   -- red
            seqs[2][#seqs[2] + 1] = { rank = rank, suit = SUIT_SPADE, face_up = true }   -- black
            seqs[3][#seqs[3] + 1] = { rank = rank, suit = SUIT_DIAMOND, face_up = true } -- red
            seqs[4][#seqs[4] + 1] = { rank = rank, suit = SUIT_CLUB, face_up = true }    -- black
        else
            seqs[1][#seqs[1] + 1] = { rank = rank, suit = SUIT_SPADE, face_up = true }   -- black
            seqs[2][#seqs[2] + 1] = { rank = rank, suit = SUIT_DIAMOND, face_up = true } -- red
            seqs[3][#seqs[3] + 1] = { rank = rank, suit = SUIT_CLUB, face_up = true }    -- black
            seqs[4][#seqs[4] + 1] = { rank = rank, suit = SUIT_HEART, face_up = true }   -- red
        end
    end
    return seqs
end

local function random_four_columns()
    local cols = { 1, 2, 3, 4, 5, 6, 7 }
    for i = #cols, 2, -1 do
        local j = rand_range(1, i)
        cols[i], cols[j] = cols[j], cols[i]
    end
    return { cols[1], cols[2], cols[3], cols[4] }
end

local function collect_deck_from_tableau()
    local deck = {}
    for col = 1, 7 do
        local pile = state.tableau[col]
        for i = 1, #pile do
            local card = clone_card(pile[i])
            card.face_up = false
            deck[#deck + 1] = card
        end
    end
    return deck
end

local function generate_reverse_solved_layout(mode)
    fresh_empty_piles()
    state.mode = mode == MODE_FOUNDATION and MODE_FOUNDATION or MODE_TABLEAU

    local target_steps = rand_range(300, 400)
    local seqs = build_mode_ordered_sequences(state.mode)
    local seed_cols = random_four_columns()

    for i = 1, 4 do
        local col = seed_cols[i]
        for j = 1, #seqs[i] do
            state.tableau[col][#state.tableau[col] + 1] = clone_card(seqs[i][j])
        end
    end

    for _ = 1, target_steps do
        local src_candidates = {}
        for c = 1, 7 do
            if #state.tableau[c] > 0 then src_candidates[#src_candidates + 1] = c end
        end
        if #src_candidates == 0 then break end

        local src_col = src_candidates[rand_range(1, #src_candidates)]
        local src = state.tableau[src_col]
        local start_idx = rand_range(1, #src)
        local moving_first = src[start_idx]

        local dst_candidates = {}
        for dst_col = 1, 7 do
            if dst_col ~= src_col and can_place_on_tableau(moving_first, dst_col) then
                dst_candidates[#dst_candidates + 1] = dst_col
            end
        end

        if #dst_candidates > 0 then
            local dst_col = dst_candidates[rand_range(1, #dst_candidates)]
            local dst = state.tableau[dst_col]
            for i = start_idx, #src do dst[#dst + 1] = src[i] end
            for i = #src, start_idx, -1 do table.remove(src, i) end
        end
    end

    local deck = collect_deck_from_tableau()
    if #deck ~= 52 then
        deck = {}
        for suit = SUIT_HEART, SUIT_CLUB do
            for rank = 1, 13 do
                deck[#deck + 1] = { rank = rank, suit = suit, face_up = false }
            end
        end
    end

    fresh_empty_piles()
    local p = 1
    for col = 1, 7 do
        for row = 1, col do
            local card = clone_card(deck[p])
            p = p + 1
            card.face_up = (row == col)
            state.tableau[col][#state.tableau[col] + 1] = card
        end
    end

    while p <= #deck do
        local card = clone_card(deck[p])
        p = p + 1
        card.face_up = false
        state.stock[#state.stock + 1] = card
    end
end

fresh_empty_piles = function()
    state.tableau = {}
    for i = 1, 7 do state.tableau[i] = {} end
    state.stock = {}
    state.waste = {}
    state.foundation = {}
    for i = 1, 4 do state.foundation[i] = {} end
end

local function update_best_if_needed()
    if not state.won then return end
    local elapsed = elapsed_seconds()
    if state.mode == MODE_TABLEAU then
        if state.best_tableau_sec <= 0 or elapsed < state.best_tableau_sec then state.best_tableau_sec = elapsed end
    else
        if state.best_foundation_sec <= 0 or elapsed < state.best_foundation_sec then state.best_foundation_sec = elapsed end
    end
    if type(save_data) == "function" then
        pcall(save_data, "solitaire_best", { tableau = state.best_tableau_sec, foundation = state.best_foundation_sec })
    end
end

local function load_best_record()
    state.best_tableau_sec = 0
    state.best_foundation_sec = 0
    if type(load_data) ~= "function" then return end
    local ok, data = pcall(load_data, "solitaire_best")
    if not ok or type(data) ~= "table" then return end
    state.best_tableau_sec = math.max(0, math.floor(tonumber(data.tableau) or 0))
    state.best_foundation_sec = math.max(0, math.floor(tonumber(data.foundation) or 0))
end

local function deal_new_game(mode)
    state.mode = mode == MODE_FOUNDATION and MODE_FOUNDATION or MODE_TABLEAU
    generate_reverse_solved_layout(state.mode)

    state.cursor_col = 1
    state.selected_col = nil
    state.confirm_mode = nil
    state.mode_input = false
    state.start_frame = state.frame
    state.end_frame = nil
    state.won = false
    state.last_auto_save_sec = 0
    state.undo_stack = {}
    clear_message()
    state.dirty = true
end

local function total_foundation_cards()
    local total = 0
    for i = 1, 4 do total = total + #state.foundation[i] end
    return total
end

local function check_win()
    if state.won then return end
    if total_foundation_cards() >= 52 then
        state.won = true
        state.end_frame = state.frame
        update_best_if_needed()
        if type(update_game_stats) == "function" then pcall(update_game_stats, "solitaire", 0, elapsed_seconds()) end
        show_message(
            tr("game.solitaire.win_banner", "All cards have been collected!") .. " " .. tr("game.solitaire.result_controls", "[R] Restart  [Q]/[ESC] Exit"),
            "green",
            0,
            true
        )
    end
end

local function color_group(card)
    return is_red(card) and 1 or 0
end

local function top_card(cards)
    if #cards <= 0 then return nil end
    return cards[#cards]
end

local function foundation_slot_color(slot)
    if slot == 1 or slot == 3 then return 1 end
    return 0
end

local function can_place_foundation(slot, card)
    local pile = state.foundation[slot]
    local top = top_card(pile)
    if top == nil then return card.rank == 1 end

    if state.mode == MODE_FOUNDATION then
        if top.suit ~= card.suit then return false end
    else
        if foundation_slot_color(slot) ~= color_group(card) then return false end
    end

    return card.rank == top.rank + 1
end

local function choose_foundation_slot(card)
    if state.mode == MODE_FOUNDATION then
        local slot = card.suit
        if can_place_foundation(slot, card) then return slot end
        return nil
    end

    local candidates = color_group(card) == 1 and { 1, 3 } or { 2, 4 }
    for i = 1, #candidates do
        if can_place_foundation(candidates[i], card) then return candidates[i] end
    end
    return nil
end

can_place_on_tableau = function(card, dest_col)
    local dest = state.tableau[dest_col]
    local top = top_card(dest)
    if top == nil then return true end
    if not top.face_up then return false end
    if state.mode == MODE_FOUNDATION then
        if top.suit ~= card.suit then return false end
        return card.rank == top.rank + 1
    end
    if top.rank ~= card.rank + 1 then return false end
    return color_group(top) ~= color_group(card)
end

local function first_face_up_index(col)
    local cards = state.tableau[col]
    for i = 1, #cards do
        if cards[i].face_up then return i end
    end
    return nil
end

local function column_satisfies_mode_sequence(cards)
    if #cards ~= 13 then return false end
    for i = 1, 13 do
        if not cards[i].face_up then return false end
    end

    if state.mode == MODE_FOUNDATION then
        local suit = cards[1].suit
        for i = 1, 13 do
            if cards[i].suit ~= suit then return false end
            if cards[i].rank ~= i then return false end
        end
        return true
    end

    for i = 1, 13 do
        if cards[i].rank ~= (14 - i) then return false end
        if i > 1 and color_group(cards[i]) == color_group(cards[i - 1]) then return false end
    end
    return true
end

local function find_collect_slot_for_column(cards)
    if state.mode == MODE_FOUNDATION then
        local slot = cards[1].suit
        if #state.foundation[slot] == 0 then return slot end
        return nil
    end

    local group = color_group(cards[1])
    local candidates = (group == 1) and { 1, 3 } or { 2, 4 }
    for i = 1, #candidates do
        local slot = candidates[i]
        if #state.foundation[slot] == 0 then return slot end
    end
    return nil
end

local function try_collect_column(col)
    if col < 1 or col > 7 then return false end
    local pile = state.tableau[col]
    if not column_satisfies_mode_sequence(pile) then return false end

    local slot = find_collect_slot_for_column(pile)
    if slot == nil then return false end

    state.foundation[slot] = deep_copy_cards(pile)
    state.tableau[col] = {}
    if state.selected_col == col then state.selected_col = nil end
    state.dirty = true
    check_win()
    return true
end

local function try_collect_after_move(col_a, col_b)
    local changed = false
    if try_collect_column(col_a) then changed = true end
    if col_b ~= col_a and try_collect_column(col_b) then changed = true end
    return changed
end

local function reveal_new_top(col)
    local cards = state.tableau[col]
    if #cards <= 0 then return end
    local t = cards[#cards]
    if not t.face_up then t.face_up = true; state.dirty = true end
end

local function stack_sequence_valid(cards, start_idx)
    if start_idx < 1 or start_idx > #cards then return false end
    for i = start_idx, #cards - 1 do
        local a = cards[i]
        local b = cards[i + 1]
        if not a.face_up or not b.face_up then return false end
        if state.mode == MODE_FOUNDATION then
            if a.suit ~= b.suit then return false end
            if b.rank ~= a.rank + 1 then return false end
        else
            if b.rank ~= a.rank - 1 then return false end
            if color_group(a) == color_group(b) then return false end
        end
    end
    return true
end

local function move_tableau_stack(src_col, dst_col)
    if src_col < 1 or src_col > 7 or dst_col < 1 or dst_col > 7 then return false end
    if src_col == dst_col then return false end

    local src = state.tableau[src_col]
    if #src <= 0 then return false end
    local dst = state.tableau[dst_col]

    local start_idx = first_face_up_index(src_col)
    if start_idx == nil then return false end
    if not stack_sequence_valid(src, start_idx) then return false end
    local moving_first = src[start_idx]
    if not can_place_on_tableau(moving_first, dst_col) then return false end

    push_undo()
    for i = start_idx, #src do dst[#dst + 1] = src[i] end
    for i = #src, start_idx, -1 do table.remove(src, i) end
    reveal_new_top(src_col)
    state.selected_col = nil
    try_collect_after_move(src_col, dst_col)
    state.dirty = true
    return true
end

local function move_current_to_foundation(col)
    if col < 1 or col > 7 then return false end
    local pile = state.tableau[col]
    local card = top_card(pile)
    if card == nil or not card.face_up then return false end

    local slot = choose_foundation_slot(card)
    if slot == nil then return false end

    push_undo()
    table.remove(pile, #pile)
    state.foundation[slot][#state.foundation[slot] + 1] = card
    reveal_new_top(col)
    state.selected_col = nil
    state.dirty = true
    check_win()
    return true
end

local function move_waste_to_foundation()
    local card = top_card(state.waste)
    if card == nil then return false end
    local slot = choose_foundation_slot(card)
    if slot == nil then return false end

    push_undo()
    table.remove(state.waste, #state.waste)
    state.foundation[slot][#state.foundation[slot] + 1] = card
    state.dirty = true
    check_win()
    return true
end

local function move_waste_to_column(col)
    if col < 1 or col > 7 then return false end
    local card = top_card(state.waste)
    if card == nil then return false end
    if not can_place_on_tableau(card, col) then return false end

    push_undo()
    table.remove(state.waste, #state.waste)
    state.tableau[col][#state.tableau[col] + 1] = card
    try_collect_column(col)
    state.dirty = true
    return true
end

local function draw_from_stock()
    if #state.stock == 0 then
        if #state.waste == 0 then
            show_message(tr("game.solitaire.stock_empty", "No cards in stock and waste."), "dark_gray", 2, false)
            return false
        end

        push_undo()
        for i = #state.waste, 1, -1 do
            local card = state.waste[i]
            card.face_up = false
            state.stock[#state.stock + 1] = card
        end
        state.waste = {}
        state.dirty = true
        show_message(tr("game.solitaire.recycle_done", "Waste recycled to stock."), "yellow", 2, false)
        return true
    end

    push_undo()
    local card = state.stock[#state.stock]
    table.remove(state.stock, #state.stock)
    card.face_up = true
    state.waste[#state.waste + 1] = card
    state.dirty = true
    return true
end
local function mode_label(mode)
    if mode == MODE_FOUNDATION then return tr("game.solitaire.mode.foundation", "Foundation") end
    return tr("game.solitaire.mode.tableau", "Tableau")
end

local function suit_color_for_foundation_slot(slot)
    if state.mode == MODE_FOUNDATION then
        if slot == 1 then return "red" end
        if slot == 2 then return "white" end
        if slot == 3 then return "rgb(255,165,0)" end
        return "cyan"
    end
    if slot == 1 or slot == 3 then return "red" end
    return "white"
end

local function foundation_label(slot)
    local pile = state.foundation[slot]
    if #pile <= 0 then return "[ ]" end
    return "[" .. rank_text(pile[#pile].rank) .. "]"
end

local function card_two_chars(card)
    local rt = rank_text(card.rank)
    if rt == "10" then return "10" end
    return " " .. rt
end

local function fill_rect(x, y, w, h, bg)
    if w <= 0 or h <= 0 then return end
    local line = string.rep(" ", w)
    for i = 0, h - 1 do
        draw_text(x, y + i, line, "white", bg or "black")
    end
end

local function board_geometry()
    local term_w, term_h = terminal_size()
    local board_w, board_h = 44, 22
    local top_h, controls_h = 5, 3
    local total_h = top_h + board_h + controls_h

    local x = math.floor((term_w - board_w) / 2) + 1
    if x < 1 then x = 1 end
    local y = math.floor((term_h - total_h) / 2) + 1
    if y < 1 then y = 1 end

    return {
        term_w = term_w,
        term_h = term_h,
        top_y = y,
        top_h = top_h,
        board_x = x,
        board_y = y + top_h,
        board_w = board_w,
        board_h = board_h,
        controls_y = y + top_h + board_h,
    }
end

local function minimum_required_size()
    local mode_w = math.max(key_width(mode_label(MODE_TABLEAU)), key_width(mode_label(MODE_FOUNDATION)))
    local top_line_w = key_width(tr("game.solitaire.collected", "Collected")) + 20
        + key_width(tr("game.solitaire.stock", "Stock")) + 6
        + key_width(tr("game.solitaire.waste", "Waste")) + 14

    local status_w = key_width(tr("game.solitaire.time", "Time") .. " 00:00:00")
        + 2 + key_width(tr("game.solitaire.mode", "Mode") .. " " .. mode_label(MODE_FOUNDATION))

    local best_w = key_width(tr("game.solitaire.best.tableau", "Tableau Best") .. " 00:00:00")
        + 2 + key_width(tr("game.solitaire.best.foundation", "Foundation Best") .. " 00:00:00")

    local controls = tr(
        "game.solitaire.controls",
        "[←]/[→] Move  [Space] Select Column  [Enter] Confirm Move  [Z] Cancel Select  [X] Draw  [C] Waste->Column  [P] Mode  [A] Undo  [S] Save  [R] Restart  [Q]/[ESC] Exit"
    )
    local controls_w = min_width_for_lines(controls, 3, 36)

    local msg_w = math.max(
        key_width(tr("game.solitaire.confirm_restart", "Confirm restart? [Y] Yes / [N] No")),
        key_width(tr("game.solitaire.confirm_exit", "Confirm exit? [Y] Yes / [N] No")),
        key_width(tr("game.solitaire.mode_prompt", "Switch mode: [T] Tableau / [F] Foundation")),
        key_width(tr("game.solitaire.win_banner", "All cards have been collected!")),
        mode_w
    )

    local min_w = math.max(80, top_line_w + 2, status_w + 2, best_w + 2, controls_w + 2, msg_w + 2)
    local min_h = 34
    return min_w, min_h
end

local function draw_terminal_size_warning(term_w, term_h, min_w, min_h)
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
        local resized = term_w ~= state.last_term_w or term_h ~= state.last_term_h
        state.last_term_w = term_w
        state.last_term_h = term_h
        if state.size_warning_active or resized then clear(); state.dirty = true end
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

local function draw_top_panel(g)
    for i = 0, g.top_h - 1 do
        draw_text(1, g.top_y + i, string.rep(" ", g.term_w), "white", "black")
    end

    local best_tableau = state.best_tableau_sec > 0 and format_duration(state.best_tableau_sec) or "--:--:--"
    local best_found = state.best_foundation_sec > 0 and format_duration(state.best_foundation_sec) or "--:--:--"

    local best_line = tr("game.solitaire.best.tableau", "Tableau Best") .. " " .. best_tableau
        .. "  " .. tr("game.solitaire.best.foundation", "Foundation Best") .. " " .. best_found

    local status_line = tr("game.solitaire.time", "Time") .. " " .. format_duration(elapsed_seconds())
        .. "  " .. tr("game.solitaire.mode", "Mode") .. " " .. mode_label(state.mode)

    local f1, f2, f3, f4 = foundation_label(1), foundation_label(2), foundation_label(3), foundation_label(4)

    local top_line = tr("game.solitaire.collected", "Collected") .. ": "
        .. f1 .. " " .. f2 .. " " .. f3 .. " " .. f4
        .. "    " .. tr("game.solitaire.stock", "Stock") .. " [##]"
        .. "   " .. tr("game.solitaire.waste", "Waste") .. " ["

    local w1, w2, w3 = "  ", "  ", "  "
    if #state.waste >= 1 then w1 = card_two_chars(state.waste[#state.waste]) end
    if #state.waste >= 2 then w2 = card_two_chars(state.waste[#state.waste - 1]) end
    if #state.waste >= 3 then w3 = card_two_chars(state.waste[#state.waste - 2]) end
    top_line = top_line .. w3 .. " " .. w2 .. " " .. w1 .. "]"

    draw_text(centered_x(best_line, 1, g.term_w), g.top_y, best_line, "dark_gray", "black")
    draw_text(centered_x(status_line, 1, g.term_w), g.top_y + 1, status_line, "light_cyan", "black")

    local top_x = centered_x(top_line, 1, g.term_w)
    draw_text(top_x, g.top_y + 2, top_line, "white", "black")

    local base_x = top_x + key_width(tr("game.solitaire.collected", "Collected")) + 2
    draw_text(base_x, g.top_y + 2, f1, suit_color_for_foundation_slot(1), "black")
    base_x = base_x + key_width(f1) + 1
    draw_text(base_x, g.top_y + 2, f2, suit_color_for_foundation_slot(2), "black")
    base_x = base_x + key_width(f2) + 1
    draw_text(base_x, g.top_y + 2, f3, suit_color_for_foundation_slot(3), "black")
    base_x = base_x + key_width(f3) + 1
    draw_text(base_x, g.top_y + 2, f4, suit_color_for_foundation_slot(4), "black")

    local waste_block = "[" .. w3 .. " " .. w2 .. " " .. w1 .. "]"
    local waste_start_x = top_x + key_width(top_line) - key_width(waste_block)
    if #state.waste >= 3 then
        draw_text(waste_start_x + 1, g.top_y + 2, w3, card_color(state.waste[#state.waste - 2]), "black")
    end
    if #state.waste >= 2 then
        draw_text(waste_start_x + 4, g.top_y + 2, w2, card_color(state.waste[#state.waste - 1]), "black")
    end
    if #state.waste >= 1 then
        draw_text(waste_start_x + 7, g.top_y + 2, w1, card_color(state.waste[#state.waste]), "black")
    end

    local pointer_x = top_x + key_width(top_line) - key_width(waste_block) + key_width(waste_block) - 3
    draw_text(pointer_x, g.top_y + 3, "^^", "yellow", "black")
end

local function draw_top_status_line(g)
    local status_line = tr("game.solitaire.time", "Time") .. " " .. format_duration(elapsed_seconds())
        .. "  " .. tr("game.solitaire.mode", "Mode") .. " " .. mode_label(state.mode)
    draw_text(1, g.top_y + 1, string.rep(" ", g.term_w), "white", "black")
    draw_text(centered_x(status_line, 1, g.term_w), g.top_y + 1, status_line, "light_cyan", "black")
end

local function draw_board_panel(g)
    fill_rect(g.board_x, g.board_y, g.board_w, g.board_h, "black")
    local col_x0 = g.board_x + 8
    local col_gap = 4
    local header_y = g.board_y

    for c = 1, 7 do
        draw_text(col_x0 + (c - 1) * col_gap, header_y, "C" .. tostring(c), "dark_gray", "black")
    end

    local max_rows = 19
    for c = 1, 7 do
        if #state.tableau[c] + 1 > max_rows then max_rows = #state.tableau[c] + 1 end
    end
    if max_rows > 19 then max_rows = 19 end

    for r = 1, max_rows do
        local ry = header_y + r
        draw_text(g.board_x + 1, ry, string.format("%-3s", "R" .. tostring(r)), "dark_gray", "black")
        for c = 1, 7 do
            local cx = col_x0 + (c - 1) * col_gap
            local pile = state.tableau[c]
            if r <= #pile then
                local card = pile[r]
                if card.face_up then
                    draw_text(cx, ry, card_two_chars(card), card_color(card), "black")
                else
                    draw_text(cx, ry, "##", "rgb(160,160,160)", "black")
                end
            else
                draw_text(cx, ry, "  ", "white", "black")
            end
        end
    end

    local function draw_column_frame(col, color)
        local pile = state.tableau[col]
        local cx = col_x0 + (col - 1) * col_gap
        if #pile == 0 then
            local ey = header_y + 1
            draw_text(cx - 1, ey, "┌──┐", color, "black")
            draw_text(cx - 1, ey + 1, "└──┘", color, "black")
            return
        end
        local start_idx = first_face_up_index(col)
        if start_idx == nil then return end

        local top_y = header_y + start_idx
        local bottom_y = header_y + #pile
        draw_text(cx - 1, top_y, "┌", color, "black")
        draw_text(cx + 2, top_y, "┐", color, "black")
        for yy = top_y + 1, bottom_y do
            draw_text(cx - 1, yy, "│", color, "black")
            draw_text(cx + 2, yy, "│", color, "black")
        end
        if bottom_y + 1 <= g.board_y + g.board_h - 1 then
            draw_text(cx - 1, bottom_y + 1, "└──┘", color, "black")
        end
    end

    if state.selected_col ~= nil then draw_column_frame(state.selected_col, "green") end
    draw_column_frame(state.cursor_col, "yellow")
end
local function current_message_line()
    if state.mode_input then
        return tr("game.solitaire.mode_prompt", "Switch mode: [T] Tableau / [F] Foundation"), "yellow"
    end
    if state.confirm_mode == "restart" then
        return tr("game.solitaire.confirm_restart", "Confirm restart? [Y] Yes / [N] No"), "yellow"
    end
    if state.confirm_mode == "exit" then
        return tr("game.solitaire.confirm_exit", "Confirm exit? [Y] Yes / [N] No"), "yellow"
    end
    if state.msg_text ~= "" then
        return state.msg_text, state.msg_color
    end
    return "", "dark_gray"
end

local function draw_bottom_panel(g)
    local msg, msg_color = current_message_line()
    draw_text(1, g.controls_y, string.rep(" ", g.term_w), "white", "black")
    if msg ~= "" then
        draw_text(centered_x(msg, 1, g.term_w), g.controls_y, msg, msg_color, "black")
    end

    local controls = tr(
        "game.solitaire.controls",
        "[←]/[→] Move  [Space] Select Column  [Enter] Confirm Move  [Z] Cancel Select  [X] Draw  [C] Waste->Column  [P] Mode  [A] Undo  [S] Save  [R] Restart  [Q]/[ESC] Exit"
    )
    local lines = wrap_words(controls, math.max(12, g.term_w - 2))
    if #lines > 2 then lines = { lines[1], lines[2] } end

    draw_text(1, g.controls_y + 1, string.rep(" ", g.term_w), "white", "black")
    draw_text(1, g.controls_y + 2, string.rep(" ", g.term_w), "white", "black")

    local off = (#lines == 1) and 1 or 0
    for i = 1, #lines do
        draw_text(centered_x(lines[i], 1, g.term_w), g.controls_y + off + i, lines[i], "white", "black")
    end
end

local function render()
    local g = board_geometry()
    draw_top_panel(g)
    draw_board_panel(g)
    draw_bottom_panel(g)
end

local function make_save_snapshot()
    return {
        mode = state.mode,
        tableau = deep_copy_tableau(state.tableau),
        stock = deep_copy_cards(state.stock),
        waste = deep_copy_cards(state.waste),
        foundation = deep_copy_foundation(state.foundation),
        cursor_col = state.cursor_col,
        selected_col = state.selected_col,
        frame = state.frame,
        start_frame = state.start_frame,
        won = state.won,
        end_frame = state.end_frame,
        last_auto_save_sec = state.last_auto_save_sec,
    }
end

local function save_game_state(show_toast)
    local snapshot = make_save_snapshot()
    local ok = false

    if type(save_game_slot) == "function" then
        ok = pcall(save_game_slot, "solitaire", snapshot)
    elseif type(save_data) == "function" then
        ok = pcall(save_data, "solitaire", snapshot)
    end

    if show_toast then
        if ok then
            show_message(tr("game.solitaire.save_success", "Save successful!"), "green", 2, false)
        else
            show_message(tr("game.solitaire.save_unavailable", "Save unavailable."), "red", 2, false)
        end
    end

    return ok
end

local function load_continue_state()
    local data = nil
    if type(load_game_slot) == "function" then
        local ok, ret = pcall(load_game_slot, "solitaire")
        if ok then data = ret end
    elseif type(load_data) == "function" then
        local ok, ret = pcall(load_data, "solitaire")
        if ok then data = ret end
    end

    if type(data) ~= "table" then return false end
    if type(data.tableau) ~= "table" or type(data.stock) ~= "table" or type(data.waste) ~= "table" or type(data.foundation) ~= "table" then
        return false
    end

    restore_snapshot(data)
    state.undo_stack = {}
    return true
end

local function read_launch_mode()
    if type(get_launch_mode) ~= "function" then return "new" end
    local ok, mode = pcall(get_launch_mode)
    if not ok or type(mode) ~= "string" then return "new" end
    mode = string.lower(mode)
    if mode == "continue" then return "continue" end
    return "new"
end

local function on_restart_requested()
    state.confirm_mode = "restart"
    state.mode_input = false
    show_message("", "dark_gray", 0, false)
    state.dirty = true
    flush_input_buffer()
end

local function on_exit_requested()
    state.confirm_mode = "exit"
    state.mode_input = false
    show_message("", "dark_gray", 0, false)
    state.dirty = true
    flush_input_buffer()
end

local function handle_confirm_key(key)
    if key == "y" or key == "enter" then
        if state.confirm_mode == "restart" then
            deal_new_game(state.mode)
            return "changed"
        end
        if state.confirm_mode == "exit" then
            return "exit"
        end
    elseif key == "n" or key == "q" or key == "esc" then
        state.confirm_mode = nil
        state.dirty = true
        return "changed"
    end
    return "none"
end

local function handle_mode_input_key(key)
    if key == "esc" or key == "q" then
        state.mode_input = false
        state.dirty = true
        return "changed"
    end
    if key == "t" then
        state.mode_input = false
        deal_new_game(MODE_TABLEAU)
        return "changed"
    end
    if key == "f" then
        state.mode_input = false
        deal_new_game(MODE_FOUNDATION)
        return "changed"
    end
    return "none"
end

local function handle_gameplay_key(key)
    if key == nil or key == "" then return "none" end

    if state.confirm_mode ~= nil then return handle_confirm_key(key) end
    if state.mode_input then return handle_mode_input_key(key) end

    if state.won then
        if key == "r" then deal_new_game(state.mode); return "changed" end
        if key == "q" or key == "esc" then return "exit" end
        if key == "s" then save_game_state(true); return "changed" end
        return "none"
    end

    if key == "r" then on_restart_requested(); return "changed" end
    if key == "q" or key == "esc" then on_exit_requested(); return "changed" end
    if key == "s" then save_game_state(true); return "changed" end

    if key == "left" then state.cursor_col = clamp(state.cursor_col - 1, 1, 7); state.dirty = true; return "changed" end
    if key == "right" then state.cursor_col = clamp(state.cursor_col + 1, 1, 7); state.dirty = true; return "changed" end

    if key == "space" then
        if #state.tableau[state.cursor_col] > 0 and first_face_up_index(state.cursor_col) ~= nil then
            state.selected_col = state.cursor_col
            state.dirty = true
            return "changed"
        end
        show_message(tr("game.solitaire.select_empty", "Current column cannot be selected."), "dark_gray", 2, false)
        return "changed"
    end

    if key == "z" then state.selected_col = nil; state.dirty = true; return "changed" end

    if key == "enter" then
        if state.selected_col ~= nil then
            local src = state.selected_col
            local dst = state.cursor_col
            if src ~= dst and move_tableau_stack(src, dst) then check_win(); return "changed" end
            show_message(tr("game.solitaire.move_invalid", "Invalid move."), "red", 2, false)
            return "changed"
        else
            show_message(tr("game.solitaire.move_invalid", "Invalid move."), "red", 2, false)
            return "changed"
        end
    end

    if key == "x" then draw_from_stock(); check_win(); return "changed" end

    if key == "c" then
        if move_waste_to_column(state.cursor_col) then check_win(); return "changed" end
        show_message(tr("game.solitaire.waste_invalid", "Waste card cannot be placed there."), "red", 2, false)
        return "changed"
    end

    if key == "p" then
        state.mode_input = true
        state.confirm_mode = nil
        state.dirty = true
        flush_input_buffer()
        return "changed"
    end

    if key == "a" then
        pop_undo()
        return "changed"
    end

    return "none"
end
local function auto_save_if_needed()
    if state.won then return end
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
        if state.dirty then
            state.top_dirty = false
        else
            state.top_dirty = true
        end
    end
    update_message_timer()
end

local function init_game()
    clear()
    local w, h = terminal_size()
    state.last_term_w = w
    state.last_term_h = h

    load_best_record()

    state.launch_mode = read_launch_mode()
    if state.launch_mode == "continue" then
        if not load_continue_state() then
            deal_new_game(MODE_TABLEAU)
        end
    else
        deal_new_game(MODE_TABLEAU)
    end

    flush_input_buffer()
end

local function game_loop()
    while true do
        local key = normalize_key(get_key(false))

        if ensure_terminal_size_ok() then
            local action = handle_gameplay_key(key)
            if action == "exit" then return end

            auto_save_if_needed()
            refresh_dirty_flags()

            if state.dirty then
                render()
                state.dirty = false
                state.top_dirty = false
            elseif state.top_dirty then
                draw_top_status_line(board_geometry())
                state.top_dirty = false
            end

            state.frame = state.frame + 1
        else
            if key == "q" or key == "esc" then return end
        end

        sleep(FRAME_MS)
    end
end

init_game()
game_loop()
