import SwiftUI

struct PunctuationSettingsView: View {
    @EnvironmentObject var appState: AppState
    @State private var selectedTab: Tab = .halfShape
    @State private var halfShapeRules: [ConfigManager.PunctRule] = []
    @State private var fullShapeRules: [ConfigManager.PunctRule] = []
    @State private var editingRule: ConfigManager.PunctRule?
    @State private var showSaved = false

    enum Tab: String, CaseIterable {
        case halfShape = "半角标点"
        case fullShape = "全角标点"
    }

    var body: some View {
        VStack(spacing: 0) {
            HStack {
                Picker("标点类型", selection: $selectedTab) {
                    ForEach(Tab.allCases, id: \.self) { tab in
                        Text(tab.rawValue).tag(tab)
                    }
                }
                .pickerStyle(.segmented)

                Button {
                    editingRule = ConfigManager.PunctRule(key: "", commit: "", pair: [], list: [])
                } label: {
                    Image(systemName: "plus")
                }
            }
            .padding()

            Divider()

            List {
                ForEach(currentRules) { rule in
                    HStack {
                        Text(rule.key)
                            .font(.system(.body, design: .monospaced))
                            .frame(width: 50, alignment: .center)

                        Text("→")
                            .foregroundColor(.secondary)

                        if !rule.commit.isEmpty {
                            Text(rule.commit)
                                .font(.system(.body, design: .monospaced))
                            Text("(commit)")
                                .font(.caption)
                                .foregroundColor(.secondary)
                        } else if !rule.pair.isEmpty {
                            Text("[\(rule.pair.joined(separator: ", "))]")
                                .font(.system(.body, design: .monospaced))
                            Text("(pair)")
                                .font(.caption)
                                .foregroundColor(.secondary)
                        } else if !rule.list.isEmpty {
                            Text("[\(rule.list.joined(separator: ", "))]")
                                .font(.system(.body, design: .monospaced))
                            Text("(list)")
                                .font(.caption)
                                .foregroundColor(.secondary)
                        }

                        Spacer()

                        Button {
                            editingRule = rule
                        } label: {
                            Image(systemName: "pencil")
                                .font(.caption)
                        }
                        .buttonStyle(.plain)

                        Button {
                            if selectedTab == .halfShape {
                                halfShapeRules.removeAll { $0.id == rule.id }
                            } else {
                                fullShapeRules.removeAll { $0.id == rule.id }
                            }
                        } label: {
                            Image(systemName: "trash")
                                .font(.caption)
                                .foregroundColor(.red)
                        }
                        .buttonStyle(.plain)
                    }
                    .padding(.vertical, 4)
                }
            }
            .listStyle(.inset(alternatesRowBackgrounds: true))

            HStack {
                Spacer()
                if showSaved {
                    Text("已保存")
                        .foregroundColor(.green)
                }
                Button("保存到 default.custom.yaml") {
                    save()
                }
                .buttonStyle(.borderedProminent)
            }
            .padding()
        }
        .navigationTitle("标点符号设置")
        .sheet(item: $editingRule) { rule in
            PunctRuleEditSheet(
                rule: rule,
                onSave: { updated in
                    if selectedTab == .halfShape {
                        if let index = halfShapeRules.firstIndex(where: { $0.id == updated.id }) {
                            halfShapeRules[index] = updated
                        } else {
                            halfShapeRules.append(updated)
                        }
                    } else {
                        if let index = fullShapeRules.firstIndex(where: { $0.id == updated.id }) {
                            fullShapeRules[index] = updated
                        } else {
                            fullShapeRules.append(updated)
                        }
                    }
                    editingRule = nil
                },
                onCancel: { editingRule = nil }
            )
        }
        .onAppear {
            halfShapeRules = appState.configManager.halfShapePunct
            fullShapeRules = appState.configManager.fullShapePunct
        }
    }

    private var currentRules: [ConfigManager.PunctRule] {
        selectedTab == .halfShape ? halfShapeRules : fullShapeRules
    }

