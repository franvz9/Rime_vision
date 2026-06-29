import Foundation

@MainActor
public final class ConfigManager: ObservableObject {
    public static let shared = ConfigManager()

    @Published var squirrelStyle: RimeStyle = RimeStyle()
    @Published var colorSchemes: [String: RimeColorScheme] = [:]
    @Published var darkColorSchemes: [String: RimeColorScheme] = [:]
    @Published var schemas: [RimeSchema] = []
    @Published var customSchemas: [RimeSchema] = []
    @Published public var keyBindings: [KeyBindingItem] = []
    @Published public var pageSize: Int = 6

    public struct KeyBindingItem: Identifiable, Equatable {
        public var id = UUID()
        public var when: String
        public var accept: String
        public var send: String
        public var toggle: String
        public var select: String

        public init(id: UUID = UUID(), when: String, accept: String, send: String, toggle: String, select: String) {
            self.id = id
            self.when = when
            self.accept = accept
            self.send = send
            self.toggle = toggle
            self.select = select
        }

        public var actionType: String {
            if !toggle.isEmpty { return "toggle" }
            if !select.isEmpty { return "select" }
            return "send"
        }

        public var actionValue: String {
            if !toggle.isEmpty { return toggle }
            if !select.isEmpty { return select }
            return send
        }

        public static func == (lhs: KeyBindingItem, rhs: KeyBindingItem) -> Bool {
            lhs.when == rhs.when && lhs.accept == rhs.accept
        }
    }

    public struct PunctRule: Identifiable {
        public var id = UUID()
        public var key: String
        public var commit: String
        public var pair: [String]
        public var list: [String]

        public init(id: UUID = UUID(), key: String, commit: String, pair: [String], list: [String]) {
            self.id = id
            self.key = key
            self.commit = commit
            self.pair = pair
            self.list = list
        }
    }

    @Published public var halfShapePunct: [PunctRule] = []
    @Published public var fullShapePunct: [PunctRule] = []

    public let rimeUserDir: URL

    var squirrelYAML: URL { rimeUserDir.appendingPathComponent("squirrel.yaml") }
    var squirrelCustomYAML: URL { rimeUserDir.appendingPathComponent("squirrel.custom.yaml") }
    var defaultYAML: URL { rimeUserDir.appendingPathComponent("default.yaml") }
    var defaultCustomYAML: URL { rimeUserDir.appendingPathComponent("default.custom.yaml") }

    public init(rimeUserDir: URL = FileManager.default.homeDirectoryForCurrentUser.appendingPathComponent("Library/Rime")) {
        self.rimeUserDir = rimeUserDir
    }

    func loadAll() {
        loadSquirrelConfig()
        loadDefaultConfig()
        loadKeyBindings()
        loadPunctuation()
    }

    func loadSquirrelConfig() {
        let baseText = loadFile(squirrelYAML)
        let customText = loadFile(squirrelCustomYAML)

        let baseDict = baseText.flatMap { YAMLParser.shared.parse($0) } ?? [:]
        let customDict = customText.flatMap { YAMLParser.shared.parse($0) } ?? [:]

        let merged = applyPatch(base: baseDict, custom: customDict)
        parseSquirrelConfig(merged)
    }

    public func loadDefaultConfig() {
        let merged = (try? RimeConfigStore.loadEffective(baseURL: defaultYAML, customURL: defaultCustomYAML)) ?? [:]
        parseDefaultConfig(merged)
    }

    public func loadKeyBindings() {
        let dict = (try? RimeConfigStore.loadEffective(baseURL: defaultYAML, customURL: defaultCustomYAML)) ?? [:]
        if let keyBinder = dict["key_binder"] as? [String: Any],
           let bindings = keyBinder["bindings"] as? [[String: Any]] {
            keyBindings = bindings.compactMap { item in
                guard let when = item["when"] as? String,
                      let accept = item["accept"] as? String else { return nil }
                return KeyBindingItem(
                    when: when,
                    accept: accept,
                    send: item["send"] as? String ?? "",
                    toggle: item["toggle"] as? String ?? "",
                    select: item["select"] as? String ?? ""
                )
            }
        }
    }

