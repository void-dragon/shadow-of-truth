local main = methatron.scene.new("main")
main:create_shader(
  "phong",
  "assets/shaders/phong.vertex.glsl",
  "assets/shaders/phong.fragment.glsl"
)
main:create_model(
  "cube",
  "assets/models/cube.json"
)
main:create_model(
  "iko",
  "assets/models/ikosaeder.json"
)
main:create_drawable("user", "phong", "cube")
main:create_drawable("rocket", "phong", "iko")

local cam = main:get_camera()
local mat = cam:get_node():get_transform()
local cam_distance = methatron.math.vector.new(0.0, 0.0, 10.0)

local light = main:get_lights()[1]
local l_node = light:get_node()
local l_mat = l_node:get_transform()
l_mat:translate(methatron.math.vector.new(0, 2, 0))

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

on_update = function()
  if ub then
    ub:on_update()
  end

  local pos = engine:mouse_position()
  mat:batch(function(m)
    m:identity()
    m:rotate_y(pos[1] / 300)
    m:rotate_x(pos[2] / 200)
    m:translate(cam_distance)
  end)
end