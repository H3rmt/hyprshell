local function slot(binding)
    local key = binding.keycode and binding.keycode > 0
        and ("code:" .. binding.keycode) or string.lower(binding.key)
    return tostring(binding.modmask) .. ":" .. key
end

local function identity(handle, binding)
    return handle.handler ~= nil
        and handle.handler == binding.dispatcher and handle.arg == binding.arg
        and handle.modmask == binding.modmask and handle.key == binding.key
        and handle.keycode == binding.keycode and handle.submap == binding.submap
end

local function reconcile(desired, live)
    local registry = rawget(_G, "__hyprshell_bindings_v1")
    if registry == nil then
        registry = { version = 1, entries = {} }
    end
    assert(type(registry) == "table" and registry.version == 1
        and type(registry.entries) == "table", "Invalid Hyprshell binding registry")

    local changed = {}
    local present = {}
    for chord, previous in pairs(registry.entries) do
        assert(type(previous) == "table" and type(previous.handles) == "table"
            and type(previous.specs) == "table", "Invalid Hyprshell binding ownership record")
        changed[chord] = true
    end
    for chord in pairs(desired) do
        changed[chord] = true
    end

    for _, binding in ipairs(live) do
        local chord = slot(binding)
        local previous = registry.entries[chord]
        local owner = false
        if previous then
            for _, handle in ipairs(previous.handles) do
                if identity(handle, binding) then
                    owner = true
                    break
                end
            end
        end
        if changed[chord] then
            if not owner then
                local has_owned = false
                if previous then
                    for _, handle in ipairs(previous.handles) do
                        if handle.handler ~= nil then
                            has_owned = true
                        end
                    end
                end
                assert(not desired[chord] and not has_owned,
                    "Hyprshell will not change unowned binding " .. chord
                    .. " (submap " .. binding.submap .. "). Resolve the conflict or start a fresh session manually.")
            else
                present[chord] = (present[chord] or 0) + 1
            end
        end
    end

    for chord, wanted in pairs(desired) do
        local previous = registry.entries[chord]
        if previous and present[chord] == #wanted and #previous.handles == #wanted
            and #previous.specs == #wanted then
            local same = true
            for index, binding in ipairs(wanted) do
                if previous.specs[index] ~= binding.spec then
                    same = false
                end
            end
            if same then
                changed[chord] = nil
            end
        end
    end

    rawset(_G, "__hyprshell_bindings_v1", registry)
    for chord in pairs(changed) do
        local previous = registry.entries[chord]
        if previous then
            for _, handle in ipairs(previous.handles) do
                if handle.handler ~= nil then
                    handle:remove()
                end
            end
            registry.entries[chord] = nil
        end
        if desired[chord] then
            local entry = { handles = {}, specs = {} }
            registry.entries[chord] = entry
            for _, binding in ipairs(desired[chord]) do
                table.insert(entry.handles, binding.create())
                table.insert(entry.specs, binding.spec)
            end
        end
    end
end

return reconcile
