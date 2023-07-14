function help(player)
    print("This is a Lua function")

    local Transform = world:get_type_by_name("Transform")
    local tf = world:get_component(player, Transform)
    print(tf.translation.x)

 end

function process()
    print("Process")
end

local function init()
end
