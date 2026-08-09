local stars = {}
local width = 80
local height = 24

function Init(ctx)
  width = ctx.terminal.width
  height = ctx.terminal.height
  for index = 1, 32 do
    stars[index] = {
      x = random.randint{ min = 0, max = width - 1 },
      y = random.randint{ min = 0, max = height - 1 },
      speed = random.randint{ min = 1, max = 3 }
    }
  end
end

function HandleEvent(event)
  if event.type == "resize" then
    width = event.data.width
    height = event.data.height
  end
end

function Update(dt)
  for _, star in base.ipairs(stars) do
    star.x = star.x - star.speed
    if star.x < 0 then
      star.x = width - 1
      star.y = random.randint{ min = 0, max = height - 1 }
    end
  end
end

function UpdateFrame(dt, alpha)
end

function Render(surface)
  draw.fill_rect{ x = 0, y = 0, width = surface.width, height = surface.height, char = " ", bg = "black" }
  for _, star in base.ipairs(stars) do
    if star.x < surface.width and star.y < surface.height then
      draw.text{ x = star.x, y = star.y, text = star.speed == 3 and "*" or ".", fg = star.speed == 3 and "white" or "gray" }
    end
  end
end
