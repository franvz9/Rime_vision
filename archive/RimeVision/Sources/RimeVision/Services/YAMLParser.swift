import Foundation

final class YAMLParser {
    static let shared = YAMLParser()
    private init() {}

    enum ArrayMode {
        case none
        case dict
        case string
    }

    func parse(_ text: String) -> [String: Any] {
        let lines = text.components(separatedBy: .newlines)
        var root: [String: Any] = [:]
        var stack: [(indent: Int, key: String?, dict: [String: Any])] = []
        var currentDict: [String: Any] = [:]
        var arrayMode: ArrayMode = .none
        var dictArrayItems: [[String: Any]] = []
        var stringArrayItems: [String] = []

        for line in lines {
            let trimmed = line.trimmingCharacters(in: .whitespaces)

            if trimmed.isEmpty || trimmed.hasPrefix("#") {
                continue
            }

            let indent = line.prefix(while: { $0 == " " || $0 == "\t" }).count

            while let last = stack.last, indent <= last.indent {
                let popped = stack.removeLast()
                if let key = popped.key {
                    var parentDict = popped.dict
                    switch arrayMode {
                    case .dict:
                        parentDict[key] = dictArrayItems
                    case .string:
                        parentDict[key] = stringArrayItems
                    case .none:
                        parentDict[key] = currentDict
                    }
                    currentDict = parentDict
                } else {
                    currentDict = popped.dict
                }
                arrayMode = .none
                dictArrayItems = []
                stringArrayItems = []
            }

            if trimmed.hasPrefix("-") {
                if trimmed.hasPrefix("- ") || trimmed == "-" {
                    handleArrayItem(trimmed: trimmed, &arrayMode, &dictArrayItems, &stringArrayItems)
                } else {
                    print("🔍 weird dash line (no space after -): '\(trimmed.prefix(30))' (char2=\(trimmed.count > 1 ? String(trimmed[trimmed.index(trimmed.startIndex, offsetBy: 1)]).unicodeScalars.map { String(format: "U+%04X", $0.value) }.joined(separator: " ") : "none"))")
                }
                continue
            }

            if let colonIndex = trimmed.firstIndex(of: ":") {
                let key = String(trimmed[trimmed.startIndex..<colonIndex]).trimmingCharacters(in: .whitespaces)
                let valuePart = String(trimmed[trimmed.index(after: colonIndex)...]).trimmingCharacters(in: .whitespaces)
                let cleanedValue = stripComment(valuePart)

                if cleanedValue.isEmpty {
                    stack.append((indent: indent, key: key, dict: currentDict))
                    currentDict = [:]
                    arrayMode = .none
                    dictArrayItems = []
                    stringArrayItems = []
                } else {
                    if arrayMode == .dict, !dictArrayItems.isEmpty {
                        var lastItem = dictArrayItems[dictArrayItems.count - 1]
                        lastItem[key] = parseValue(cleanedValue)
                        dictArrayItems[dictArrayItems.count - 1] = lastItem
                    } else {
                        currentDict[key] = parseValue(cleanedValue)
                    }
                }
            }
        }

        while !stack.isEmpty {
            let popped = stack.removeLast()
            if let key = popped.key {
                var parentDict = popped.dict
                switch arrayMode {
                case .dict:
                    parentDict[key] = dictArrayItems
                case .string:
                    parentDict[key] = stringArrayItems
                case .none:
                    parentDict[key] = currentDict
                }
                currentDict = parentDict
            } else {
                currentDict = popped.dict
            }
            arrayMode = .none
            dictArrayItems = []
            stringArrayItems = []
        }

        root = currentDict
        return root
    }

