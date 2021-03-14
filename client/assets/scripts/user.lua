return {
    new = function(node)
        local mat = node:get_transform()
        local network = engine:network()
        local speed = 0.3

        return {
            node = node,
            on_key_press = function(self, key)
                if key == "Q" then
                    network:spawn("main", "cube", "assets/scripts/rocket.lua")
                end
            end,
            on_update = function()
                local step = {0, 0, 0}
                if engine:is_key_down("W") then
                    step[3] = -1
                end
                if engine:is_key_down("S") then
                    step[3] = 1
                end
                if engine:is_key_down("A") then
                    step[1] = 1
                end
                if engine:is_key_down("D") then
                    step[1] = -1
                end
                local n = math.sqrt(step[1] * step[1] + step[2] * step[2] + step[3] * step[3])

                if n > 0 then
                    for i=1, 3 do
                        step[i] = speed * step[i] / n
                    end
                    mat:translate(step)
                end
            end
        }
    end
}