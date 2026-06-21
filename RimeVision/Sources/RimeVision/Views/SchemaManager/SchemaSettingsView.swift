import SwiftUI

struct SchemaSettingsView: View {
    @EnvironmentObject var appState: AppState
    @State private var selectedSchema: RimeSchema?
    @State private var schemaConfig: [String: Any] = [:]
    @State private var switches: [SchemaSwitch] = []
    @State private var spellerAlgebra: [String] = []
    @State private var translatorSettings: [String: Any] = [:]
    @State private var grammarSettings: [String: Any] = [:]
    @State private var showSaved = false

    struct SchemaSwitch: Identifiable {
        var id = UUID()
        var name: String
        var states: [String]
        var reset: Int
    }

    var body: some View {
        HSplitView {
            VStack(spacing: 0) {
                Text("选择方案")
                    .font(.headline)
                    .padding()

                List(appState.configManager.schemas) { schema in
                    SchemaConfigRow(
                        schema: schema,
                        isSelected: selectedSchema?.id == schema.id
                    )
                    .onTapGesture {
                        selectedSchema = schema
                        loadSchemaConfig(schema)
                    }
                }
            }
            .frame(minWidth: 200, idealWidth: 250)

            if let schema = selectedSchema {
                SchemaConfigDetail(schema: schema, config: $schemaConfig, switches: $switches, spellerAlgebra: $spellerAlgebra, translatorSettings: $translatorSettings, grammarSettings: $grammarSettings)
            } else {
                Text("选择一个方案查看配置")
                    .foregroundColor(.secondary)
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
            }
        }
        .navigationTitle("方案配置")
    }

    private func loadSchemaConfig(_ schema: RimeSchema) {
        let userDir = FileManager.default.homeDirectoryForCurrentUser.appendingPathComponent("Library/Rime")
        let safeId = URL(fileURLWithPath: schema.schemaId).lastPathComponent
        let schemaFile = userDir.appendingPathComponent("\(safeId).schema.yaml")
        let customFile = userDir.appendingPathComponent("\(safeId).custom.yaml")

        Task.detached(priority: .userInitiated) {
            guard let dict = try? RimeConfigStore.loadYAML(from: schemaFile) else { return }

            var switchesResult: [SchemaSwitch] = []
            var spellerResult: [String] = []
            var translatorResult: [String: Any] = [:]
            var grammarResult: [String: Any] = [:]

            if let sw = dict["switches"] as? [[String: Any]] {
                switchesResult = sw.compactMap { item in
                    guard let name = item["name"] as? String else { return nil }
                    let states = item["states"] as? [String] ?? []
                    let reset = item["reset"] as? Int ?? 0
                    return SchemaSwitch(name: name, states: states, reset: reset)
                }
            }

            if let speller = dict["speller"] as? [String: Any],
               let algebra = speller["algebra"] as? [String] {
                spellerResult = algebra
            }

            if let translator = dict["translator"] as? [String: Any] {
                translatorResult = translator
            }

            if let grammar = dict["grammar"] as? [String: Any] {
                grammarResult = grammar
            }

            var mergedDict = dict

            if let customDict = try? RimeConfigStore.loadYAML(from: customFile),
               let patch = customDict["patch"] as? [String: Any] {
                let merged = RimePatch.merge(base: dict, patch: patch)
                if let grammar = merged["grammar"] as? [String: Any] {
                    grammarResult = grammar
                }
                mergedDict = merged
            }

            let finalConfig = mergedDict
            let finalSwitches = switchesResult
            let finalSpeller = spellerResult
            let finalTranslator = translatorResult
            let finalGrammar = grammarResult

            await MainActor.run {
                self.schemaConfig = finalConfig
                self.switches = finalSwitches
                self.spellerAlgebra = finalSpeller
                self.translatorSettings = finalTranslator
                self.grammarSettings = finalGrammar
            }
        }
    }
}

struct SchemaConfigRow: View {
    let schema: RimeSchema
    let isSelected: Bool

    var body: some View {
        HStack {
            Circle()
                .fill(schema.enabled ? Color.green : Color.gray)
                .frame(width: 8, height: 8)
            Text(schema.schemaId)
                .font(.system(.body, design: .monospaced))
        }
        .padding(.vertical, 4)
        .padding(.horizontal, 8)
        .background(
            RoundedRectangle(cornerRadius: 6)
                .fill(isSelected ? Color.accentColor.opacity(0.1) : Color.clear)
        )
    }
}

struct SchemaConfigDetail: View {
    let schema: RimeSchema
    @Binding var config: [String: Any]
    @Binding var switches: [SchemaSettingsView.SchemaSwitch]
    @Binding var spellerAlgebra: [String]
    @Binding var translatorSettings: [String: Any]
    @Binding var grammarSettings: [String: Any]
    @State private var selectedSection: Section = .switches

    enum Section: String, CaseIterable {
        case switches = "开关选项"
        case speller = "拼写设置"
        case translator = "翻译器"
        case grammar = "语法模型"
        case engine = "引擎"
    }