    private func save() {
        do {
            try appState.configManager.savePunctuation(half: halfShapeRules, full: fullShapeRules)
            showSaved = true
            DispatchQueue.main.asyncAfter(deadline: .now() + 2) {
                showSaved = false
            }
        } catch {
            print("Error saving punctuation: \(error)")
        }
    }
}

struct PunctRuleEditSheet: View {
    @FocusState private var focusedField: Field?
    @State private var key: String
    @State private var editType: EditType
    @State private var commitValue: String
    @State private var pairValues: String
    @State private var listValues: String
    @State private var originalId: UUID

    let onSave: (ConfigManager.PunctRule) -> Void
    let onCancel: () -> Void

    enum EditType: String, CaseIterable {
        case commit = "直接上屏"
        case pair = "配对输入"
        case list = "候选列表"
    }

    enum Field {
        case key, commitValue, pairValues, listValues
    }

    init(rule: ConfigManager.PunctRule, onSave: @escaping (ConfigManager.PunctRule) -> Void, onCancel: @escaping () -> Void) {
        _key = State(initialValue: rule.key)
        _commitValue = State(initialValue: rule.commit)
        _pairValues = State(initialValue: rule.pair.joined(separator: ", "))
        _listValues = State(initialValue: rule.list.joined(separator: ", "))
        _originalId = State(initialValue: rule.id)
        _editType = State(initialValue: {
            if !rule.pair.isEmpty { return .pair }
            if !rule.list.isEmpty { return .list }
            return .commit
        }())
        self.onSave = onSave
        self.onCancel = onCancel
    }

    var body: some View {
        VStack(spacing: 16) {
            Text("编辑标点")
                .font(.headline)

            VStack(spacing: 12) {
                HStack {
                    Text("按键:")
                        .frame(width: 80, alignment: .trailing)
                    TextField(",", text: $key)
                        .textFieldStyle(.roundedBorder)
                        .frame(width: 80)
                        .focused($focusedField, equals: .key)
                }

                HStack {
                    Text("类型:")
                        .frame(width: 80, alignment: .trailing)
                    Picker("", selection: $editType) {
                        ForEach(EditType.allCases, id: \.self) { Text($0.rawValue) }
                    }
                    .labelsHidden()
                }

                switch editType {
                case .commit:
                    HStack {
                        Text("输出:")
                            .frame(width: 80, alignment: .trailing)
                        TextField("，", text: $commitValue)
                            .textFieldStyle(.roundedBorder)
                            .focused($focusedField, equals: .commitValue)
                    }
                case .pair:
                    HStack {
                        Text("配对:")
                            .frame(width: 80, alignment: .trailing)
                        TextField("「, 」", text: $pairValues)
                            .textFieldStyle(.roundedBorder)
                            .focused($focusedField, equals: .pairValues)
                    }
                case .list:
                    HStack {
                        Text("候选:")
                            .frame(width: 80, alignment: .trailing)
                        TextField("、, ＼", text: $listValues)
                            .textFieldStyle(.roundedBorder)
                            .focused($focusedField, equals: .listValues)
                    }
                }
            }

            HStack {
                Button("取消") { onCancel() }
                Spacer()
                Button("保存") {
                    var updated = ConfigManager.PunctRule(
                        key: key,
                        commit: editType == .commit ? commitValue : "",
                        pair: editType == .pair ? pairValues.split(separator: ",").map { $0.trimmingCharacters(in: .whitespaces) } : [],
                        list: editType == .list ? listValues.split(separator: ",").map { $0.trimmingCharacters(in: .whitespaces) } : []
                    )
                    updated.id = originalId
                    onSave(updated)
                }
                .buttonStyle(.borderedProminent)
                .disabled(key.isEmpty)
            }
        }
        .padding()
        .frame(width: 400, height: 250)
        .onAppear {
            NSApp.activate(ignoringOtherApps: true)
            focusedField = .key
        }
    }
}