    private func parseFlowBinding(_ str: String) -> KeyBindingItem? {
        // Strip trailing comment
        var s = str
        if let hashIndex = s.firstIndex(of: "#") {
            s = String(s[s.startIndex..<hashIndex])
        }

        let cleaned = s.replacingOccurrences(of: "{", with: "")
            .replacingOccurrences(of: "}", with: "")

        var when = "", accept = "", send = "", toggle = "", select = ""
        let parts = cleaned.split(separator: ",").map { $0.trimmingCharacters(in: .whitespaces) }
        for part in parts {
            guard let colonIndex = part.firstIndex(of: ":") else { continue }
            let key = String(part[part.startIndex..<colonIndex]).trimmingCharacters(in: .whitespaces)
            let value = String(part[part.index(after: colonIndex)...]).trimmingCharacters(in: .whitespaces)
                .replacingOccurrences(of: "'", with: "")
                .replacingOccurrences(of: "\"", with: "")

            switch key {
            case "when": when = value
            case "accept": accept = value
            case "send": send = value
            case "toggle": toggle = value
            case "select": select = value
            default: break
            }
        }

        guard !when.isEmpty, !accept.isEmpty else { return nil }
        return KeyBindingItem(when: when, accept: accept, send: send, toggle: toggle, select: select)
    }

    public func loadPunctuation() {
        let dict = (try? RimeConfigStore.loadEffective(baseURL: defaultYAML, customURL: defaultCustomYAML)) ?? [:]

        if let punctuator = dict["punctuator"] as? [String: Any] {
            if let half = punctuator["half_shape"] as? [String: Any] {
                halfShapePunct = parsePunctDict(half)
            }
            if let full = punctuator["full_shape"] as? [String: Any] {
                fullShapePunct = parsePunctDict(full)
            }
        }
    }

    private func parsePunctDict(_ dict: [String: Any]) -> [PunctRule] {
        var rules: [PunctRule] = []
        for (key, value) in dict {
            var commit = ""
            var pair: [String] = []
            var list: [String] = []

            if let commitDict = value as? [String: Any] {
                commit = commitDict["commit"] as? String ?? ""
                if let p = commitDict["pair"] as? [String] {
                    pair = p
                }
            } else if let l = value as? [String] {
                list = l
            } else if let str = value as? String {
                if str.hasPrefix("{") && str.hasSuffix("}") {
                    let inner = String(str.dropFirst().dropLast()).trimmingCharacters(in: .whitespaces)
                    if inner.hasPrefix("commit:") {
                        commit = String(inner.dropFirst("commit:".count)).trimmingCharacters(in: .whitespaces)
                    } else if inner.hasPrefix("pair:") {
                        let pairStr = String(inner.dropFirst("pair:".count)).trimmingCharacters(in: .whitespaces)
                        pair = parsePairArray(pairStr)
                    }
                } else if str.hasPrefix("[") && str.hasSuffix("]") {
                    let inner = String(str.dropFirst().dropLast()).trimmingCharacters(in: .whitespaces)
                    list = inner.split(separator: ",").map { $0.trimmingCharacters(in: .whitespaces).replacingOccurrences(of: "'", with: "") }
                } else {
                    commit = str
                }
            }

            rules.append(PunctRule(key: key, commit: commit, pair: pair, list: list))
        }
        return rules.sorted { $0.key < $1.key }
    }

    private func parsePairArray(_ str: String) -> [String] {
        let cleaned = str.replacingOccurrences(of: "[", with: "")
            .replacingOccurrences(of: "]", with: "")
            .replacingOccurrences(of: "'", with: "")
            .replacingOccurrences(of: "\"", with: "")
        return cleaned.split(separator: ",").map { $0.trimmingCharacters(in: .whitespaces) }
    }

