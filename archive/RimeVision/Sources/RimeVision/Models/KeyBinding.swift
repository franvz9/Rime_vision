import Foundation

struct KeyBinding: Identifiable, Equatable {
    var id = UUID()
    var when: String
    var accept: String
    var send: String
    var toggle: String
    var select: String

    init(when: String = "always", accept: String = "", send: String = "", toggle: String = "", select: String = "") {
        self.when = when
        self.accept = accept
        self.send = send
        self.toggle = toggle
        self.select = select
    }

    static func == (lhs: KeyBinding, rhs: KeyBinding) -> Bool {
        lhs.when == rhs.when && lhs.accept == rhs.accept && lhs.send == rhs.send && lhs.toggle == rhs.toggle && lhs.select == rhs.select
    }
}
