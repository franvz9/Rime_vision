import Foundation
import Yams

public enum RimeConfigStore {
    public static func parseYAML(_ text: String) throws -> [String: Any] {
        guard let loaded = try Yams.load(yaml: text) else { return [:] }
        return normalize(loaded) as? [String: Any] ?? [:]
    }

    public static func dumpYAML(_ object: [String: Any]) throws -> String {
        try Yams.dump(object: normalizeForDump(object), sortKeys: true)
    }

    public static func loadYAML(from url: URL) throws -> [String: Any] {
        guard FileManager.default.fileExists(atPath: url.path) else { return [:] }
        let content = try String(contentsOf: url, encoding: .utf8)
        return try parseYAML(content)
    }

    public static func loadEffective(baseURL: URL, customURL: URL) throws -> [String: Any] {
        let base = try loadYAML(from: baseURL)
        let custom: [String: Any]
        do {
            custom = try loadYAML(from: customURL)
        } catch {
            print("⚠️ RimeVision: failed to parse \(customURL.lastPathComponent), using base config only: \(error)")
            return base
        }
        guard let patch = custom["patch"] as? [String: Any] else { return base }
        return RimePatch.merge(base: base, patch: patch)
    }

    @discardableResult
    public static func savePatch(customURL: URL, mutate: (inout [String: Any]) throws -> Void) throws -> ConfigBackup.WriteResult {
        let existing = try loadYAML(from: customURL)
        var patch = existing["patch"] as? [String: Any] ?? [:]
        patch = RimePatch.expandedPatch(patch)
        try mutate(&patch)
        let content = try dumpYAML(["patch": patch])
        return try ConfigBackup.writeIfChanged(content, to: customURL)
    }

    private static func normalize(_ value: Any) -> Any {
        if let dict = value as? [AnyHashable: Any] {
            var normalized: [String: Any] = [:]
            for (key, value) in dict {
                normalized[String(describing: key)] = normalize(value)
            }
            return normalized
        }
        if let dict = value as? [String: Any] {
            return dict.mapValues(normalize)
        }
        if let array = value as? [Any] {
            return array.map(normalize)
        }
        if let number = value as? NSNumber {
            if CFGetTypeID(number) == CFBooleanGetTypeID() { return number.boolValue }
            let double = number.doubleValue
            if double == Double(number.intValue) { return number.intValue }
            return double
        }
        return value
    }

    private static func normalizeForDump(_ value: Any) -> Any {
        if let dict = value as? [String: Any] {
            return dict.mapValues(normalizeForDump)
        }
        if let array = value as? [Any] {
            return array.map(normalizeForDump)
        }
        return value
    }
}