    func saveSquirrelCustom() {
        var existingPatch: [String: Any] = [:]
        if let content = loadFile(squirrelCustomYAML) {
            let dict = YAMLParser.shared.parse(content)
            if let patch = dict["patch"] as? [String: Any] {
                existingPatch = patch
            }
        }

        var stylePatch = existingPatch["style"] as? [String: Any] ?? [:]
        stylePatch["color_scheme"] = squirrelStyle.colorSchemeName
        stylePatch["color_scheme_dark"] = squirrelStyle.colorSchemeDarkName
        stylePatch["text_orientation"] = squirrelStyle.textOrientation
        stylePatch["inline_preedit"] = squirrelStyle.inlinePreedit
        stylePatch["inline_candidate"] = squirrelStyle.inlineCandidate
        stylePatch["translucency"] = squirrelStyle.translucency
        stylePatch["mutual_exclusive"] = squirrelStyle.mutualExclusive
        stylePatch["memorize_size"] = squirrelStyle.memorizeSize
        stylePatch["show_paging"] = squirrelStyle.showPaging
        stylePatch["candidate_list_layout"] = squirrelStyle.candidateListLayout
        stylePatch["candidate_format"] = squirrelStyle.candidateFormat
        stylePatch["status_message_type"] = squirrelStyle.statusMessageType
        stylePatch["alpha"] = squirrelStyle.alpha
        stylePatch["corner_radius"] = squirrelStyle.cornerRadius
        stylePatch["hilited_corner_radius"] = squirrelStyle.hilitedCornerRadius
        stylePatch["border_height"] = squirrelStyle.borderHeight
        stylePatch["border_width"] = squirrelStyle.borderWidth
        stylePatch["line_spacing"] = squirrelStyle.lineSpacing
        stylePatch["spacing"] = squirrelStyle.spacing
        stylePatch["shadow_size"] = squirrelStyle.shadowSize
        stylePatch["font_face"] = squirrelStyle.fontFace
        stylePatch["font_point"] = squirrelStyle.fontPoint
        stylePatch["label_font_face"] = squirrelStyle.labelFontFace
        stylePatch["label_font_point"] = squirrelStyle.labelFontPoint
        stylePatch["comment_font_face"] = squirrelStyle.commentFontFace
        stylePatch["comment_font_point"] = squirrelStyle.commentFontPoint

        var patch = existingPatch
        patch["style"] = stylePatch

        let existingSchemes = existingPatch["preset_color_schemes"] as? [String: Any] ?? [:]
        var schemesDict: [String: Any] = [:]
        for (key, scheme) in colorSchemes {
            schemesDict[key] = scheme.toDict()
        }
        for (key, scheme) in darkColorSchemes {
            schemesDict[key] = scheme.toDict()
        }
        for (key, value) in existingSchemes where schemesDict[key] == nil {
            schemesDict[key] = value
        }
        if !schemesDict.isEmpty {
            patch["preset_color_schemes"] = schemesDict
        }

        let yaml = serializePatch(patch: patch)
        let content = "patch:\n\(yaml)"

        print("💾 saveSquirrelCustom: color_scheme='\(stylePatch["color_scheme"] ?? "nil")', dark='\(stylePatch["color_scheme_dark"] ?? "nil")', schemes=\(schemesDict.count)")
        print("💾 first 200 chars: \(content.prefix(200))")

        guard !yaml.isEmpty else {
            print("⚠️ RimeVision: refusing to write empty squirrel.custom.yaml (would lose all config)")
            return
        }

        // Backup existing file before overwriting
        let backupURL = squirrelCustomYAML.appendingPathExtension("bak")
        if FileManager.default.fileExists(atPath: squirrelCustomYAML.path) {
            try? FileManager.default.copyItem(at: squirrelCustomYAML, to: backupURL)
        }

        do {
            try content.write(to: squirrelCustomYAML, atomically: true, encoding: .utf8)
        } catch {
            print("Error saving squirrel.custom.yaml: \(error)")
        }
    }

