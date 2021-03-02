local main = methatron.scene.new("main")
main:create_shader(
  "phong",
  "assets/shaders/phong.vertex.glsl",
  "assets/shaders/phong.fragment.glsl"
)
main:create_model(
  "iko",
  "assets/models/ikosaeder.json"
)
main:create_drawable("user", "phong", "iko")

local cam = main:get_camera()
local mat = cam:get_node():get_transform()
local cam_distance = methatron.math.vector.new(0.0, 0.0, 10.0)

print("set scene")
context:set_scene(main)
local network = context:network()
local ub = nil

on_connect = function()
  network:join("main")
  n0 = network:spawn("main", "user", "assets/scripts/user.remote.lua")

  local user = require("assets/scripts/user")
  ub = user.new(n0)

  print(n0:id() .. " " .. n0:network_id())
end

on_disconnect = function()
  print("disconnect")
end

print("prepare update")
on_update = function()
  if ub then
    ub:on_update()
  end

  local pos = context:mouse_position()
  mat:identity()
  mat:rotate_y(pos[1] / 300)
  mat:rotate_x(pos[2] / 200)
  mat:translate(cam_distance)
end

on_draw = function()
end