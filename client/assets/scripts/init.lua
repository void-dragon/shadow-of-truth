local main = methatron.scene.new("main")
main:create_model(
  "cube",
  "assets/models/cube.json"
)
main:create_model(
  "bunny",
  "assets/models/bunny-ball.json"
)
main:create_model(
  "terrain",
  "assets/models/test-map.json"
)
main:create_drawable("cube", "cube")
main:create_drawable("bunny", "bunny")
main:create_drawable("terrain", "terrain")

local root = main:get_root()

local node_terrain = methatron.node.new()
node_terrain:set_drawable(main:get_drawable("terrain"))
root:add_child(node_terrain)

local node_target = methatron.node.new()
local node_inner = methatron.node.new()
node_inner:set_drawable(main:get_drawable("cube"))
node_inner:get_transform():scale(0.3)
local material = node_inner:get_material()
material:set_ambient({0.9, 0.2, 0.4})
material:set_diffuse({0.9, 0.2, 0.4})
node_target:add_child(node_inner)
node_target:get_transform():translate({0.0, 1.0, 0.0})
root:add_child(node_target)

local cam = main:get_camera()

local orb_behavior = require("assets/scripts/orbit")
local orb = orb_behavior.new(cam, node_target)

local light = main:get_lights()[1]
light:set_target(node_target)
local l_node = light:get_node()
local node_inner = methatron.node.new()
node_inner:set_drawable(main:get_drawable("cube"))
node_inner:get_transform():scale(0.3)
local material = node_inner:get_material()
material:set_ambient({0.2, 0.9, 0.4})
material:set_diffuse({0.2, 0.9, 0.4})
l_node:add_child(node_inner)
local l_mat = l_node:get_transform()
local alpha = 0
l_mat:translate({0, 10, 5})

engine:set_scene(main)
local network = engine:network()
local ub = nil
local bunny = nil
local user = require("assets/scripts/user")
local font = methatron.d2.font.new("assets/fonts/UbuntuMono-Regular.ttf")
-- lua.print("yeha")
-- font.draw(100, 100, "test string")

on_connect = function()
  network:join("main")
  bunny = network:spawn("main", "bunny", nil)

  bunny:get_transform():translate({0.0, 1.0, 3.0})
  ub = user.new(node_target)
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
  orb:on_mouse_wheel(pos)
end

on_update = function()
  if ub then
    ub:on_update()
    bunny:get_transform():look_at(node_target:get_transform())
  end

  if font then
    -- do something
  end

  -- l_mat:translate({math.sin(alpha), 0, math.cos(alpha)})
  -- alpha = alpha + 0.05

  if engine:is_key_down("Y") then
    l_mat:translate({0,0,1})
  elseif engine:is_key_down("X") then
    l_mat:translate({0,0,-1})
  end
  orb:on_update()
end