    func saveSquirrelStyle() throws {
        try RimeConfigStore.savePatch(customURL: squirrelCustomYAML) { patch in
            var stylePatch = patch["style"] as? [String: Any] ?? [:]
            stylePatch["color_scheme"] = squirrelStyle.colorSchemeName
            stylePatch["color_scheme_dark"] = squirrelStyle.colorSchemeDarkName
            stylePatch["text_orientation"] = squirrelStyle.textOrientation
            stylePatch["inline_preedit"] = squirrelStyle.inlinePreedit
            stylePatch["inline_candidate"] = squirrelStyle.inlineCandidate
            stylePatch["translucency"] = squirrelStyle.translucency
            stylePatch["mutual_exclusive"] = squirrelStyle.mutualExclusive
            stylePatch["memorize_size"] = squirrelStyle.memorizeSize
            stylePatch["show_paging"] = squirrelStyle.showPaging
            stylePatch["candidate_list_layout"] = squirrelStyle.candidateListLayout
            stylePatch["candidate_format"] = squirrelStyle.candidateFormat
            stylePatch["status_message_type"] = squirrelStyle.statusMessageType
            stylePatch["alpha"] = squirrelStyle.alpha
            stylePatch["corner_radius"] = squirrelStyle.cornerRadius
            stylePatch["hilited_corner_radius"] = squirrelStyle.hilitedCornerRadius
            stylePatch["border_height"] = squirrelStyle.borderHeight
            stylePatch["border_width"] = squirrelStyle.borderWidth
            stylePatch["line_spacing"] = squirrelStyle.lineSpacing
            stylePatch["spacing"] = squirrelStyle.spacing
            stylePatch["shadow_size"] = squirrelStyle.shadowSize
            stylePatch["font_face"] = squirrelStyle.fontFace
            stylePatch["font_point"] = squirrelStyle.fontPoint
            stylePatch["label_font_face"] = squirrelStyle.labelFontFace
            stylePatch["label_font_point"] = squirrelStyle.labelFontPoint
            stylePatch["comment_font_face"] = squirrelStyle.commentFontFace
            stylePatch["comment_font_point"] = squirrelStyle.commentFontPoint
            patch["style"] = stylePatch

            var schemesDict = patch["preset_color_schemes"] as? [String: Any] ?? [:]
            for (key, scheme) in colorSchemes { schemesDict[key] = scheme.toDict() }
            for (key, scheme) in darkColorSchemes { schemesDict[key] = scheme.toDict() }
            if !schemesDict.isEmpty { patch["preset_color_schemes"] = schemesDict }
        }
    }

    func saveGeneralSettings(
        menuPageSize: Int,
        translatorEncoder: Bool,
        translatorSentence: Bool,
        translatorUserDict: Bool,
        translatorCommitHistory: Bool,
        switcherCaption: String,
        switcherHotkeys: [String],
        switcherFold: Bool,
        switcherAbbreviate: Bool,
        asciiCapsLock: String,
        asciiShiftL: String,
        asciiShiftR: String,
        asciiGoodOld: Bool
    ) throws {
        try RimeConfigStore.savePatch(customURL: defaultCustomYAML) { patch in
            RimePatch.setValue(menuPageSize, in: &patch, path: ["menu", "page_size"])
            RimePatch.setValue(translatorEncoder, in: &patch, path: ["translator", "enable_encoder"])
            RimePatch.setValue(translatorSentence, in: &patch, path: ["translator", "enable_sentence"])
            RimePatch.setValue(translatorUserDict, in: &patch, path: ["translator", "enable_user_dict"])
            RimePatch.setValue(translatorCommitHistory, in: &patch, path: ["translator", "encode_commit_history"])
            RimePatch.setValue(switcherCaption, in: &patch, path: ["switcher", "caption"])
            RimePatch.setValue(switcherHotkeys, in: &patch, path: ["switcher", "hotkeys"])
            RimePatch.setValue(switcherFold, in: &patch, path: ["switcher", "fold_options"])
            RimePatch.setValue(switcherAbbreviate, in: &patch, path: ["switcher", "abbreviate_options"])
            RimePatch.setValue(asciiGoodOld, in: &patch, path: ["ascii_composer", "good_old_caps_lock"])
            RimePatch.setValue(asciiCapsLock, in: &patch, path: ["ascii_composer", "switch_key", "Caps_Lock"])
            RimePatch.setValue(asciiShiftL, in: &patch, path: ["ascii_composer", "switch_key", "Shift_L"])
            RimePatch.setValue(asciiShiftR, in: &patch, path: ["ascii_composer", "switch_key", "Shift_R"])
        }
        pageSize = menuPageSize
    }