    private func handleArrayItem(
        trimmed: String,
        _ arrayMode: inout ArrayMode,
        _ dictArrayItems: inout [[String: Any]],
        _ stringArrayItems: inout [String]
    ) {
        var valuePart = trimmed == "-" ? "" : String(trimmed.dropFirst(2))

        // Empty item: just mark as dict mode (conservative default)
        if valuePart.isEmpty {
            arrayMode = .dict
            return
        }

        // Strip trailing comment (handles "{ ... } # comment")
        if let hashIndex = valuePart.firstIndex(of: "#") {
            let before = String(valuePart[valuePart.startIndex..<hashIndex])
                .trimmingCharacters(in: .whitespaces)
            if !before.isEmpty {
                valuePart = before
            }
        }

        // Flow mapping: { key: val, ... }
        if valuePart.hasPrefix("{"), let closeBrace = valuePart.lastIndex(of: "}") {
            arrayMode = .dict
            let inner = String(valuePart[valuePart.startIndex...closeBrace])
            let parsed = parseFlowMapping(inner)
            dictArrayItems.append(parsed)
            return
        }

        // Flow sequence: [ item, ... ]
        if valuePart.hasPrefix("["), let closeBracket = valuePart.lastIndex(of: "]") {
            arrayMode = .string
            let inner = String(valuePart[valuePart.startIndex...closeBracket])
            let seqInner = String(inner.dropFirst().dropLast())
            let items = splitFlowSequence(seqInner)
            stringArrayItems.append(contentsOf: items)
            return
        }

        // Inline key: value (or fallback when flow mapping detection fails)
        if let colonIndex = valuePart.firstIndex(of: ":") {
            arrayMode = .dict
            let key = String(valuePart[valuePart.startIndex..<colonIndex]).trimmingCharacters(in: .whitespaces)
            let val = String(valuePart[valuePart.index(after: colonIndex)...]).trimmingCharacters(in: .whitespaces)
            let cleanedVal = stripComment(val)
            dictArrayItems.append([key: parseValue(cleanedVal)])
            if valuePart.hasPrefix("{") {
                print("⚠️ YAML: flow mapping fell through to inline key:val, key='\(key)' valuePart=\(valuePart.prefix(60))")
            }
            return
        }

        // Plain scalar
        arrayMode = .string
        let cleanedVal = stripComment(valuePart)
        stringArrayItems.append(cleanedVal)
    }

    // MARK: - Flow style parsers

    private func parseFlowMapping(_ str: String) -> [String: Any] {
        var result: [String: Any] = [:]
        let inner = String(str.dropFirst().dropLast()) // strip { }
        let pairs = splitFlowMappingPairs(inner)
        for pair in pairs {
            guard let colonIndex = pair.firstIndex(of: ":") else { continue }
            let key = String(pair[pair.startIndex..<colonIndex]).trimmingCharacters(in: .whitespaces)
            let value = String(pair[pair.index(after: colonIndex)...]).trimmingCharacters(in: .whitespaces)
            result[key] = parseValue(value)
        }
        return result
    }

    private func splitFlowMappingPairs(_ str: String) -> [String] {
        // Split on commas, respecting nested braces and brackets
        var pairs: [String] = []
        var depth = 0
        var current = ""
        for char in str {
            if char == "{" || char == "[" { depth += 1 }
            else if char == "}" || char == "]" { depth -= 1 }
            else if char == "," && depth == 0 {
                pairs.append(current)
                current = ""
                continue
            }
            current.append(char)
        }
        if !current.isEmpty {
            pairs.append(current)
        }
        return pairs
    }

    private func splitFlowSequence(_ str: String) -> [String] {
        // Split on commas, respecting nested braces/brackets
        var items: [String] = []
        var depth = 0
        var current = ""
        for char in str {
            if char == "{" || char == "[" { depth += 1 }
            else if char == "}" || char == "]" { depth -= 1 }
            else if char == "," && depth == 0 {
                let trimmed = current.trimmingCharacters(in: .whitespaces)
                if !trimmed.isEmpty { items.append(trimmed) }
                current = ""
                continue
            }
            current.append(char)
        }
        let trimmed = current.trimmingCharacters(in: .whitespaces)
        if !trimmed.isEmpty { items.append(trimmed) }
        return items
    }

    // MARK: - Helpers

    private func stripComment(_ value: String) -> String {
        let trimmed = value.trimmingCharacters(in: .whitespaces)
        if trimmed.hasPrefix("'") || trimmed.hasPrefix("\"") {
            return trimmed
        }
        if let hashIndex = trimmed.firstIndex(of: "#") {
            let beforeHash = String(trimmed[trimmed.startIndex..<hashIndex]).trimmingCharacters(in: .whitespaces)
            if !beforeHash.isEmpty {
                return beforeHash
            }
        }
        return value
    }

    private func parseValue(_ str: String) -> Any {
        let trimmed = str.trimmingCharacters(in: .whitespaces)

        // Quoted string
        if trimmed.count >= 2 {
            let first = trimmed.first!
            let last = trimmed.last!
            if ((first == "'" && last == "'") || (first == "\"" && last == "\"")) && trimmed.count > 2 {
                return String(trimmed.dropFirst().dropLast())
            }
        }

        // Flow mapping (inline dict)
        if trimmed.hasPrefix("{") && trimmed.hasSuffix("}") {
            return parseFlowMapping(trimmed)
        }

        // Booleans
        if trimmed == "true" { return true }
        if trimmed == "false" { return false }

        // Number
        if let v = Double(trimmed) { return v }

        return trimmed
    }
}
