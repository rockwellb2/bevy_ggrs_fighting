import "fighter" for State, Vec3, Helper

class S100 is State {
    construct new() {
    }

    

    static processState(position, input) {
        //import "fighter" for left

        //System.print("The input is " + input.toString)

        //var vector = Vec3.new(5, 4, 3)
        //var new_vector = Helper.fighterMovement(vector, left)

        //System.print("After moving left, the value of x is " + new_vector.x.toString)
        Helper.fighterMovement(position, input)
        System.print("The vector's x is " + position.x.toString)
    }
}