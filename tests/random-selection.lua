-- Run from the repository root with Lua 5.3 (or texlua using Lua 5.3).
-- Compile the entire game script, then execute its actual selection helper and
-- finalization block with mocked game calls. No game or NRO is loaded here.
assert(_VERSION == "Lua 5.3", "Use a Lua 5.3 interpreter for the game script")
assert(loadfile("stage_select_actor3.lua"))
local file = assert(io.open("stage_select_actor3.lua", "r"))
local source = file:read("*a")
file:close()

local helper_start = assert(source:find("local capture_alt_selection = function(preview_index)", 1, true))
local helper_end = assert(source:find("\nend\n", helper_start, true))
local helper = source:sub(helper_start, helper_end + 3) .. "\nreturn capture_alt_selection"
local dispatch_start = assert(source:find("    -- Snapshot alt choices before finalizing", 1, true))
local dispatch_end = assert(source:find("\nend\n\nget_tab_switch", dispatch_start, true))
local dispatch = "return function()\n" .. source:sub(dispatch_start, dispatch_end - 1) .. "\nend"

local function preview(panel, alt, form, random, custom)
    return {
        panel_id_ = panel, selected_alt_ = alt, form_type_ = form,
        random = random, is_sub_stage_ = custom or false
    }
end

local function exercise(previews, canceled)
    local finalized, stopped, received = false, false, nil
    local queries, logs = {}, {}
    local context = setmetatable({
        stage_previews = previews, USE_STAGE_NUM = #previews,
        exit_code_ = canceled and 1 or 0, SCENE_EXIT_CODE_NORMAL = 0,
        stop_long_cancel_se = function() stopped = true end,
        Alts = {
            send_message = function(message) logs[#logs + 1] = message end,
            set_alts = function(...)
                assert(finalized, "Keep the existing post-finalization Rust dispatch order")
                received = table.pack(...)
            end
        },
        UiScriptPlayer = {
            invoke = function(name, index)
                if name == "is_random_stage_preview" then
                    assert(not finalized, "Read the preview before it is finalized")
                    queries[#queries + 1] = index
                    return assert(previews[index + 1]).random
                end
                assert(name == "finalize_bgm", "Unexpected game call: " .. tostring(name))
                assert(not finalized, "Finalize exactly once")
                finalized = true
                -- Prove dispatch uses the snapshot even if finalization changes
                -- the live preview and its Random state.
                for _, item in ipairs(previews) do
                    item.panel_id_, item.selected_alt_, item.form_type_ = 99, 99, 99
                    item.random = false
                end
            end
        }
    }, {__index = _G})
    context.capture_alt_selection = assert(load(helper, "@production-selection-helper", "t", context))()
    assert(load(dispatch, "@production-selection-dispatch", "t", context))()()
    assert(finalized and stopped)
    return received, queries, logs
end

local function expect(values, wanted)
    assert(values and values.n == 9, "Preserve the nine-argument Rust ABI")
    for index = 1, 9 do
        local expected = wanted[index]
        if expected == nil then expected = -1 end
        assert(values[index] == expected,
            string.format("Argument %d: expected %s, got %s", index, tostring(expected), tostring(values[index])))
    end
end

local passed = 0
local function test(name, body)
    body()
    passed = passed + 1
    print("PASS " .. name)
end

test("Random panel zero clears a stale manual alt", function()
    local args, queries, logs = exercise({preview(0, 2, 0, true)})
    expect(args, {0, -1, 0})
    assert(#queries == 1 and queries[1] == 0)
    assert(logs[1]:find("random=true raw_panel=0 sent_panel=-1 alt=0 form=0", 1, true))
end)
test("Random negative panel keeps RandomAuto", function()
    expect(exercise({preview(-1, 5, 0, true)}), {0, -1, 0})
end)
test("Random detection does not depend on a particular panel number", function()
    expect(exercise({preview(27, 3, 0, true)}), {0, -1, 0})
end)
test("A real manual panel zero remains manual", function()
    expect(exercise({preview(0, 2, 0, false)}), {2, 0, 0})
end)
test("Manual alt and stage form survive finalization", function()
    expect(exercise({preview(6, 3, 2, false)}), {3, 6, 2})
end)
test("Random Battlefield form is retained", function()
    expect(exercise({preview(0, 4, 1, true)}), {0, -1, 1})
end)
test("Random Omega form is retained", function()
    expect(exercise({preview(0, 4, 2, true)}), {0, -1, 2})
end)
test("Cancel finalizes without querying or sending selections", function()
    local args, queries, logs = exercise({preview(0, 2, 0, true)}, true)
    assert(args == nil and #queries == 0 and #logs == 0)
end)
test("Unused previews remain invalid", function()
    local args, queries = exercise({})
    expect(args, {})
    assert(#queries == 0)
end)
test("Multiple previews preserve serialization order and unused fields", function()
    local args, queries = exercise({preview(6, 2, 0, false), preview(4, 1, 2, false)})
    expect(args, {2, 6, 0, 1, 4, 2})
    assert(#queries == 2 and queries[1] == 0 and queries[2] == 1)
end)
test("All three previews are captured before finalization", function()
    local args, queries = exercise({preview(6, 2, 0, false), preview(4, 1, 2, false), preview(3, 5, 1, false)})
    expect(args, {2, 6, 0, 1, 4, 2, 5, 3, 1})
    assert(#queries == 3 and queries[3] == 2)
end)
test("Custom stages are not reclassified using the standard Random flag", function()
    local args, queries = exercise({preview(0, 0, 0, true, true)})
    expect(args, {0, 0, 0})
    assert(#queries == 0)
end)

print(string.format("%d Lua regression tests passed; full standard script compiled with %s", passed, _VERSION))
