local player = { x = 8, y = 6 }
local score = 0
local best_score = 0
local elapsed = 0
local width = 80
local height = 24

function Init(ctx)
  width = ctx.base.width
  height = ctx.base.height
  if ctx.continue_data ~= nil then
    player.x = ctx.continue_data.x or player.x
    player.y = ctx.continue_data.y or player.y
    score = ctx.continue_data.score or score
    elapsed = ctx.continue_data.elapsed or elapsed
  end
  if ctx.best_data ~= nil then
    best_score = ctx.best_data.score or best_score
  end
end

function HandleEvent(event)
  if event.type == "resize" then
    width = event.data.width
    height = event.data.height
    return
  end
  if event.type ~= "action" or event.data.state ~= "pressed" then
    return
  end
  if event.data.action == "move_up" then
    player.y = math.max{ values = { 2, player.y - 1 } }
  elseif event.data.action == "move_down" then
    player.y = player.y + 1
  elseif event.data.action == "move_left" then
    player.x = math.max{ values = { 1, player.x - 1 } }
  elseif event.data.action == "move_right" then
    player.x = player.x + 1
  elseif event.data.action == "leave" then
    game.exit_game()
  end
end

function Update(dt)
  elapsed = elapsed + dt
  score = math.floor(elapsed)
end

function UpdateFrame(dt, alpha)
end

function Render()
  player.x = math.min{ values = { player.x, width - 2 } }
  player.y = math.min{ values = { player.y, height - 2 } }
  draw.fill_rect{ x = 0, y = 0, width = width, height = height, char = " ", bg = "black" }
  draw.stroke_rect{ x = 0, y = 0, width = width, height = height, fg = "bright_green" }
  draw.text{ x = 2, y = 1, text = "Trail Runner  Score: " .. base.tostring(score), fg = "bright_yellow" }
  draw.text{ x = player.x, y = player.y, text = "@", fg = "bright_cyan" }
  draw.text{ x = 2, y = height - 2, text = "Move with WASD or arrows. Esc returns safely.", fg = "gray" }
end

function SaveGame()
  return { x = player.x, y = player.y, score = score, elapsed = elapsed }
end

function SaveBest()
  best_score = math.max{ values = { best_score, score } }
  return {
    best_string = "f%<fg:bright_yellow>Best trail: " .. base.tostring(best_score) .. "</fg>",
    score = best_score
  }
end
