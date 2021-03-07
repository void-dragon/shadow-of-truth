
local m = node:get_transform()
local border = 50

on_update = function()
    if is_owner then
        m:rotate_y(0.03)
        border = border - 1
        if border == 0 then
            engine:network():destroy("main", node:network_id())
            lua.exit()
        end
    end
end