    func savePunctuation(half: [PunctRule], full: [PunctRule]) throws {
        try RimeConfigStore.savePatch(customURL: defaultCustomYAML) { patch in
            var punctPatch = patch["punctuator"] as? [String: Any] ?? [:]
            punctPatch["half_shape"] = serializePunctRules(half)
            punctPatch["full_shape"] = serializePunctRules(full)
            patch["punctuator"] = punctPatch
        }
        halfShapePunct = half
        fullShapePunct = full
    }

    func saveSchemas(_ schemasToSave: [RimeSchema]) throws {
        try RimeConfigStore.savePatch(customURL: defaultCustomYAML) { patch in
            patch["schema_list"] = schemasToSave.filter(\.enabled).map { ["schema": $0.schemaId] }
        }
        schemas = schemasToSave
    }

    public func saveKeyBindings(_ bindings: [KeyBindingItem]) throws {
        try RimeConfigStore.savePatch(customURL: defaultCustomYAML) { patch in
            patch["key_binder"] = [
                "bindings": bindings.map { binding in
                    var item: [String: Any] = [
                        "when": binding.when,
                        "accept": binding.accept
                    ]
                    if !binding.send.isEmpty { item["send"] = binding.send }
                    if !binding.toggle.isEmpty { item["toggle"] = binding.toggle }
                    if !binding.select.isEmpty { item["select"] = binding.select }
                    return item
                }
            ]
        }
        keyBindings = bindings
    }

