local main = methatron.scene.new()
local drw = methatron.drawable.quick_load(
  "assets/shaders/phong.vertex.glsl",
  "assets/shaders/phong.fragment.glsl",
  "assets/models/ikosaeder.json"
)

print("set drawable to node")
local n0 = methatron.node.new()
n0:set_drawable(drw)

print("main get root")
local root = main:get_root()
root:add_child(n0)

local cam = main:get_camera()
local mat = cam:get_node():get_transform()
local step = methatron.math.vector.new(0.0, 0.0, 10.0)
methatron.math.matrix.translate(mat, step)

print("add drawable")
main:add_drawable(drw)

print("set scene")
context:set_scene(main)

print("transform")
local rot_x = methatron.math.matrix.rotate_x
local rot_y = methatron.math.matrix.rotate_y
local n0_mat = n0:get_transform()
local old_pos = context:mouse_position()

print("prepare update")
on_update = function()
  if context:is_key_down("A") then
    return rot_x(n0_mat, 0.1)
  elseif context:is_key_down("D") then
    return rot_x(n0_mat, -0.1)
  end

  local pos = context:mouse_position()
  rot_y(n0_mat, (old_pos[1] - pos[1]) / 100)
  old_pos = pos
end
