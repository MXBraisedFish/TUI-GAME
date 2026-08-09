local phase = 0

function Init(ctx)
end

function HandleEvent(event)
end

function Update(dt)
  phase = phase + dt * 3
end

function UpdateFrame(dt, alpha)
end

function Render(surface)
  draw.fill_rect{ x = 0, y = 0, width = surface.width, height = surface.height, char = " ", bg = "black" }
  for y = 1, surface.height - 2, 2 do
    local offset = math.floor((math.sin(phase + y * 0.25) + 1) * 4)
    local width = math.max{ values = { 1, surface.width - offset * 2 - 2 } }
    draw.fill_rect{
      x = offset + 1,
      y = y,
      width = width,
      height = 1,
      char = "=",
      fg = y % 4 == 1 and "bright_cyan" or "bright_magenta"
    }
  end
end
