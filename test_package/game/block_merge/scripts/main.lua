local blocks = { 1, 2, 3, 4 }
local score = 0
local best_score = 0
local encoded_preview = ""

local function refresh_preview()
  encoded_preview = serialization.json_encode(blocks)
end

function Init(ctx)
  if ctx.continue_data ~= nil and ctx.continue_data.blocks ~= nil then
    blocks = ctx.continue_data.blocks
    score = ctx.continue_data.score or 0
  end
  if ctx.best_data ~= nil then
    best_score = ctx.best_data.score or 0
  end
  refresh_preview()
end

function HandleEvent(event)
  if event.type ~= "action" or event.data.state ~= "pressed" then
    return
  end
  if event.data.action == "roll" then
    blocks[#blocks + 1] = random.randint{ min = 1, max = 9 }
    score = score + blocks[#blocks]
    refresh_preview()
  elseif event.data.action == "merge" then
    local total = 0
    for item in ipairs(blocks) do
      total = total + item.value
    end
    blocks = { total }
    score = score + total
    refresh_preview()
  elseif event.data.action == "leave" then
    game.exit_game()
  end
end

function Update(dt)
end

function UpdateFrame(dt, alpha)
end

function Render(surface)
  draw.fill_rect{ x = 0, y = 0, width = surface.width, height = surface.height, char = " ", bg = "black" }
  draw.stroke_rect{ x = 0, y = 0, width = surface.width, height = surface.height, fg = "bright_magenta" }
  draw.text{ x = 2, y = 1, text = "Block Merge  Score: " .. base.tostring(score), fg = "bright_yellow" }
  draw.text{ x = 2, y = 4, text = "Board JSON: " .. encoded_preview, fg = "bright_cyan", max_width = surface.width - 4 }
  draw.text{ x = 2, y = 7, text = "Space: roll    Enter: merge    Esc: leave", fg = "bright_gray" }
end

function SaveGame()
  return serialization.json_decode(serialization.json_encode{
    t = { blocks = blocks, score = score }
  })
end

function SaveBest()
  best_score = math.max{ values = { best_score, score } }
  return {
    best_string = "f%<fg:bright_magenta>Best merge: " .. base.tostring(best_score) .. "</fg>",
    score = best_score
  }
end
