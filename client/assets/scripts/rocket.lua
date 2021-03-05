
local m = node:get_transform()
local border = 50

on_update = function()
    if is_owner then
        m:rotate_y(0.03)
        border = border - 1
        if border == 0 then
            context:network():destroy("main", node:network_id())
            exit()
        end
    end
end