    func saveDefaultCustom(
        menuPageSize: Int? = nil,
        translatorEncoder: Bool? = nil,
        translatorSentence: Bool? = nil,
        translatorUserDict: Bool? = nil,
        translatorCommitHistory: Bool? = nil,
        switcherCaption: String? = nil,
        switcherHotkeys: [String]? = nil,
        switcherFold: Bool? = nil,
        switcherAbbreviate: Bool? = nil,
        asciiCapsLock: String? = nil,
        asciiShiftL: String? = nil,
        asciiShiftR: String? = nil,
        asciiGoodOld: Bool? = nil,
        keyBindingsToSave: [KeyBindingItem]? = nil,
        halfPunct: [PunctRule]? = nil,
        fullPunct: [PunctRule]? = nil,
        schemasToSave: [RimeSchema]? = nil
    ) {
        var existingPatch: [String: Any] = [:]
        if let content = loadFile(defaultCustomYAML) {
            let dict = YAMLParser.shared.parse(content)
            if let patch = dict["patch"] as? [String: Any] {
                existingPatch = patch
            }
        }

        var patch = existingPatch

        let effectiveSchemas = schemasToSave ?? schemas
        if !effectiveSchemas.isEmpty {
            let schemaList = effectiveSchemas.filter(\.enabled).map { ["schema": $0.schemaId] }
            patch["schema_list"] = schemaList
        }

        var menuPatch = existingPatch["menu"] as? [String: Any] ?? [:]
        menuPatch["page_size"] = Double(menuPageSize ?? pageSize)
        patch["menu"] = menuPatch

        var translatorPatch = existingPatch["translator"] as? [String: Any] ?? [:]
        if let v = translatorEncoder { translatorPatch["enable_encoder"] = v }
        if let v = translatorSentence { translatorPatch["enable_sentence"] = v }
        if let v = translatorUserDict { translatorPatch["enable_user_dict"] = v }
        if let v = translatorCommitHistory { translatorPatch["encode_commit_history"] = v }
        patch["translator"] = translatorPatch

        var switcherPatch = existingPatch["switcher"] as? [String: Any] ?? [:]
        if let v = switcherCaption { switcherPatch["caption"] = v }
        if let v = switcherHotkeys { switcherPatch["hotkeys"] = v }
        if let v = switcherFold { switcherPatch["fold_options"] = v }
        if let v = switcherAbbreviate { switcherPatch["abbreviate_options"] = v }
        patch["switcher"] = switcherPatch

        var asciiPatch = existingPatch["ascii_composer"] as? [String: Any] ?? [:]
        if let v = asciiGoodOld { asciiPatch["good_old_caps_lock"] = v }
        var switchKey = asciiPatch["switch_key"] as? [String: Any] ?? [:]
        if let v = asciiCapsLock { switchKey["Caps_Lock"] = v }
        if let v = asciiShiftL { switchKey["Shift_L"] = v }
        if let v = asciiShiftR { switchKey["Shift_R"] = v }
        asciiPatch["switch_key"] = switchKey
        patch["ascii_composer"] = asciiPatch

        let effectiveBindings = keyBindingsToSave ?? keyBindings
        if !effectiveBindings.isEmpty {
            var keyBinderPatch: [String: Any] = [:]
            var bindingsList: [[String: Any]] = []
            for binding in effectiveBindings {
                var item: [String: Any] = ["when": binding.when, "accept": binding.accept]
                if !binding.send.isEmpty { item["send"] = binding.send }
                if !binding.toggle.isEmpty { item["toggle"] = binding.toggle }
                if !binding.select.isEmpty { item["select"] = binding.select }
                bindingsList.append(item)
            }
            keyBinderPatch["bindings"] = bindingsList
            patch["key_binder"] = keyBinderPatch
        }

        let effectiveHalf = halfPunct ?? halfShapePunct
        let effectiveFull = fullPunct ?? fullShapePunct
        if !effectiveHalf.isEmpty || !effectiveFull.isEmpty {
            var punctPatch: [String: Any] = [:]
            if !effectiveHalf.isEmpty {
                punctPatch["half_shape"] = serializePunctRules(effectiveHalf)
            }
            if !effectiveFull.isEmpty {
                punctPatch["full_shape"] = serializePunctRules(effectiveFull)
            }
            patch["punctuator"] = punctPatch
        }

        let yaml = serializePatch(patch: patch)
        let content = "patch:\n\(yaml)"

        guard !yaml.isEmpty else {
            print("⚠️ RimeVision: refusing to write empty default.custom.yaml (would lose all config)")
            return
        }

        let backupURL = defaultCustomYAML.appendingPathExtension("bak")
        if FileManager.default.fileExists(atPath: defaultCustomYAML.path) {
            try? FileManager.default.copyItem(at: defaultCustomYAML, to: backupURL)
        }

        do {
            try content.write(to: defaultCustomYAML, atomically: true, encoding: .utf8)
        } catch {
            print("Error saving default.custom.yaml: \(error)")
        }
    }

    private func serializePunctRules(_ rules: [PunctRule]) -> [String: Any] {
        var dict: [String: Any] = [:]
        for rule in rules {
            if !rule.commit.isEmpty {
                dict[rule.key] = ["commit": rule.commit]
            } else if !rule.pair.isEmpty {
                dict[rule.key] = ["pair": rule.pair]
            } else if !rule.list.isEmpty {
                dict[rule.key] = rule.list
            }
        }
        return dict
    }

    private func serializePunctValue(_ value: Any) -> String {
        if let dict = value as? [String: Any] {
            if let commit = dict["commit"] as? String {
                return "{ commit: \(escapeYAMLString(commit)) }"
            } else if let pair = dict["pair"] as? [String] {
                let pairStr = pair.map { escapeYAMLString($0) }.joined(separator: ", ")
                return "{ pair: [ \(pairStr) ] }"
            }
        } else if let list = value as? [String] {
            let listStr = list.map { escapeYAMLString($0) }.joined(separator: ", ")
            return "[ \(listStr) ]"
        }
        return "\(value)"
    }

    private func loadFile(_ url: URL) -> String? {
        guard FileManager.default.fileExists(atPath: url.path) else { return nil }
        return try? String(contentsOf: url, encoding: .utf8)
    }

