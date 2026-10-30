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

local function production_function(name, context)
    local start = assert(source:find("local " .. name .. " = function(", 1, true))
    local finish = assert(source:find("\nend\n", start, true))
    return assert(load(source:sub(start, finish + 3) .. "\nreturn " .. name,
        "@production-" .. name, "t", context))()
end

local function preview(panel, alt, form, random, custom, fixed_form)
    return {
        panel_id_ = panel, selected_alt_ = alt, form_type_ = form,
        random = random, is_sub_stage_ = custom or false, fixed_form = fixed_form
    }
end

local function exercise(previews, canceled, before_capture)
    local finalized, stopped, received = false, false, nil
    local queries, logs, form_queries = {}, {}, {}
    local context = setmetatable({
        stage_previews = previews, USE_STAGE_NUM = #previews,
        UI_INVALID_INDEX = -1, current_selected_preview = 0,
        set_tab_form_text = function(_) end,
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
            invoke = function(name, index, value)
                if name == "set_stage_form_type_stage_preview" then
                    assert(not finalized)
                    previews[index + 1].game_applied_form = value
                    return
                end
                if name == "set_stage_preview_from_sub_panel" then return end
                if name == "is_random_stage_preview" then
                    assert(not finalized, "Read the preview before it is finalized")
                    queries[#queries + 1] = index
                    return assert(previews[index + 1]).random
                end
                if name == "is_fixed_form_type_stage_preview" or name == "get_stage_fixed_form_type" then
                    assert(not finalized, "Read form diagnostics before finalization")
                    local item = assert(previews[index + 1])
                    assert(not item.is_sub_stage_, "Do not query standard forms for custom stages")
                    form_queries[#form_queries + 1] = name
                    if name == "is_fixed_form_type_stage_preview" then
                        return item.fixed_form ~= nil
                    end
                    assert(item.fixed_form ~= nil, "Only query the override for a fixed-form preview")
                    return item.fixed_form
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
    if before_capture then before_capture(context) end
    context.capture_alt_selection = assert(load(helper, "@production-selection-helper", "t", context))()
    assert(load(dispatch, "@production-selection-dispatch", "t", context))()()
    assert(finalized and stopped)
    return received, queries, logs, form_queries
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
    local args, queries, logs, form_queries = exercise({preview(0, 2, 0, true)}, true)
    assert(args == nil and #queries == 0 and #logs == 0 and #form_queries == 0)
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
    local args, queries, _, form_queries = exercise({preview(0, 0, 0, true, true)})
    expect(args, {0, 0, 0})
    assert(#queries == 0 and #form_queries == 0)
end)

test("Form diagnostics do not query an override for an ordinary preview", function()
    local args, _, logs, queries = exercise({preview(-1, 0, 2, true)})
    expect(args, {0, -1, 2})
    assert(#queries == 1 and queries[1] == "is_fixed_form_type_stage_preview")
    assert(logs[2]:find("cached_form=2 is_fixed=false fixed_form=-1 sent_form=2", 1, true))
end)
test("Without a tracked application the final query alone does not change Random", function()
    local args, _, logs, queries = exercise({preview(-1, 0, 0, true, false, 2)})
    expect(args, {0, -1, 0})
    assert(#queries == 2 and queries[2] == "get_stage_fixed_form_type")
    assert(logs[2]:find("cached_form=0 is_fixed=true fixed_form=2 sent_form=0", 1, true))
end)
test("Fixed form zero is logged without changing a manual alt", function()
    local args, _, logs = exercise({preview(3, 1, 2, false, false, 0)})
    expect(args, {1, 3, 2})
    assert(logs[2]:find("cached_form=2 is_fixed=true fixed_form=0 sent_form=2", 1, true))
end)
test("Override logging preserves the setter's existing fixed and ordinary behavior", function()
    local start = assert(source:find("local set_stage_preview_form = function(preview_index, stage_form)", 1, true))
    local finish = assert(source:find("\nend\n", start, true))
    local setter_source = source:sub(start, finish + 3) .. "\nreturn set_stage_preview_form"
    local fixed, applied, displayed = true, nil, nil
    local logs = {}
    local item = {form_type_ = 0}
    local context = setmetatable({
        UI_INVALID_INDEX = -1, current_selected_preview = 0,
        stage_previews = {item},
        set_tab_form_text = function(form) displayed = form end,
        Alts = {send_message = function(message) logs[#logs + 1] = message end},
        UiScriptPlayer = {invoke = function(name, index, form)
            assert(index == 0, "An invalid preview must not query the game")
            if name == "is_fixed_form_type_stage_preview" then return fixed end
            if name == "get_stage_fixed_form_type" then assert(fixed); return 2 end
            assert(name == "set_stage_form_type_stage_preview")
            applied = form
        end}
    }, {__index = _G})
    local setter = assert(load(setter_source, "@production-form-setter", "t", context))()
    setter(0, 1)
    assert(applied == 2 and displayed == 2 and item.form_type_ == 0 and item.applied_form_ == 2)
    assert(logs[1]:find("cached_form=0 requested_form=1 applied_form=2", 1, true))
    fixed = false
    setter(0, 1)
    assert(applied == 1 and displayed == 1 and item.form_type_ == 1 and item.applied_form_ == 1 and #logs == 1)
    setter(-1, 2)
    assert(applied == 1 and item.form_type_ == 1 and #logs == 1)
end)

test("Random retains applied Omega when the confirmation query reverts to Normal", function()
    local item = preview(-1, 0, 0, true, false, 2)
    local args, _, logs = exercise({item}, false, function(context)
        production_function("set_stage_preview_form", context)(0, 0)
        assert(item.game_applied_form == 2 and item.form_type_ == 0)
        item.fixed_form = 0
    end)
    expect(args, {0, -1, 2})
    assert(logs[#logs]:find("fixed_form=0 sent_form=2 applied_form=2 form_source=applied tracking=v1", 1, true))
end)
test("Returning to Normal replaces a previously applied Omega", function()
    local item = preview(-1, 0, 0, true, false, 2)
    local args = exercise({item}, false, function(context)
        local setter = production_function("set_stage_preview_form", context)
        setter(0, 0)
        item.fixed_form = 0
        setter(0, 0)
        assert(item.game_applied_form == 0 and item.applied_form_ == 0)
    end)
    expect(args, {0, -1, 0})
end)
test("Ordinary form changes replace a prior fixed override", function()
    local item = preview(-1, 0, 0, true, false, 2)
    local args = exercise({item}, false, function(context)
        local setter = production_function("set_stage_preview_form", context)
        setter(0, 0)
        item.fixed_form = nil
        setter(0, 1)
        assert(item.game_applied_form == 1 and item.applied_form_ == 1)
    end)
    expect(args, {0, -1, 1})
end)
test("A fixed override does not reindex an explicitly selected manual alt", function()
    local item = preview(3, 1, 0, false, false, 2)
    local args = exercise({item}, false, function(context)
        production_function("set_stage_preview_form", context)(0, 0)
    end)
    expect(args, {1, 3, 0})
end)
test("Tracked forms remain independent across previews", function()
    local a, b = preview(-1, 0, 0, true, false, 2), preview(-1, 0, 0, true, false, 1)
    local args = exercise({a, b}, false, function(context)
        local setter = production_function("set_stage_preview_form", context)
        setter(0, 0)
        setter(1, 0)
        a.fixed_form, b.fixed_form = 0, 0
    end)
    expect(args, {0, -1, 2, 0, -1, 1})
end)
test("Disabling a preview clears the tracked form before reuse", function()
    local item = preview(-1, 0, 0, true, false, 2)
    local args = exercise({item}, false, function(context)
        production_function("set_stage_preview_form", context)(0, 0)
        production_function("disable_stage_preview", context)(0)
        assert(item.applied_form_ == nil)
        item.fixed_form = 0
    end)
    expect(args, {0, -1, 0})
end)
test("Entering a custom stage clears the tracked standard form", function()
    local item = preview(-1, 0, 0, true, false, 2)
    local args = exercise({item}, false, function(context)
        production_function("set_stage_preview_form", context)(0, 0)
        production_function("set_stage_preview_from_sub_stage_panel", context)(0, 5)
        assert(item.applied_form_ == nil)
        item.panel_id_, item.is_sub_stage_ = 5, true
    end)
    expect(args, {0, 5, 0})
end)
test("Invalid tracked forms fall back to the cached selection", function()
    local item = preview(-1, 0, 1, true)
    item.applied_form_ = 99
    expect(exercise({item}), {0, -1, 1})
end)

print(string.format("%d Lua regression tests passed; full standard script compiled with %s", passed, _VERSION))
