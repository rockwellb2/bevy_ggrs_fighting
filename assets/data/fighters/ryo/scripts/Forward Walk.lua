function init(player)

end


function process(player)
    --
end


function enter(player)
    -- local Velocity = world:get_type_by_name("Velocity")
    -- local velo = world:get_component(player, Velocity)[0]
    -- velo.x = 3.0
    -- velo.y = 0
    -- velo.z = 0

    local Velocity = world:get_type_by_name("Velocity")
    local velo = world:get_component(player, Velocity)
    print("Before")
    print(velo.x)
    velo:set(3, 0, 0)
    print("After")
    print(velo.x)



end


function exit(player)
    -- local Velocity = world:get_type_by_name("Velocity")
    -- local velo = world:get_component(player, Velocity)[0]
    -- velo.x = 0
    -- velo.y = 0;
    -- velo.z = 0;

    local Velocity = world:get_type_by_name("Velocity")
    local velo = world:get_component(player, Velocity)
    print("Before")
    print(velo.x)
    velo:set(0, 0, 0)
    print("After")
    print(velo.x)

end
