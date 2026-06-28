import SwiftUI

struct KeybindingEditorView: View {
    @EnvironmentObject var appState: AppState
    @State private var bindings: [ConfigManager.KeyBindingItem] = []
    @State private var selectedCategory: Category = .paging
    @State private var editingItem: ConfigManager.KeyBindingItem?
    @State private var showSaved = false

    enum Category: String, CaseIterable {
        case paging = "翻页"
        case cursor = "光标移动"
        case state = "状态切换"
        case emacs = "Emacs"
        case all = "全部"
    }

    var body: some View {
        VStack(spacing: 0) {
            HStack {
                Picker("分类", selection: $selectedCategory) {
                    ForEach(Category.allCases, id: \.self) { cat in
                        Text(cat.rawValue).tag(cat)
                    }
                }
                .pickerStyle(.segmented)

                Button {
                    editingItem = ConfigManager.KeyBindingItem(when: "always", accept: "", send: "", toggle: "", select: "")
                } label: {
                    Image(systemName: "plus")
                }
            }
            .padding()

            Divider()

            List {
                ForEach(filteredBindings) { binding in
                    KeyBindingRow(binding: binding)
                        .contextMenu {
                            Button("编辑") {
                                editingItem = binding
                            }
                            Divider()
                            Button("删除", role: .destructive) {
                                bindings.removeAll { $0.id == binding.id }
                            }
                        }
                        .onTapGesture(count: 2) {
                            editingItem = binding
                        }
                }
                .onMove { source, destination in
                    bindings.move(fromOffsets: source, toOffset: destination)
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
        .navigationTitle("快捷键配置")
        .sheet(item: $editingItem) { item in
            KeyBindingEditSheet(
                binding: item,
                onSave: { updated in
                    if let index = bindings.firstIndex(where: { $0.id == updated.id }) {
                        bindings[index] = updated
                    } else {
                        bindings.append(updated)
                    }
                    editingItem = nil
                },
                onCancel: { editingItem = nil }
            )
        }
        .onAppear {
            bindings = appState.configManager.keyBindings
        }
    }

    private var filteredBindings: [ConfigManager.KeyBindingItem] {
        switch selectedCategory {
        case .paging:
            return bindings.filter { $0.when == "paging" || $0.when == "has_menu" }
        case .cursor:
            return bindings.filter { $0.when == "composing" && $0.actionType == "send" }
        case .state:
            return bindings.filter { $0.when == "always" && ($0.actionType == "toggle" || $0.actionType == "select") }
        case .emacs:
            return bindings.filter { $0.when == "composing" && $0.accept.hasPrefix("Control") }
        case .all:
            return bindings
        }
    }

    private func save() {
        do {
            try appState.configManager.saveKeyBindings(bindings)
            showSaved = true
            DispatchQueue.main.asyncAfter(deadline: .now() + 2) {
                showSaved = false
            }
        } catch {
            print("Error saving key bindings: \(error)")
        }
    }
}

struct KeyBindingRow: View {
    let binding: ConfigManager.KeyBindingItem

    var body: some View {
        HStack(spacing: 12) {
            Text(binding.when)
                .font(.caption)
                .foregroundColor(.secondary)
                .frame(width: 80, alignment: .leading)

            Text(binding.accept)
                .font(.system(.body, design: .monospaced))

            Image(systemName: "arrow.right")
                .foregroundColor(.secondary)
                .font(.caption)

            Text(binding.actionValue)
                .font(.system(.body, design: .monospaced))
                .foregroundColor(.accentColor)

            Spacer()

            Text(binding.actionType)
                .font(.caption2)
                .foregroundColor(.secondary)
                .padding(.horizontal, 6)
                .padding(.vertical, 2)
                .background(Color.secondary.opacity(0.1))
                .cornerRadius(4)
        }
        .padding(.vertical, 4)
    }
}

struct KeyBindingEditSheet: View {
    @FocusState private var focusedField: Field?
    @State private var when: String
    @State private var accept: String
    @State private var actionType: String
    @State private var actionValue: String
    @State private var originalId: UUID

    let onSave: (ConfigManager.KeyBindingItem) -> Void
    let onCancel: () -> Void

    private let whenOptions = ["always", "composing", "paging", "has_menu", "predicting"]

    enum Field {
        case actionValue
    }

    init(binding: ConfigManager.KeyBindingItem, onSave: @escaping (ConfigManager.KeyBindingItem) -> Void, onCancel: @escaping () -> Void) {
        _when = State(initialValue: binding.when)
        _accept = State(initialValue: binding.accept)
        _actionType = State(initialValue: {
            if !binding.toggle.isEmpty { return "toggle" }
            if !binding.select.isEmpty { return "select" }
            return "send"
        }())
        _actionValue = State(initialValue: {
            if !binding.toggle.isEmpty { return binding.toggle }
            if !binding.select.isEmpty { return binding.select }
            return binding.send
        }())
        _originalId = State(initialValue: binding.id)
        self.onSave = onSave
        self.onCancel = onCancel
    }

    var body: some View {
        VStack(spacing: 16) {
            Text("编辑快捷键")
                .font(.headline)

            VStack(spacing: 12) {
                HStack {
                    Text("触发条件:")
                        .frame(width: 80, alignment: .trailing)
                    Picker("", selection: $when) {
                        ForEach(whenOptions, id: \.self) { Text($0) }
                    }
                    .labelsHidden()
                }

                HStack {
                    Text("按键:")
                        .frame(width: 80, alignment: .trailing)
                    KeyCaptureField(text: $accept, placeholder: "点击后按下快捷键")
                        .frame(minWidth: 180)
                        .frame(height: 24)
                }

                HStack {
                    Text("动作类型:")
                        .frame(width: 80, alignment: .trailing)
                    Picker("", selection: $actionType) {
                        Text("send").tag("send")
                        Text("toggle").tag("toggle")
                        Text("select").tag("select")
                    }
                    .labelsHidden()
                    .onChange(of: actionType) { _ in
                        focusedField = .actionValue
                    }
                }

                HStack {
                    Text("动作值:")
                        .frame(width: 80, alignment: .trailing)
                    if actionType == "send" {
                        KeyCaptureField(text: $actionValue, placeholder: "按下目标键")
                            .frame(minWidth: 180)
                        .frame(height: 24)
                    } else if actionType == "toggle" {
                        TextField("ascii_mode", text: $actionValue)
                            .textFieldStyle(.roundedBorder)
                            .focused($focusedField, equals: .actionValue)
                    } else {
                        TextField(".next", text: $actionValue)
                            .textFieldStyle(.roundedBorder)
                            .focused($focusedField, equals: .actionValue)
                    }
                }
            }

            HStack {
                Button("取消") { onCancel() }
                Spacer()
                Button("保存") {
                    var updated = ConfigManager.KeyBindingItem(
                        when: when,
                        accept: accept,
                        send: actionType == "send" ? actionValue : "",
                        toggle: actionType == "toggle" ? actionValue : "",
                        select: actionType == "select" ? actionValue : ""
                    )
                    updated.id = originalId
                    onSave(updated)
                }
                .buttonStyle(.borderedProminent)
                .disabled(accept.isEmpty || actionValue.isEmpty)
            }
        }
        .padding()
        .frame(width: 400, height: 250)
        .onAppear {
            NSApp.activate(ignoringOtherApps: true)
        }
    }
}
