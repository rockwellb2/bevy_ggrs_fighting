function init(player)

end


function help(player)
    print("This is a Lua function")

    local Transform = world:get_type_by_name("Transform")
    local tf = world:get_component(player, Transform)
    print(tf.translation.x)

 end

function process(player)
end


function enter(player)
    -- local Velocity = world:get_type_by_name("Velocity")
    -- local velo = world:get_component(player, Velocity)[0]
    -- velo.x = 0
    -- velo.y = 0;
    -- velo.z = 0;
end


function exit(player)
end
