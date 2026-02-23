
GAME_META = {
    name = "Solitaire",
    description = "Play FreeCell, Klondike, or Spider Solitaire in one game."
}

local FPS = 60
local FRAME_MS = 16
local MAX_UNDO = 100

local MODE_FREECELL = "freecell"
local MODE_KLONDIKE = "klondike"
local MODE_SPIDER = "spider"

local SUIT_HEART = 1
local SUIT_SPADE = 2
local SUIT_DIAMOND = 3
local SUIT_CLUB = 4

local state = {
    mode = MODE_FREECELL,
    spider_diff = 1,

    tableau = {},
    foundations = {},
    cells = {},
    stock = {},
    waste = {},

    spider_removed = 0,

    cursor_col = 1,
    selected_col = nil,

    frame = 0,
    start_frame = 0,
    end_frame = nil,
    won = false,

    confirm_mode = nil,
    mode_input = false,
    spider_diff_input = false,

    msg_text = "",
    msg_color = "dark_gray",
    msg_until = 0,
    msg_persistent = false,

    dirty = true,
    last_elapsed_sec = -1,
    last_auto_save_sec = 0,

    best = {
        freecell = 0,
        klondike = 0,
        spider1 = 0,
        spider2 = 0,
        spider3 = 0,
    },

    launch_mode = "new",
    undo_stack = {},

    size_warning_active = false,
    last_warn_term_w = 0,
    last_warn_term_h = 0,
    last_warn_min_w = 0,
    last_warn_min_h = 0,
    last_term_w = 0,
    last_term_h = 0,
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
    local lines, current, had = {}, "", false
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
    if not had then return { "" } end
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

local function mode_key()
    if state.mode == MODE_SPIDER then
        return "spider" .. tostring(state.spider_diff)
    end
    return state.mode
end

local function mode_label(mode)
    if mode == MODE_FREECELL then return tr("game.solitaire.mode.freecell", "FreeCell") end
    if mode == MODE_KLONDIKE then return tr("game.solitaire.mode.klondike", "Klondike") end
    return tr("game.solitaire.mode.spider", "Spider")
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
    if card.suit == SUIT_HEART then return "red" end
    if card.suit == SUIT_DIAMOND then return "rgb(255,165,0)" end
    if card.suit == SUIT_CLUB then return "cyan" end
    return "white"
end

local function color_group(card)
    return is_red(card) and 1 or 0
end

local function clone_card(card)
    return { rank = card.rank, suit = card.suit, face_up = card.face_up == true }
end

local function copy_cards(cards)
    local out = {}
    for i = 1, #cards do out[i] = clone_card(cards[i]) end
    return out
end

local function copy_tableau(tableau)
    local out = {}
    for i = 1, #tableau do out[i] = copy_cards(tableau[i] or {}) end
    return out
end

local function copy_foundations(fd)
    local out = {}
    for i = 1, 4 do out[i] = copy_cards(fd[i] or {}) end
    return out
end

local function copy_cells(cells)
    local out = {}
    for i = 1, 4 do
        if cells[i] ~= nil then out[i] = clone_card(cells[i]) else out[i] = nil end
    end
    return out
end

local function empty_mode_piles()
    local cols = 8
    if state.mode == MODE_KLONDIKE then cols = 7 end
    if state.mode == MODE_SPIDER then cols = 10 end

    state.tableau = {}
    for i = 1, cols do state.tableau[i] = {} end

    state.foundations = {}
    for i = 1, 4 do state.foundations[i] = {} end

    state.cells = { nil, nil, nil, nil }
    state.stock = {}
    state.waste = {}
    state.spider_removed = 0
end

local function build_standard_deck()
    local deck = {}
    for suit = SUIT_HEART, SUIT_CLUB do
        for rank = 1, 13 do
            deck[#deck + 1] = { rank = rank, suit = suit, face_up = false }
        end
    end
    return deck
end

local function shuffle_cards(cards)
    for i = #cards, 2, -1 do
        local j = rand_int(i) + 1
        cards[i], cards[j] = cards[j], cards[i]
    end
end

local function build_spider_deck(diff)
    local suits, copies = {}, 2
    if diff == 1 then
        suits = { SUIT_SPADE }
        copies = 8
    elseif diff == 2 then
        suits = { SUIT_SPADE, SUIT_HEART }
        copies = 4
    else
        suits = { SUIT_HEART, SUIT_SPADE, SUIT_DIAMOND, SUIT_CLUB }
        copies = 2
    end

    local deck = {}
    for _, suit in ipairs(suits) do
        for _ = 1, copies do
            for rank = 1, 13 do
                deck[#deck + 1] = { rank = rank, suit = suit, face_up = false }
            end
        end
    end
    return deck
end

local function deal_freecell()
    state.mode = MODE_FREECELL
    empty_mode_piles()

    local deck = build_standard_deck()
    shuffle_cards(deck)

    local idx = 1
    for col = 1, 8 do
        local cnt = (col <= 4) and 7 or 6
        for _ = 1, cnt do
            local c = clone_card(deck[idx])
            idx = idx + 1
            c.face_up = true
            state.tableau[col][#state.tableau[col] + 1] = c
        end
    end
end

local function deal_klondike()
    state.mode = MODE_KLONDIKE
    empty_mode_piles()

    local deck = build_standard_deck()
    shuffle_cards(deck)

    local idx = 1
    for col = 1, 7 do
        for row = 1, col do
            local c = clone_card(deck[idx])
            idx = idx + 1
            c.face_up = (row == col)
            state.tableau[col][#state.tableau[col] + 1] = c
        end
    end

    while idx <= #deck do
        local c = clone_card(deck[idx])
        idx = idx + 1
        c.face_up = false
        state.stock[#state.stock + 1] = c
    end
end

local function deal_spider(diff)
    state.mode = MODE_SPIDER
    state.spider_diff = clamp(math.floor(diff or 1), 1, 3)
    empty_mode_piles()

    local deck = build_spider_deck(state.spider_diff)
    shuffle_cards(deck)

    local idx = 1
    for col = 1, 10 do
        local cnt = (col <= 4) and 6 or 5
        for row = 1, cnt do
            local c = clone_card(deck[idx])
            idx = idx + 1
            c.face_up = (row == cnt)
            state.tableau[col][#state.tableau[col] + 1] = c
        end
    end

    while idx <= #deck do
        local c = clone_card(deck[idx])
        idx = idx + 1
        c.face_up = false
        state.stock[#state.stock + 1] = c
    end
end
local function snapshot_state()
    return {
        mode = state.mode,
        spider_diff = state.spider_diff,
        tableau = copy_tableau(state.tableau),
        foundations = copy_foundations(state.foundations),
        cells = copy_cells(state.cells),
        stock = copy_cards(state.stock),
        waste = copy_cards(state.waste),
        spider_removed = state.spider_removed,
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
    state.mode = (snap.mode == MODE_KLONDIKE or snap.mode == MODE_SPIDER) and snap.mode or MODE_FREECELL
    state.spider_diff = clamp(math.floor(tonumber(snap.spider_diff) or 1), 1, 3)
    state.tableau = copy_tableau(snap.tableau or {})
    state.foundations = copy_foundations(snap.foundations or {})
    state.cells = copy_cells(snap.cells or {})
    state.stock = copy_cards(snap.stock or {})
    state.waste = copy_cards(snap.waste or {})
    state.spider_removed = math.max(0, math.floor(tonumber(snap.spider_removed) or 0))

    local cols = #state.tableau
    if cols < 1 then
        if state.mode == MODE_SPIDER then cols = 10 elseif state.mode == MODE_KLONDIKE then cols = 7 else cols = 8 end
    end
    state.cursor_col = clamp(math.floor(tonumber(snap.cursor_col) or 1), 1, cols)
    local sel = tonumber(snap.selected_col)
    if sel ~= nil then state.selected_col = clamp(math.floor(sel), 1, cols) else state.selected_col = nil end

    state.frame = math.max(0, math.floor(tonumber(snap.frame) or state.frame))
    state.start_frame = math.max(0, math.floor(tonumber(snap.start_frame) or state.start_frame))
    state.end_frame = snap.end_frame and math.max(0, math.floor(tonumber(snap.end_frame) or state.frame)) or nil
    state.won = snap.won == true
    state.last_auto_save_sec = math.max(0, math.floor(tonumber(snap.last_auto_save_sec) or 0))

    state.confirm_mode = nil
    state.mode_input = false
    state.spider_diff_input = false
    if clear_undo == nil or clear_undo then state.undo_stack = {} end
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
    restore_snapshot(snap, false)
    show_message(tr("game.solitaire.undo_done", "Undo successful."), "yellow", 2, false)
    return true
end

local function load_best_record()
    state.best.freecell = 0
    state.best.klondike = 0
    state.best.spider1 = 0
    state.best.spider2 = 0
    state.best.spider3 = 0

    if type(load_data) ~= "function" then return end
    local ok, data = pcall(load_data, "solitaire_best_v2")
    if not ok or type(data) ~= "table" then return end

    state.best.freecell = math.max(0, math.floor(tonumber(data.freecell) or 0))
    state.best.klondike = math.max(0, math.floor(tonumber(data.klondike) or 0))
    state.best.spider1 = math.max(0, math.floor(tonumber(data.spider1) or 0))
    state.best.spider2 = math.max(0, math.floor(tonumber(data.spider2) or 0))
    state.best.spider3 = math.max(0, math.floor(tonumber(data.spider3) or 0))
end

local function save_best_record()
    if type(save_data) ~= "function" then return false end
    local payload = {
        freecell = state.best.freecell,
        klondike = state.best.klondike,
        spider1 = state.best.spider1,
        spider2 = state.best.spider2,
        spider3 = state.best.spider3,
    }
    local ok = pcall(save_data, "solitaire_best_v2", payload)
    return ok
end

local function best_time_for_current_mode()
    local k = mode_key()
    return state.best[k] or 0
end

local function total_foundation_cards()
    local total = 0
    for i = 1, 4 do total = total + #state.foundations[i] end
    return total
end

local function update_best_if_needed()
    if not state.won then return end
    local elapsed = elapsed_seconds()
    local k = mode_key()
    local old = state.best[k] or 0
    if old <= 0 or elapsed < old then
        state.best[k] = elapsed
        save_best_record()
    end
end

local function mode_score()
    if state.mode == MODE_SPIDER then return state.spider_removed * 13 end
    return total_foundation_cards()
end

local function check_win()
    if state.won then return end
    local won = false
    if state.mode == MODE_SPIDER then
        won = state.spider_removed >= 8
    else
        won = total_foundation_cards() >= 52
    end

    if won then
        state.won = true
        state.end_frame = state.frame
        update_best_if_needed()
        if type(update_game_stats) == "function" then
            pcall(update_game_stats, "solitaire", mode_score(), elapsed_seconds())
        end
        show_message(tr("game.solitaire.win_banner", "All cards have been collected!") .. " " .. tr("game.solitaire.result_controls", "[R] Restart  [Q]/[ESC] Exit"), "green", 0, true)
    end
end

local function first_face_up_index(col)
    local pile = state.tableau[col]
    for i = 1, #pile do
        if pile[i].face_up then return i end
    end
    return nil
end

local function reveal_new_top(col)
    if state.mode ~= MODE_KLONDIKE and state.mode ~= MODE_SPIDER then return end
    local pile = state.tableau[col]
    if #pile == 0 then return end
    local t = pile[#pile]
    if not t.face_up then
        t.face_up = true
        state.dirty = true
    end
end

local function can_place_on_tableau(card, dest_col)
    local dest = state.tableau[dest_col]
    local top = dest[#dest]
    if top == nil then
        if state.mode == MODE_KLONDIKE then
            return card.rank == 13
        end
        return true
    end
    if not top.face_up then return false end

    if state.mode == MODE_SPIDER then
        return top.rank == card.rank + 1
    end

    if top.rank ~= card.rank + 1 then return false end
    return color_group(top) ~= color_group(card)
end

local function run_valid_klondike_like(cards, start_idx)
    if start_idx < 1 or start_idx > #cards then return false end
    for i = start_idx, #cards - 1 do
        local a = cards[i]
        local b = cards[i + 1]
        if not a.face_up or not b.face_up then return false end
        if b.rank ~= a.rank - 1 then return false end
        if color_group(a) == color_group(b) then return false end
    end
    return true
end

local function run_valid_spider_same(cards, start_idx)
    if start_idx < 1 or start_idx > #cards then return false end
    for i = start_idx, #cards - 1 do
        local a = cards[i]
        local b = cards[i + 1]
        if not a.face_up or not b.face_up then return false end
        if b.rank ~= a.rank - 1 then return false end
        if b.suit ~= a.suit then return false end
    end
    return true
end

local function count_empty_cells()
    local n = 0
    for i = 1, 4 do if state.cells[i] == nil then n = n + 1 end end
    return n
end

local function count_empty_columns(src_col, dst_col)
    local n = 0
    for c = 1, #state.tableau do
        if c ~= src_col and c ~= dst_col and #state.tableau[c] == 0 then n = n + 1 end
    end
    return n
end

local function max_freecell_movable(src_col, dst_col)
    local cells = count_empty_cells()
    local empties = count_empty_columns(src_col, dst_col)
    return (1 + cells) * (2 ^ empties)
end

local function find_move_start_index(src_col, dst_col)
    local src = state.tableau[src_col]
    if #src <= 0 then return nil end

    if state.mode == MODE_FREECELL or state.mode == MODE_KLONDIKE then
        local start = 1
        if state.mode == MODE_KLONDIKE then
            start = first_face_up_index(src_col)
            if start == nil then return nil end
        end

        for i = start, #src do
            if run_valid_klondike_like(src, i) then
                local len = #src - i + 1
                if state.mode ~= MODE_FREECELL or len <= max_freecell_movable(src_col, dst_col) then
                    if can_place_on_tableau(src[i], dst_col) then
                        return i
                    end
                end
            end
        end
        return nil
    end

    local first = first_face_up_index(src_col)
    if first == nil then return nil end

    for i = first, #src do
        local len = #src - i + 1
        local ok_run = false
        if len == 1 then ok_run = true else ok_run = run_valid_spider_same(src, i) end
        if ok_run and can_place_on_tableau(src[i], dst_col) then return i end
    end

    return nil
end

local function move_tableau_stack(src_col, dst_col)
    if src_col < 1 or src_col > #state.tableau or dst_col < 1 or dst_col > #state.tableau then return false end
    if src_col == dst_col then return false end

    local src = state.tableau[src_col]
    local dst = state.tableau[dst_col]
    if #src == 0 then return false end

    local start_idx = find_move_start_index(src_col, dst_col)
    if start_idx == nil then return false end

    push_undo()
    for i = start_idx, #src do dst[#dst + 1] = src[i] end
    for i = #src, start_idx, -1 do table.remove(src, i) end
    reveal_new_top(src_col)
    state.selected_col = nil
    state.dirty = true
    return true
end

local function can_place_foundation(slot, card)
    local pile = state.foundations[slot]
    local top = pile[#pile]
    if top == nil then
        return card.rank == 1
    end
    return top.suit == card.suit and card.rank == top.rank + 1
end

local function move_card_to_foundation(card)
    if state.mode == MODE_SPIDER then return false end
    local slot = card.suit
    if slot < 1 or slot > 4 then return false end
    if not can_place_foundation(slot, card) then return false end
    state.foundations[slot][#state.foundations[slot] + 1] = card
    return true
end

local function move_column_top_to_foundation(col)
    if state.mode == MODE_SPIDER then return false end
    if col < 1 or col > #state.tableau then return false end
    local pile = state.tableau[col]
    local card = pile[#pile]
    if card == nil or not card.face_up then return false end

    push_undo()
    table.remove(pile, #pile)
    if not move_card_to_foundation(card) then
        pile[#pile + 1] = card
        table.remove(state.undo_stack)
        return false
    end

    reveal_new_top(col)
    state.dirty = true
    check_win()
    return true
end

local function move_waste_to_foundation()
    if state.mode ~= MODE_KLONDIKE then return false end
    local card = state.waste[#state.waste]
    if card == nil then return false end

    push_undo()
    table.remove(state.waste, #state.waste)
    if not move_card_to_foundation(card) then
        state.waste[#state.waste + 1] = card
        table.remove(state.undo_stack)
        return false
    end

    state.dirty = true
    check_win()
    return true
end

local function move_waste_to_column(col)
    if state.mode ~= MODE_KLONDIKE then return false end
    if col < 1 or col > #state.tableau then return false end
    local card = state.waste[#state.waste]
    if card == nil then return false end
    if not can_place_on_tableau(card, col) then return false end

    push_undo()
    table.remove(state.waste, #state.waste)
    state.tableau[col][#state.tableau[col] + 1] = card
    state.dirty = true
    return true
end
local function move_column_to_cell(col)
    if state.mode ~= MODE_FREECELL then return false end
    if col < 1 or col > #state.tableau then return false end

    local empty_slot = nil
    for i = 1, 4 do
        if state.cells[i] == nil then empty_slot = i; break end
    end
    if empty_slot == nil then return false end

    local pile = state.tableau[col]
    local card = pile[#pile]
    if card == nil then return false end

    push_undo()
    table.remove(pile, #pile)
    state.cells[empty_slot] = card
    state.dirty = true
    return true
end

local function move_cell_to_column(col)
    if state.mode ~= MODE_FREECELL then return false end
    if col < 1 or col > #state.tableau then return false end

    for i = 1, 4 do
        local card = state.cells[i]
        if card ~= nil and can_place_on_tableau(card, col) then
            push_undo()
            state.cells[i] = nil
            state.tableau[col][#state.tableau[col] + 1] = card
            state.dirty = true
            return true
        end
    end
    return false
end

local function move_cell_to_foundation()
    if state.mode ~= MODE_FREECELL then return false end
    for i = 1, 4 do
        local card = state.cells[i]
        if card ~= nil then
            if can_place_foundation(card.suit, card) then
                push_undo()
                state.cells[i] = nil
                state.foundations[card.suit][#state.foundations[card.suit] + 1] = card
                state.dirty = true
                check_win()
                return true
            end
        end
    end
    return false
end

local function draw_from_stock_klondike()
    if state.mode ~= MODE_KLONDIKE then return false end

    if #state.stock == 0 then
        if #state.waste == 0 then
            show_message(tr("game.solitaire.stock_empty", "No cards in stock and waste."), "dark_gray", 2, false)
            return false
        end

        push_undo()
        local recycled = {}
        for i = #state.waste, 1, -1 do
            local card = state.waste[i]
            card.face_up = false
            recycled[#recycled + 1] = card
        end
        state.stock = recycled
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

local function draw_spider_row()
    if state.mode ~= MODE_SPIDER then return false end
    if #state.stock < 10 then
        show_message(tr("game.solitaire.spider_no_stock", "No more stock rows."), "dark_gray", 2, false)
        return false
    end

    for c = 1, 10 do
        if #state.tableau[c] == 0 then
            show_message(tr("game.solitaire.spider_need_full", "Cannot deal while any tableau column is empty."), "red", 2, false)
            return false
        end
    end

    push_undo()
    for c = 1, 10 do
        local card = state.stock[#state.stock]
        table.remove(state.stock, #state.stock)
        card.face_up = true
        state.tableau[c][#state.tableau[c] + 1] = card
    end
    state.dirty = true
    return true
end

local function remove_spider_complete_runs()
    if state.mode ~= MODE_SPIDER then return false end
    local changed = false

    for c = 1, 10 do
        local pile = state.tableau[c]
        while #pile >= 13 do
            local start = #pile - 12
            local suit = pile[start].suit
            local ok = pile[start].face_up == true and pile[start].rank == 13
            if ok then
                for i = start, #pile - 1 do
                    local a = pile[i]
                    local b = pile[i + 1]
                    if not a.face_up or not b.face_up then ok = false; break end
                    if b.suit ~= suit then ok = false; break end
                    if b.rank ~= a.rank - 1 then ok = false; break end
                end
            end

            if not ok then break end

            for i = #pile, start, -1 do table.remove(pile, i) end
            state.spider_removed = state.spider_removed + 1
            reveal_new_top(c)
            changed = true
            show_message(tr("game.solitaire.spider_removed", "Completed sequence removed."), "green", 2, false)
        end
    end

    if changed then
        state.dirty = true
        check_win()
    end
    return changed
end

local function save_progress(manual)
    local snap = snapshot_state()
    local ok = false
    if type(save_game_slot) == "function" then
        ok = pcall(save_game_slot, "solitaire", snap)
    elseif type(save_data) == "function" then
        ok = pcall(save_data, "solitaire_v2", snap)
    end

    if manual then
        if ok then
            show_message(tr("game.solitaire.save_success", "Save successful!"), "green", 2, false)
        else
            show_message(tr("game.solitaire.save_unavailable", "Save unavailable."), "red", 2, false)
        end
    end

    return ok
end

local function try_load_progress()
    local data = nil
    if type(load_game_slot) == "function" then
        local ok, ret = pcall(load_game_slot, "solitaire")
        if ok and type(ret) == "table" then data = ret end
    end
    if data == nil and type(load_data) == "function" then
        local ok, ret = pcall(load_data, "solitaire_v2")
        if ok and type(ret) == "table" then data = ret end
    end
    if data == nil then return false end

    if type(data.tableau) ~= "table" or type(data.foundations) ~= "table" then return false end

    restore_snapshot(data, true)
    return true
end

local function deal_new_game(mode, diff)
    if mode == MODE_SPIDER then
        deal_spider(diff or state.spider_diff)
    elseif mode == MODE_KLONDIKE then
        deal_klondike()
    else
        deal_freecell()
    end

    state.cursor_col = 1
    state.selected_col = nil
    state.confirm_mode = nil
    state.mode_input = false
    state.spider_diff_input = false
    state.start_frame = state.frame
    state.end_frame = nil
    state.won = false
    state.last_auto_save_sec = 0
    state.undo_stack = {}
    clear_message()
    flush_input_buffer()
    state.dirty = true
end

local function minimum_size()
    local cols = #state.tableau
    if cols <= 0 then cols = 8 end

    local row_label_w = 4
    local col_cell_w = 4
    local gap = 1
    local grid_w = row_label_w + cols * col_cell_w + (cols - 1) * gap + 2

    local controls = tr("game.solitaire.controls." .. state.mode, "")
    local controls_w = min_width_for_lines(controls, 3, 32)

    local min_w = math.max(70, grid_w + 6, controls_w + 4)
    local min_h = 24
    if state.mode == MODE_SPIDER then min_h = 26 end
    return min_w, min_h
end

local function draw_size_warning(term_w, term_h, min_w, min_h)
    clear()
    local title = tr("warning.size_title", "Terminal Too Small")
    local req = tr("warning.required", "Required") .. ": " .. tostring(min_w) .. "x" .. tostring(min_h)
    local cur = tr("warning.current", "Current") .. ": " .. tostring(term_w) .. "x" .. tostring(term_h)
    local hint = tr("warning.enlarge_hint", "Please enlarge the terminal window.")

    local y = math.max(2, math.floor(term_h / 2) - 2)
    draw_text(centered_x(title, 1, term_w), y, title, "yellow", "black")
    draw_text(centered_x(req, 1, term_w), y + 1, req, "white", "black")
    draw_text(centered_x(cur, 1, term_w), y + 2, cur, "white", "black")
    draw_text(centered_x(hint, 1, term_w), y + 3, hint, "dark_gray", "black")
end

local function card_two_chars(card)
    local rt = rank_text(card.rank)
    if rt == "10" then return "10" end
    return " " .. rt
end

local function draw_column_frame(x, y, h, color)
    if h < 1 then h = 1 end
    draw_text(x, y - 1, "┌──┐", color, "black")
    for i = 0, h - 1 do
        draw_text(x, y + i, "│", color, "black")
        draw_text(x + 3, y + i, "│", color, "black")
    end
    draw_text(x, y + h, "└──┘", color, "black")
end

local function draw_cards_grid(g, max_visible_rows)
    local cols = #state.tableau
    local max_rows = 1
    for c = 1, cols do
        if #state.tableau[c] > max_rows then max_rows = #state.tableau[c] end
    end

    local rows_to_draw = math.max(max_rows, 19)
    if max_visible_rows ~= nil then
        rows_to_draw = math.min(rows_to_draw, math.max(1, max_visible_rows))
    end

    for r = 1, rows_to_draw do
        draw_text(g.x, g.y + r - 1, string.format("R%-2d", r), "dark_gray", "black")
    end

    for c = 1, cols do
        local cx = g.x + 5 + (c - 1) * 5
        draw_text(cx, g.y - 1, string.format("C%-2d", c), "dark_gray", "black")

        local pile = state.tableau[c]
        for r = 1, rows_to_draw do
            local text = "  "
            local fg = "dark_gray"
            if r <= #pile then
                local card = pile[r]
                if card.face_up then
                    text = card_two_chars(card)
                    fg = card_color(card)
                else
                    text = "##"
                    fg = "dark_gray"
                end
            end
            draw_text(cx + 1, g.y + r - 1, text, fg, "black")
        end
    end

    for c = 1, cols do
        local pile_h = #state.tableau[c]
        if pile_h < 1 then pile_h = 1 end
        pile_h = math.min(pile_h, rows_to_draw)
        local frame_x = g.x + 5 + (c - 1) * 5
        if state.selected_col == c then
            draw_column_frame(frame_x, g.y, pile_h, "green")
        end
        if state.cursor_col == c then
            draw_column_frame(frame_x, g.y, pile_h, "yellow")
        end
    end
end
local function foundation_label(slot)
    local pile = state.foundations[slot]
    if #pile == 0 then return "[ ]" end
    return "[" .. rank_text(pile[#pile].rank) .. "]"
end
local function draw_top_bar(term_w)
    local best = best_time_for_current_mode()
    local best_text = best > 0 and format_duration(best) or "--:--:--"
    local mode_text = mode_label(state.mode)
    if state.mode == MODE_SPIDER then
        mode_text = mode_text .. " " .. tostring(state.spider_diff)
    end

    local line1 = tr("game.solitaire.time", "Time") .. " " .. format_duration(elapsed_seconds())
        .. "   " .. tr("game.solitaire.mode", "Mode") .. " " .. mode_text
        .. "   " .. tr("game.solitaire.mode_best", "Mode Best") .. " " .. best_text
    draw_text(centered_x(line1, 1, term_w), 2, line1, "cyan", "black")

    if state.mode == MODE_FREECELL then
        local cells = ""
        for i = 1, 4 do
            if state.cells[i] == nil then
                cells = cells .. "[  ] "
            else
                cells = cells .. "[" .. card_two_chars(state.cells[i]) .. "] "
            end
        end
        local f = foundation_label(1) .. " " .. foundation_label(2) .. " " .. foundation_label(3) .. " " .. foundation_label(4)
        local line2 = tr("game.solitaire.cells", "Cells") .. " " .. cells .. "   " .. tr("game.solitaire.foundations", "Foundations") .. " " .. f
        draw_text(centered_x(line2, 1, term_w), 4, line2, "white", "black")
    elseif state.mode == MODE_KLONDIKE then
        local w1, w2, w3 = "  ", "  ", "  "
        if #state.waste >= 1 then w1 = card_two_chars(state.waste[#state.waste]) end
        if #state.waste >= 2 then w2 = card_two_chars(state.waste[#state.waste - 1]) end
        if #state.waste >= 3 then w3 = card_two_chars(state.waste[#state.waste - 2]) end
        local f = foundation_label(1) .. " " .. foundation_label(2) .. " " .. foundation_label(3) .. " " .. foundation_label(4)
        local line2 = tr("game.solitaire.stock", "Stock") .. " [##]   " .. tr("game.solitaire.waste", "Waste") .. " [" .. w3 .. " " .. w2 .. " " .. w1 .. "]"
            .. "   " .. tr("game.solitaire.foundations", "Foundations") .. " " .. f
        draw_text(centered_x(line2, 1, term_w), 4, line2, "white", "black")
    else
        local line2 = tr("game.solitaire.spider_stock", "Spider Stock") .. " " .. tostring(math.floor(#state.stock / 10))
            .. "   " .. tr("game.solitaire.spider_removed", "Removed") .. " " .. tostring(state.spider_removed) .. "/8"
        draw_text(centered_x(line2, 1, term_w), 4, line2, "white", "black")
    end
end

local function current_message()
    if state.mode_input then
        if state.spider_diff_input then
            return tr("game.solitaire.mode_prompt_spider", "Spider difficulty: [1] Easy [2] Medium [3] Hard"), "yellow"
        end
        return tr("game.solitaire.mode_prompt", "Switch mode: [F] FreeCell / [K] Klondike / [S] Spider"), "yellow"
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

local function controls_text()
    if state.mode == MODE_FREECELL then
        return tr("game.solitaire.controls.freecell", "[←]/[→] Move  [Space] Select Column  [Enter] Move  [X] To Cell  [C] Cell->Column  [P] Switch Mode  [A] Undo  [S] Save  [R] Restart  [Q]/[ESC] Exit")
    elseif state.mode == MODE_KLONDIKE then
        return tr("game.solitaire.controls.klondike", "[←]/[→] Move  [Space] Select Column  [Enter] Move/Foundation  [X] Draw  [C] Waste->Column  [P] Switch Mode  [A] Undo  [S] Save  [R] Restart  [Q]/[ESC] Exit")
    end
    return tr("game.solitaire.controls.spider", "[←]/[→] Move  [Space] Select Column  [Enter] Move  [X] Deal Row  [P] Switch Mode  [A] Undo  [S] Save  [R] Restart  [Q]/[ESC] Exit")
end

local function render()
    local term_w, term_h = terminal_size()
    local min_w, min_h = minimum_size()
    state.last_term_w, state.last_term_h = term_w, term_h

    if term_w < min_w or term_h < min_h then
        state.size_warning_active = true
        state.last_warn_term_w, state.last_warn_term_h = term_w, term_h
        state.last_warn_min_w, state.last_warn_min_h = min_w, min_h
        draw_size_warning(term_w, term_h, min_w, min_h)
        state.dirty = false
        return
    end

    state.size_warning_active = false

    clear()
    draw_top_bar(term_w)

    local cols = #state.tableau
    local grid_w = 4 + cols * 5
    local grid_x = math.max(2, math.floor((term_w - grid_w) / 2))
    local grid_y = 8

    local controls = controls_text()
    local wrapped = wrap_words(controls, term_w - 2)
    local controls_lines = #wrapped
    if controls_lines < 1 then controls_lines = 1 end
    if controls_lines > 3 then controls_lines = 3 end

    local reserved_bottom = controls_lines + 2
    local max_visible_rows = term_h - grid_y - reserved_bottom
    if max_visible_rows < 1 then max_visible_rows = 1 end

    local g = { x = grid_x, y = grid_y }
    draw_cards_grid(g, max_visible_rows)

    local msg, msg_color = current_message()
    local controls_start_y = term_h - controls_lines + 1
    if msg ~= "" then
        local msg_y = controls_start_y - 2
        if msg_y >= 1 then
            draw_text(centered_x(msg, 1, term_w), msg_y, msg, msg_color, "black")
        end
    end

    if #wrapped > 3 then
        draw_text(centered_x(tr("warning.size_title", "Terminal Too Small"), 1, term_w), controls_start_y, tr("warning.size_title", "Terminal Too Small"), "yellow", "black")
    else
        for i = 1, #wrapped do
            draw_text(centered_x(wrapped[i], 1, term_w), controls_start_y + i - 1, wrapped[i], "white", "black")
        end
    end

    state.dirty = false
end
local function auto_save_tick()
    if state.won then return end
    local sec = elapsed_seconds()
    if sec > 0 and sec % 60 == 0 and sec ~= state.last_auto_save_sec then
        save_progress(false)
        state.last_auto_save_sec = sec
    end
end

local function handle_mode_input_key(key)
    if key == "esc" or key == "q" or key == "z" then
        state.mode_input = false
        state.spider_diff_input = false
        state.dirty = true
        return
    end

    if state.spider_diff_input then
        if key == "1" or key == "2" or key == "3" then
            state.mode_input = false
            state.spider_diff_input = false
            deal_new_game(MODE_SPIDER, tonumber(key))
        end
        return
    end

    if key == "f" then
        state.mode_input = false
        deal_new_game(MODE_FREECELL)
        return
    elseif key == "k" then
        state.mode_input = false
        deal_new_game(MODE_KLONDIKE)
        return
    elseif key == "s" then
        state.spider_diff_input = true
        state.dirty = true
        return
    end
end

local function handle_confirm_key(key)
    if key == "y" then
        if state.confirm_mode == "restart" then
            deal_new_game(state.mode, state.spider_diff)
        elseif state.confirm_mode == "exit" then
            exit_game()
        end
        state.confirm_mode = nil
    elseif key == "n" or key == "esc" or key == "q" then
        state.confirm_mode = nil
        state.dirty = true
    end
end

local function handle_result_key(key)
    if key == "r" then
        deal_new_game(state.mode, state.spider_diff)
    elseif key == "q" or key == "esc" then
        exit_game()
    end
end

local function selectable_column(col)
    if col < 1 or col > #state.tableau then return false end
    local pile = state.tableau[col]
    if #pile == 0 then return false end
    if state.mode == MODE_FREECELL then return true end
    return first_face_up_index(col) ~= nil
end

local function handle_normal_key(key)
    if key == "left" then
        state.cursor_col = clamp(state.cursor_col - 1, 1, #state.tableau)
        state.dirty = true
        return
    elseif key == "right" then
        state.cursor_col = clamp(state.cursor_col + 1, 1, #state.tableau)
        state.dirty = true
        return
    end

    if key == "space" then
        if selectable_column(state.cursor_col) then
            state.selected_col = state.cursor_col
            state.dirty = true
        else
            show_message(tr("game.solitaire.select_empty", "Current column cannot be selected."), "dark_gray", 2, false)
        end
        return
    end

    if key == "enter" then
        if state.selected_col ~= nil then
            local src, dst = state.selected_col, state.cursor_col
            if src ~= dst and move_tableau_stack(src, dst) then
                if state.mode == MODE_SPIDER then remove_spider_complete_runs() else check_win() end
            else
                show_message(tr("game.solitaire.move_invalid", "Invalid move."), "red", 2, false)
            end
        else
            if not move_column_top_to_foundation(state.cursor_col) then
                if state.mode == MODE_KLONDIKE then move_waste_to_foundation() end
                if state.mode == MODE_FREECELL then move_cell_to_foundation() end
            end
        end
        return
    end

    if key == "z" then
        state.selected_col = nil
        state.dirty = true
        return
    end

    if key == "x" then
        if state.mode == MODE_KLONDIKE then
            draw_from_stock_klondike()
        elseif state.mode == MODE_SPIDER then
            draw_spider_row()
            remove_spider_complete_runs()
        else
            if not move_column_to_cell(state.cursor_col) then
                show_message(tr("game.solitaire.cell_full", "No empty FreeCell."), "dark_gray", 2, false)
            end
        end
        return
    end

    if key == "c" then
        if state.mode == MODE_KLONDIKE then
            if not move_waste_to_column(state.cursor_col) then
                show_message(tr("game.solitaire.waste_invalid", "Waste card cannot be placed there."), "red", 2, false)
            end
        elseif state.mode == MODE_FREECELL then
            if not move_cell_to_column(state.cursor_col) then
                show_message(tr("game.solitaire.cell_empty", "No suitable card in FreeCells."), "dark_gray", 2, false)
            end
        end
        return
    end

    if key == "p" then
        state.mode_input = true
        state.spider_diff_input = false
        state.dirty = true
        return
    end

    if key == "a" then
        pop_undo()
        return
    end

    if key == "s" then
        save_progress(true)
        return
    end

    if key == "r" then
        state.confirm_mode = "restart"
        state.dirty = true
        return
    end

    if key == "q" or key == "esc" then
        state.confirm_mode = "exit"
        state.dirty = true
        return
    end
end
local function load_launch_mode()
    if type(get_launch_mode) ~= "function" then return "new" end
    local ok, mode = pcall(get_launch_mode)
    if ok and mode == "continue" then return "continue" end
    return "new"
end

local function init_game()
    load_best_record()
    state.launch_mode = load_launch_mode()

    if state.launch_mode == "continue" and try_load_progress() then
        state.start_frame = state.frame - elapsed_seconds() * FPS
        state.dirty = true
        show_message(tr("game.solitaire.continue_loaded", "Loaded previous save."), "green", 2, false)
    else
        deal_new_game(MODE_FREECELL)
    end
end

local function input_tick()
    local key = normalize_key(get_key(false))
    if key == "" then return end

    if state.mode_input then
        handle_mode_input_key(key)
        return
    end

    if state.confirm_mode ~= nil then
        handle_confirm_key(key)
        return
    end

    if state.won then
        handle_result_key(key)
        return
    end

    handle_normal_key(key)
end

init_game()

while true do
    state.frame = state.frame + 1

    local sec = elapsed_seconds()
    if sec ~= state.last_elapsed_sec then
        state.last_elapsed_sec = sec
        state.dirty = true
    end

    auto_save_tick()
    update_message_timer()
    input_tick()

    local tw, th = terminal_size()
    if tw ~= state.last_term_w or th ~= state.last_term_h then
        state.dirty = true
    end

    if state.size_warning_active then
        local min_w, min_h = minimum_size()
        if tw ~= state.last_warn_term_w or th ~= state.last_warn_term_h or min_w ~= state.last_warn_min_w or min_h ~= state.last_warn_min_h then
            state.dirty = true
        end
    end

    if state.dirty then
        render()
    end

    sleep(FRAME_MS)
end



