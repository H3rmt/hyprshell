local reconcile = dofile("crates/exec-lib/src/binds/registry.lua")
local bindings = {}
local serial = 0
local mutations = 0
local fail_create = false

local function create(key, modifier, submap)
    assert(not fail_create, "simulated binding error")
    serial = serial + 1
    mutations = mutations + 1
    local handle = { key = key, keycode = 0, modmask = modifier, submap = submap or "", handler = "__lua", arg = tostring(serial) }
    function handle:remove()
        mutations = mutations + 1
        for _, other in ipairs(bindings) do
            if other.modmask == self.modmask and other.key == self.key then
                other.handler = nil
            end
        end
    end
    table.insert(bindings, handle)
    return handle
end

local function live()
    local result = {}
    for _, handle in ipairs(bindings) do
        if handle.handler ~= nil then
            table.insert(result, {key=handle.key,keycode=handle.keycode,modmask=handle.modmask,submap=handle.submap,dispatcher=handle.handler,arg=handle.arg})
        end
    end
    return result
end

local function desired(key)
    return {["4:" .. string.lower(key)]={{spec=key,create=function() return create(key, 4) end}}}
end

local desktop = create("c", 64)
local desktop_reference = desktop.arg
reconcile(desired("Tab"), live())
local reference = live()[2].arg
mutations = 0
reconcile(desired("Tab"), live())
assert(mutations == 0 and live()[2].arg == reference)
assert(desktop.arg == desktop_reference and desktop.handler == "__lua")

reconcile(desired("grave"), live())
assert(#live() == 2 and live()[2].key == "grave")
reconcile({}, live())
assert(#live() == 1 and desktop.arg == desktop_reference)

reconcile(desired("Tab"), live())
local external = create("Tab", 4, "other-submap")
mutations = 0
assert(not pcall(reconcile, {}, live()))
assert(mutations == 0 and external.handler == "__lua")
assert(desktop.arg == desktop_reference)

bindings = {desktop}
_G.__hyprshell_bindings_v1 = nil
local untracked = create("Tab", 4)
mutations = 0
assert(not pcall(reconcile, desired("Tab"), live()))
assert(mutations == 0 and untracked.handler == "__lua")

bindings = {desktop}
_G.__hyprshell_bindings_v1 = nil
fail_create = true
assert(not pcall(reconcile, desired("Tab"), live()))
fail_create = false
reconcile(desired("Tab"), live())
assert(#live() == 2)

local previous = _G.__hyprshell_bindings_v1.entries["4:tab"]
table.insert(previous.handles, create("Tab", 4))
table.insert(previous.specs, "Tab")
reconcile(desired("Tab"), live())
assert(#live() == 2 and desktop.arg == desktop_reference)

for _, handle in ipairs(bindings) do
    if handle ~= desktop then handle.handler = nil end
end
reconcile(desired("Tab"), live())
assert(#live() == 2)

_G.__hyprshell_bindings_v1 = {version=2,entries={}}
mutations = 0
assert(not pcall(reconcile, desired("Tab"), live()))
assert(mutations == 0)
print("owned binding registry tests passed")
