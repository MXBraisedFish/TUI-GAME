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
  x = align.resolve_x {width = 5, horizontal_align = align.CENTER}
  draw.text {x = x, y = 0, text = "Hellow"}
end

function SaveGame()
end

function SaveBest()
end
