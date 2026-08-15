local x = 0
local y = 0
local t = { "a", "b", "c", x = 1, [5] = "d" }
local t1 = { "a" }
local t2 = { "a" }
local b = 0
local index = nil

function Init(ctx)

end

function HandleEvent(event)
  if event.data.action == "leave" then
    game.exit_game()
  end
end

function Update(dt)
  -- if i == 0 then
  --   i = 1
  -- end
end

function UpdateFrame(dt, alpha)
end

function Render(surface)
  for item in ipairs(char.ASCII) do
    x = x + 2
    if x % 20 == 0 then
      x = 2
      y = y + 1
    end
    draw.text { x = x, y = y, text = item.value }
  end
  x = 0
  y = 0
end

function SaveGame()
end

function SaveBest()
end
