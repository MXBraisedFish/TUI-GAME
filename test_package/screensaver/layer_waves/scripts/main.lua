local panel = nil
local phase = 0

function Init(ctx)
  panel = slice.create{ width = slice["75P"], height = slice["50P"], layer = 10 }
end

function HandleEvent(event)
end

function Update(dt)
  phase = phase + dt * 2
end

function UpdateFrame(dt, alpha)
end

function Render(surface)
  local panel_width = slice.get_width(panel)
  local panel_height = slice.get_height(panel)
  slice.draw{
    id = panel,
    x = math.floor((surface.width - panel_width) / 2),
    y = math.floor((surface.height - panel_height) / 2)
  }
  draw.fill_rect{ x = 0, y = 0, width = surface.width, height = surface.height, char = " ", bg = "black" }
  draw.fill_rect{ x = 0, y = 0, width = panel_width, height = panel_height, char = " ", bg = "blue", slice_layer = panel }
  draw.stroke_rect{ x = 0, y = 0, width = panel_width, height = panel_height, fg = "bright_cyan", slice_layer = panel }
  for y = 2, panel_height - 2, 2 do
    local x = math.floor((math.sin(phase + y * 0.4) + 1) * (panel_width - 4) / 2) + 1
    draw.text{ x = x, y = y, text = "~", fg = "white", slice_layer = panel }
  end
end
