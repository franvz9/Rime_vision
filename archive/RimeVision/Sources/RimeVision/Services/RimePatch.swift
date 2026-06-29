import Foundation

public enum RimePatch {
    public static func splitPath(_ key: String) -> [String] {
        key.split(separator: "/").map(String.init).filter { !$0.isEmpty }
    }

    public static func expandedPatch(_ patch: [String: Any]) -> [String: Any] {
        var result: [String: Any] = [:]
        for (key, value) in patch where key != "__delete__" && key != "__append__" {
            let expandedValue: Any
            if let dict = value as? [String: Any] {
                expandedValue = expandedPatch(dict)
            } else {
                expandedValue = value
            }
            setValue(expandedValue, in: &result, path: splitPath(key))
        }
        if let deletions = patch["__delete__"] { result["__delete__"] = deletions }
        if let appends = patch["__append__"] { result["__append__"] = appends }
        return result
    }

    public static func merge(base: [String: Any], patch: [String: Any]) -> [String: Any] {
        var result = base
        let expanded = expandedPatch(patch)

        if let deletions = expanded["__delete__"] as? [String] {
            for key in deletions {
                removeValue(in: &result, path: splitPath(key))
            }
        }

        if let appends = expanded["__append__"] as? [String: Any] {
            for (key, value) in appends {
                let path = splitPath(key)
                let newItems = value as? [Any] ?? [value]
                if var existing = Self.value(in: result, path: path) as? [Any] {
                    existing.append(contentsOf: newItems)
                    setValue(existing, in: &result, path: path)
                } else {
                    setValue(newItems, in: &result, path: path)
                }
            }
        }

        for (key, value) in expanded where key != "__delete__" && key != "__append__" {
            mergeValue(value, into: &result, path: [key])
        }
        return result
    }

    public static func value(in dict: [String: Any], path: [String]) -> Any? {
        guard let first = path.first else { return dict }
        guard let current = dict[first] else { return nil }
        if path.count == 1 { return current }
        guard let nested = current as? [String: Any] else { return nil }
        return value(in: nested, path: Array(path.dropFirst()))
    }

    public static func setValue(_ value: Any, in dict: inout [String: Any], path: [String]) {
        guard let first = path.first else { return }
        if path.count == 1 {
            dict[first] = value
            return
        }
        var nested = dict[first] as? [String: Any] ?? [:]
        setValue(value, in: &nested, path: Array(path.dropFirst()))
        dict[first] = nested
    }

    public static func removeValue(in dict: inout [String: Any], path: [String]) {
        guard let first = path.first else { return }
        if path.count == 1 {
            dict.removeValue(forKey: first)
            return
        }
        guard var nested = dict[first] as? [String: Any] else { return }
        removeValue(in: &nested, path: Array(path.dropFirst()))
        dict[first] = nested
    }

    private static func mergeValue(_ value: Any, into dict: inout [String: Any], path: [String]) {
        guard let first = path.first else { return }
        if path.count > 1 {
            var nested = dict[first] as? [String: Any] ?? [:]
            mergeValue(value, into: &nested, path: Array(path.dropFirst()))
            dict[first] = nested
            return
        }

        if let patchDict = value as? [String: Any],
           let baseDict = dict[first] as? [String: Any] {
            dict[first] = merge(base: baseDict, patch: patchDict)
        } else {
            dict[first] = value
        }
    }
}
