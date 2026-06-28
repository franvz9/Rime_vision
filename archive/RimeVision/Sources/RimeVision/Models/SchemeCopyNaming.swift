import Foundation

public enum SchemeCopyNaming {
    public static func uniqueCopyName(for name: String, existingNames: Set<String>) -> String {
        let base = "\(name)_copy"
        if !existingNames.contains(base) { return base }

        var index = 2
        while existingNames.contains("\(base)_\(index)") {
            index += 1
        }
        return "\(base)_\(index)"
    }

    public static func uniqueCopyName<S: Sequence>(for name: String, existingNames: S) -> String where S.Element == String {
        uniqueCopyName(for: name, existingNames: Set(existingNames))
    }
}
