local status = "Press P to request an asset write"
local probes = 0
local best_probes = 0

function Init(ctx)
  if ctx.continue_data ~= nil then
    probes = ctx.continue_data.probes or 0
  end
  if ctx.best_data ~= nil then
    best_probes = ctx.best_data.probes or 0
  end
  debug.log("Safe Mode Lab initialized")
end

function HandleEvent(event)
  if event.type == "action" and event.data.state == "pressed" then
    if event.data.action == "write_probe" then
      probes = probes + 1
      status = "Write requested; wait for the file event"
      file.write{
        path = "state/probe.log",
        text = "probe=" .. base.tostring(probes) .. "\n",
        encoding = file.UTF_8,
        end_of_line = file.LF,
        event_tip = "safe_mode_probe"
      }
    elseif event.data.action == "leave" then
      game.exit_game()
    end
  elseif event.type == "file" and event.data.kind == "write_text" then
    if event.data.ok then
      status = "Probe written: " .. (event.data.path or "state/probe.log")
      debug.log("Safe Mode write probe completed")
    else
      status = "Probe failed: " .. event.data.error.code
      debug.warn(status)
    end
  end
end

function Update(dt)
end

function UpdateFrame(dt, alpha)
end

function Render(surface)
  draw.fill_rect{ x = 0, y = 0, width = surface.width, height = surface.height, char = " ", bg = "black" }
  draw.stroke_rect{ x = 0, y = 0, width = surface.width, height = surface.height, fg = "bright_red" }
  draw.text{ x = 2, y = 1, text = "Safe Mode Lab", fg = "bright_red" }
  draw.text{ x = 2, y = 4, text = status, fg = "bright_gray", max_width = surface.width - 4 }
  draw.text{ x = 2, y = 7, text = "Probe count: " .. base.tostring(probes), fg = "bright_yellow" }
end

function SaveGame()
  return { probes = probes, status = status }
end

function SaveBest()
  best_probes = math.max{ values = { best_probes, probes } }
  return {
    best_string = "f%<fg:bright_red>Completed probes: " .. base.tostring(best_probes) .. "</fg>",
    probes = best_probes
  }
end