    private func applyPatch(base: [String: Any], custom: [String: Any]) -> [String: Any] {
        var result = base
        if let patch = custom["patch"] as? [String: Any] {
            // __delete__: remove listed keys from result
            if let deletions = patch["__delete__"] as? [String] {
                for key in deletions {
                    result.removeValue(forKey: key)
                }
            }
            // __append__: append items to existing lists
            if let appends = patch["__append__"] as? [String: Any] {
                for (key, value) in appends {
                    if let newItems = value as? [Any] {
                        if var existing = result[key] as? [Any] {
                            existing.append(contentsOf: newItems)
                            result[key] = existing
                        } else {
                            result[key] = newItems
                        }
                    }
                }
            }
            // normal merge for remaining keys
            for (key, value) in patch {
                if key == "__delete__" || key == "__append__" { continue }
                if let nestedBase = result[key] as? [String: Any],
                   let nestedPatch = value as? [String: Any] {
                    result[key] = applyPatch(base: nestedBase, custom: ["patch": nestedPatch])
                } else {
                    result[key] = value
                }
            }
        }
        return result
    }

    private func parseSquirrelConfig(_ dict: [String: Any]) {
        if let style = dict["style"] as? [String: Any] {
            parseStyle(style)
        }

        if let schemes = dict["preset_color_schemes"] as? [String: Any] {
            var light: [String: RimeColorScheme] = [:]
            var dark: [String: RimeColorScheme] = [:]

            for (key, value) in schemes {
                guard let schemeDict = value as? [String: Any] else { continue }
                let scheme = RimeColorScheme.from(dict: schemeDict, name: key)
                if isDarkScheme(dict: schemeDict, schemeName: key) {
                    dark[key] = scheme
                } else {
                    light[key] = scheme
                }
            }

            self.colorSchemes = light
            self.darkColorSchemes = dark
        }
    }

    private func parseStyle(_ dict: [String: Any]) {
        var style = RimeStyle()

        if let v = dict["color_scheme"] as? String { style.colorSchemeName = v }
        if let v = dict["color_scheme_dark"] as? String { style.colorSchemeDarkName = v }
        if let v = dict["status_message_type"] as? String { style.statusMessageType = v }
        if let v = dict["candidate_format"] as? String { style.candidateFormat = v }
        if let v = dict["text_orientation"] as? String { style.textOrientation = v }
        if let v = dict["inline_preedit"] as? Bool { style.inlinePreedit = v }
        if let v = dict["inline_candidate"] as? Bool { style.inlineCandidate = v }
        if let v = dict["translucency"] as? Bool { style.translucency = v }
        if let v = dict["mutual_exclusive"] as? Bool { style.mutualExclusive = v }
        if let v = dict["memorize_size"] as? Bool { style.memorizeSize = v }
        if let v = dict["show_paging"] as? Bool { style.showPaging = v }
        if let v = dict["candidate_list_layout"] as? String { style.candidateListLayout = v }
        if let v = dict["alpha"] as? Double { style.alpha = v }
        if let v = dict["corner_radius"] as? Double { style.cornerRadius = v }
        if let v = dict["hilited_corner_radius"] as? Double { style.hilitedCornerRadius = v }
        if let v = dict["border_height"] as? Double { style.borderHeight = v }
        if let v = dict["border_width"] as? Double { style.borderWidth = v }
        if let v = dict["line_spacing"] as? Double { style.lineSpacing = v }
        if let v = dict["spacing"] as? Double { style.spacing = v }
        if let v = dict["shadow_size"] as? Double { style.shadowSize = v }
        if let v = dict["font_face"] as? String { style.fontFace = v }
        if let v = dict["font_point"] as? Double { style.fontPoint = v }
        if let v = dict["label_font_face"] as? String { style.labelFontFace = v }
        if let v = dict["label_font_point"] as? Double { style.labelFontPoint = v }
        if let v = dict["comment_font_face"] as? String { style.commentFontFace = v }
        if let v = dict["comment_font_point"] as? Double { style.commentFontPoint = v }

        self.squirrelStyle = style
    }