    var body: some View {
        VStack(spacing: 0) {
            Picker("配置项", selection: $selectedSection) {
                ForEach(Section.allCases, id: \.self) { section in
                    Text(section.rawValue).tag(section)
                }
            }
            .pickerStyle(.segmented)
            .padding()

            Divider()

            ScrollView {
                switch selectedSection {
                case .switches:
                    switchesSection
                case .speller:
                    spellerSection
                case .translator:
                    translatorSection
                case .grammar:
                    grammarSection
                case .engine:
                    engineSection
                }
            }
            .padding()
        }
    }

    private var switchesSection: some View {
        VStack(alignment: .leading, spacing: 8) {
            if switches.isEmpty {
                Text("该方案没有开关选项")
                    .foregroundColor(.secondary)
            } else {
                ForEach(switches) { sw in
                    HStack {
                        VStack(alignment: .leading) {
                            Text(sw.name)
                                .font(.system(.body, design: .monospaced))
                            if !sw.states.isEmpty {
                                Text("状态: \(sw.states.joined(separator: " / "))")
                                    .font(.caption)
                                    .foregroundColor(.secondary)
                            }
                        }
                        Spacer()
                        Text("默认: \(sw.reset)")
                            .font(.caption)
                            .foregroundColor(.secondary)
                    }
                    .padding(8)
                    .background(Color(NSColor.controlBackgroundColor))
                    .cornerRadius(6)
                }
            }
        }
    }

    private var spellerSection: some View {
        VStack(alignment: .leading, spacing: 8) {
            if spellerAlgebra.isEmpty {
                Text("该方案没有拼写设置")
                    .foregroundColor(.secondary)
            } else {
                Text("拼写代数规则 (\(spellerAlgebra.count) 条)")
                    .font(.headline)
                ForEach(Array(spellerAlgebra.enumerated()), id: \.offset) { index, rule in
                    HStack(alignment: .top) {
                        Text("\(index + 1).")
                            .foregroundColor(.secondary)
                            .frame(width: 30, alignment: .trailing)
                        Text(rule)
                            .font(.system(.caption, design: .monospaced))
                    }
                }
            }
        }
    }

    private var translatorSection: some View {
        VStack(alignment: .leading, spacing: 8) {
            if translatorSettings.isEmpty {
                Text("该方案没有翻译器设置")
                    .foregroundColor(.secondary)
            } else {
                ForEach(Array(translatorSettings.keys.sorted()), id: \.self) { key in
                    if let value = translatorSettings[key] {
                        HStack(alignment: .top) {
                            Text(key)
                                .font(.system(.body, design: .monospaced))
                                .foregroundColor(.accentColor)
                                .frame(width: 200, alignment: .trailing)
                            Text(": ")
                                .foregroundColor(.secondary)
                            Text("\(value)")
                                .font(.system(.body, design: .monospaced))
                        }
                    }
                }
            }
        }
    }

    private var grammarSection: some View {
        VStack(alignment: .leading, spacing: 8) {
            if grammarSettings.isEmpty {
                Text("该方案没有语法模型配置")
                    .foregroundColor(.secondary)
                Text("万象模型配置位于 rime_mint.custom.yaml")
                    .font(.caption)
                    .foregroundColor(.secondary)
            } else {
                ForEach(Array(grammarSettings.keys.sorted()), id: \.self) { key in
                    if let value = grammarSettings[key] {
                        HStack(alignment: .top) {
                            Text(key)
                                .font(.system(.body, design: .monospaced))
                                .foregroundColor(.accentColor)
                                .frame(width: 200, alignment: .trailing)
                            Text(": ")
                                .foregroundColor(.secondary)
                            Text("\(value)")
                                .font(.system(.body, design: .monospaced))
                        }
                    }
                }
            }

            Divider()

            GroupBox("万象模型说明") {
                VStack(alignment: .leading, spacing: 8) {
                    Text("万象拼音语法模型 (wanxiang-lts-zh-hans)")
                        .font(.headline)
                    Text("• language: 语言模型名称")
                    Text("• collocation_max_length: 最大搭配长度 (默认 5)")
                    Text("• collocation_min_length: 最小搭配长度 (默认 2)")
                    Text("• 影响词组搭配和智能联想")
                }
                .font(.caption)
                .frame(maxWidth: .infinity, alignment: .leading)
            }
        }
    }

    private var engineSection: some View {
        VStack(alignment: .leading, spacing: 12) {
            if let engine = config["engine"] as? [String: Any] {
                ForEach(["processors", "segmentors", "translators", "filters"], id: \.self) { key in
                    if let items = engine[key] as? [String] {
                        GroupBox(key) {
                            VStack(alignment: .leading, spacing: 4) {
                                ForEach(items, id: \.self) { item in
                                    Text(item)
                                        .font(.system(.caption, design: .monospaced))
                                }
                            }
                            .frame(maxWidth: .infinity, alignment: .leading)
                        }
                    }
                }
            } else {
                Text("该方案没有引擎配置")
                    .foregroundColor(.secondary)
            }
        }
    }
}
