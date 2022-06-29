import "fighter" for State, Vec3
import "meta" for Meta

class S1 is State {
    static process_state(position) {
        System.print(Meta.getModuleVariables("this"))
        System.print("This is the child class")
    }
}