    private func isDarkScheme(dict: [String: Any], schemeName: String) -> Bool {
        if schemeName.lowercased().contains("dark") { return true }
        if let backColor = dict["back_color"] as? String,
           let color = RimeColor.from(hex: backColor) {
            let luminance = (0.299 * Double(color.r) + 0.587 * Double(color.g) + 0.114 * Double(color.b)) / 255.0
            return luminance < 0.4
        }
        return false
    }

    private func parseDefaultConfig(_ dict: [String: Any]) {
        var parsedSchemas: [RimeSchema] = []
        if let schemaList = dict["schema_list"] as? [[String: Any]] {
            for item in schemaList {
                if let schemaId = item["schema"] as? String {
                    parsedSchemas.append(RimeSchema(schemaId: schemaId, enabled: true))
                }
            }
        }
        self.schemas = parsedSchemas

        if let menu = dict["menu"] as? [String: Any] {
            if let ps = menu["page_size"] as? Int {
                self.pageSize = ps
            } else if let ps = menu["page_size"] as? Double {
                self.pageSize = Int(ps)
            }
        }

        if let keyBinder = dict["key_binder"] as? [String: Any],
           let bindings = keyBinder["bindings"] as? [[String: Any]] {
            self.keyBindings = bindings.compactMap { item in
                guard let when = item["when"] as? String,
                      let accept = item["accept"] as? String else { return nil }
                return KeyBindingItem(
                    when: when,
                    accept: accept,
                    send: item["send"] as? String ?? "",
                    toggle: item["toggle"] as? String ?? "",
                    select: item["select"] as? String ?? ""
                )
            }
        }
    }

    func serializePatch(patch: [String: Any], indent: Int = 2) -> String {
        var lines: [String] = []
        let prefix = String(repeating: " ", count: indent)

        for (key, value) in patch {
            if let nestedDict = value as? [String: Any] {
                if isPunctDict(nestedDict) {
                    lines.append("\(prefix)\(key): \(serializePunctValue(nestedDict))")
                } else {
                    lines.append("\(prefix)\(key):")
                    lines.append(serializePatch(patch: nestedDict, indent: indent + 2))
                }
            } else if let array = value as? [[String: Any]] {
                lines.append("\(prefix)\(key):")
                for item in array {
                    if let schemaId = item["schema"] as? String {
                        lines.append("\(prefix)  - schema: \(schemaId)")
                    }
                }
            } else if let array = value as? [String] {
                lines.append("\(prefix)\(key):")
                for item in array {
                    lines.append("\(prefix)  - \(escapeYAMLString(item))")
                }
            } else if let boolVal = value as? Bool {
                lines.append("\(prefix)\(key): \(boolVal ? "true" : "false")")
            } else if let numVal = value as? Double {
                if numVal == Double(Int(numVal)) {
                    lines.append("\(prefix)\(key): \(Int(numVal))")
                } else {
                    lines.append("\(prefix)\(key): \(numVal)")
                }
            } else if let strVal = value as? String {
                lines.append("\(prefix)\(key): \(escapeYAMLString(strVal))")
            }
        }
        return lines.joined(separator: "\n")
    }

    private func isPunctDict(_ dict: [String: Any]) -> Bool {
        return dict.keys.contains("commit") || dict.keys.contains("pair")
    }

    private func escapeYAMLString(_ str: String) -> String {
        if str.isEmpty { return "\"\"" }
        let needsQuoting = str.contains(":") || str.contains("#") || str.contains("{") ||
            str.contains("}") || str.contains("[") || str.contains("]") ||
            str.hasPrefix(" ") || str.hasSuffix(" ") || str.hasPrefix("\"") ||
            str.hasPrefix("'") || str.isHexString || str == "true" || str == "false"
        if needsQuoting {
            let escaped = str.replacingOccurrences(of: "\\", with: "\\\\")
                .replacingOccurrences(of: "\"", with: "\\\"")
            return "\"\(escaped)\""
        }
        return str
    }
}

private extension String {
    var isHexString: Bool {
        guard hasPrefix("0x") || hasPrefix("0X") else { return false }
        let hexPart = String(dropFirst(2))
        return !hexPart.isEmpty && hexPart.allSatisfy { $0.isHexDigit }
    }
}
