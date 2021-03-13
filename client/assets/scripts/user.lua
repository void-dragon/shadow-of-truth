return {
    new = function(node)
        local mat = node:get_transform()
        local network = engine:network()

        return {
            node = node,
            on_key_press = function(self, key)
                if key == "Q" then
                    network:spawn("main", "rocket", "assets/scripts/rocket.lua")
                end
            end,
            on_update = function()
                if engine:is_key_down("W") then
                    mat:translate({-0.1, 0.0, 0.0})
                end
                if engine:is_key_down("S") then
                    mat:translate({0.1, 0.0, 0.0})
                end
                if engine:is_key_down("A") then
                    mat:rotate_y(0.05)
                end
                if engine:is_key_down("D") then
                    mat:rotate_y(-0.05)
                end
            end
        }
    end
}