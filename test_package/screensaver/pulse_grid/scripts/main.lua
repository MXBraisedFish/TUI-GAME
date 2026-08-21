local phase = 0
local width = 80
local height = 24

function Init(ctx)
  width = ctx.base.width
  height = ctx.base.height
end

function HandleEvent(event)
  if event.type == "resize" then
    width = event.data.width
    height = event.data.height
  end
end

function Update(dt)
  phase = phase + dt * 3
end

function UpdateFrame(dt, alpha)
end

function Render()
  draw.fill_rect{ x = 0, y = 0, width = width, height = height, char = " ", bg = "black" }
  for y = 1, height - 2, 2 do
    local offset = math.floor((math.sin(phase + y * 0.25) + 1) * 4)
    local line_width = math.max{ values = { 1, width - offset * 2 - 2 } }
    draw.fill_rect{
      x = offset + 1,
      y = y,
      width = line_width,
      height = 1,
      char = "=",
      fg = y % 4 == 1 and "bright_cyan" or "bright_magenta"
    }
  end
end
