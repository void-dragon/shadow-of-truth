local main = methatron.scene.new("main")
main:create_model(
  "cube",
  "assets/models/bunny-ball.json"
)
main:create_model(
  "terrain",
  "assets/models/test-map.json"
)
main:create_drawable("user", "cube")
main:create_drawable("terrain", "terrain")

local node_terrain = methatron.node.new()
node_terrain:set_drawable(main:get_drawable("terrain"))
main:get_root():add_child(node_terrain)

local cam = main:get_camera()
local mat = cam:get_node():get_transform()
local cam_distance = {0.0, 0.0, 10.0}

local light = main:get_lights()[1]
local l_node = light:get_node()
local l_mat = l_node:get_transform()
l_mat:translate({5, 19, 0})

lua.print("set scene")
engine:set_scene(main)
local network = engine:network()
local ub = nil
local user = require("assets/scripts/user")

on_connect = function()
  network:join("main")
  n0 = network:spawn("main", "user", nil)

  ub = user.new(n0)

  lua.print(n0:id() .. " " .. n0:network_id())
end

on_disconnect = function()
  lua.print("disconnect")
end

on_key_press = function(key)
  -- print("press " .. key)
  if ub then
    ub:on_key_press(key)
  end
end

on_key_release = function(key)
  -- print("release " .. key)
end

on_mouse_wheel = function(pos)
  cam_distance[3] = cam_distance[3] + pos
end

on_update = function()
  local offset = nil

  if ub then
    ub:on_update()
    offset = ub.node:get_transform():position()
  end

  local pos = engine:mouse_position()
  mat:batch(function(m)
    m:identity()
    if offset then
      m:translate(offset)
    end
    m:rotate_y(pos[1] / 300)
    m:rotate_x(pos[2] / 200)
    m:translate(cam_distance)
  end)

  if engine:is_key_down("B") then
    l_mat:translate({0.0, 1.0, 0.0})
  end
  if engine:is_key_down("N") then
    l_mat:translate({0.0, -1.0, 0.0})
  end
end