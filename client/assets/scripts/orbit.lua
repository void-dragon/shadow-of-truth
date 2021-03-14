return {
    new = function(cam, node)
        local target = node:get_transform()
        local mat = cam:get_node():get_transform()
        local cam_distance = {0.0, 0.0, 20.0}

        return {
            on_update = function()
                local pos = engine:mouse_position()
                mat:batch(function(m)
                    m:identity()
                    m:translate(target:position())
                    m:rotate_y(pos[1] / 300)
                    m:rotate_x(pos[2] / 200)
                    m:translate(cam_distance)
                end)
            end,
            on_mouse_wheel = function(this, pos)
                cam_distance[3] = cam_distance[3] + pos
            end
        }
    end
}