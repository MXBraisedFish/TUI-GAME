TEXT_COMMANDS = TEXT_COMMANDS or {}

TEXT_COMMANDS.tc = function(params, _ctx)
    local p1 = (params and params[1]) or ""
    if p1 == nil or p1 == "" then
        return { error = "参数无效" }
    end

    if string.lower(p1) == "clear" then
        if params[2] ~= nil and tostring(params[2]) ~= "" then
            return { error = "参数无效" }
        end
        return { clear = true }
    end

    local out = { clear = false, color = tostring(p1) }
    if params[2] ~= nil and tostring(params[2]) ~= "" then
        local n = tonumber(params[2])
        if not n or n < 1 then
            return { error = "参数无效" }
        end
        out.count = math.floor(n)
    end
    return out
end
