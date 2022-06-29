var lp = 1 << 0
var mp = 1 << 1
var hp = 1 << 2
var lk = 1 << 3
var mk = 1 << 4
var hk = 1 << 5

var left = 1 << 6



class Fighter {

}

class Input {}

class State {
    static processState(position, input) {
        System.print("This is the base State")
    }
}


foreign class Vec3 {
    construct new(x, y, z) {}

    foreign x=(x)
    foreign y=(y)
    foreign z=(z)

    foreign x
    foreign y
    foreign z
}

class Helper {
    foreign static fighterMovement(position, input)
}