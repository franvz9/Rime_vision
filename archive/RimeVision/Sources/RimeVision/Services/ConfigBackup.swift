import Foundation

public enum ConfigBackup {
    public enum WriteResult: Equatable {
        case unchanged
        case written
    }

    @discardableResult
    public static func writeIfChanged(_ content: String, to url: URL) throws -> WriteResult {
        if FileManager.default.fileExists(atPath: url.path) {
            let existing = try String(contentsOf: url, encoding: .utf8)
            if existing == content {
                return .unchanged
            }
            let backupURL = timestampedBackupURL(for: url)
            try FileManager.default.copyItem(at: url, to: backupURL)
        } else {
            let directory = url.deletingLastPathComponent()
            try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
        }

        try content.write(to: url, atomically: true, encoding: .utf8)
        return .written
    }

    private static func timestampedBackupURL(for url: URL) -> URL {
        let formatter = DateFormatter()
        formatter.dateFormat = "yyyyMMdd-HHmmss-SSS"
        let timestamp = formatter.string(from: Date())
        return url.deletingLastPathComponent()
            .appendingPathComponent("\(url.lastPathComponent).\(timestamp).bak")
    }
}
