return {
    new = function(node)
        local mat = node:get_transform()
        local network = context:network()

        return {
            on_key_press = function(self, key)
                if key == "Q" then
                    network:spawn("main", "rocket", "assets/scripts/rocket.lua")
                end
            end,
            on_update = function()
                if context:is_key_down("A") then
                    mat:translate(methatron.math.vector.new(0.1, 0.0, 0.0))
                elseif context:is_key_down("D") then
                    mat:translate(methatron.math.vector.new(-0.1, 0.0, 0.0))
                end
            end
        }
    end
}