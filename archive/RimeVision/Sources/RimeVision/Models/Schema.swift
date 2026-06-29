import Foundation

struct RimeSchema: Identifiable, Equatable, Hashable {
    var id: String { schemaId }

    var schemaId: String
    var name: String
    var version: String
    var enabled: Bool
    var dependencies: [String]

    init(schemaId: String, name: String = "", version: String = "", enabled: Bool = true, dependencies: [String] = []) {
        self.schemaId = schemaId
        self.name = name
        self.version = version
        self.enabled = enabled
        self.dependencies = dependencies
    }

    static func == (lhs: RimeSchema, rhs: RimeSchema) -> Bool {
        lhs.schemaId == rhs.schemaId
    }

    func hash(into hasher: inout Hasher) {
        hasher.combine(schemaId)
    }
}
