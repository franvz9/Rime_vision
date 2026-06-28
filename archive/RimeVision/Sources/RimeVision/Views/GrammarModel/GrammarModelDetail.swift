import SwiftUI

struct GrammarModelDetail: View {
    let model: GrammarModel
    @ObservedObject var manager: GrammarModelManager
    let schemas: [RimeSchema]
    let onSave: () -> Void
    let onError: (String) -> Void

    @State private var selectedSchemaId: String?
    @State private var editingConfig: SchemaGrammarConfig?
    @State private var isSaving = false
    @State private var showDeleteConfirm = false

    private static let dateFormatter: DateFormatter = {
        let f = DateFormatter()
        f.dateFormat = "yyyy-MM-dd HH:mm"
        return f
    }()

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 16) {
                modelInfoSection
                Divider()
                mountSchemasSection
                if editingConfig != nil {
                    Divider()
                    parameterConfigSection
                }
            }
            .padding()
        }
    }

    // ── MARK: 模型信息 ──

    private var modelInfoSection: some View {
        GroupBox("模型信息") {
            VStack(alignment: .leading, spacing: 8) {
                infoRow(label: "名称", value: model.displayName)
                infoRow(label: "文件大小", value: model.formattedSize)
                infoRow(label: "修改时间", value: Self.dateFormatter.string(from: model.modificationDate))

                HStack {
                    Button(role: .destructive) {
                        showDeleteConfirm = true
                    } label: {
                        Label("删除模型", systemImage: "trash")
                    }
                    .buttonStyle(.bordered)
                    .confirmationDialog("确认删除 \(model.filename).gram？", isPresented: $showDeleteConfirm, titleVisibility: .visible) {
                        Button("删除", role: .destructive) {
                            do {
                                try manager.deleteModel(model, schemaIds: schemas.map(\.schemaId))
                            } catch {
                                onError("删除失败: \(error.localizedDescription)")
                            }
                        }
                        Button("取消", role: .cancel) {}
                    }
                }
                .padding(.top, 4)
            }
            .padding(8)
        }
    }

    // ── MARK: 挂载方案列表 ──

    private var mountSchemasSection: some View {
        GroupBox("挂载方案") {
            if schemas.isEmpty {
                Text("无可用的输入方案")
                    .foregroundColor(.secondary)
                    .padding(8)
            } else {
                VStack(spacing: 0) {
                    ForEach(schemas) { schema in
                        schemaMountRow(schema)
                        if schema.id != schemas.last?.id {
                            Divider()
                        }
                    }
                }
            }
        }
    }

    private func schemaMountRow(_ schema: RimeSchema) -> some View {
        let config = manager.mountConfigs[schema.schemaId] ?? SchemaGrammarConfig.defaultConfig(for: schema.schemaId)
        let isMounted = config.mountedModel == model.filename
        let isSelected = selectedSchemaId == schema.schemaId

        return HStack {
            Image(systemName: isMounted ? "checkmark.circle.fill" : "circle")
                .foregroundColor(isMounted ? .green : .gray)

            VStack(alignment: .leading, spacing: 2) {
                Text(schema.schemaId)
                    .font(.system(.body, design: .monospaced))
                if let mounted = config.mountedModel, mounted != model.filename {
                    Text("已挂载: \(mounted)")
                        .font(.caption)
                        .foregroundColor(.orange)
                }
            }

            Spacer()

            if isSelected && isMounted {
                Text("编辑中")
                    .font(.caption)
                    .foregroundColor(.accentColor)
                    .padding(.horizontal, 6)
                    .padding(.vertical, 2)
                    .background(Color.accentColor.opacity(0.1))
                    .cornerRadius(4)
            }

            Button(isMounted ? "卸载" : "挂载") {
                if isMounted {
                    do {
                        try manager.unmount(from: schema.schemaId)
                        if selectedSchemaId == schema.schemaId {
                            editingConfig = nil
                            selectedSchemaId = nil
                        }
                        onSave()
                    } catch {
                        onError("卸载失败: \(error.localizedDescription)")
                    }
                } else {
                    // 挂载使用默认参数
                    let newConfig = SchemaGrammarConfig.defaultConfig(for: schema.schemaId).withModel(model)
                    do {
                        try manager.mount(model: model, to: schema.schemaId, config: newConfig)
                        selectedSchemaId = schema.schemaId
                        editingConfig = manager.mountConfigs[schema.schemaId]
                        onSave()
                    } catch {
                        onError("挂载失败: \(error.localizedDescription)")
                    }
                }
            }
            .buttonStyle(.bordered)
            .controlSize(.small)
        }
        .padding(.vertical, 6)
        .padding(.horizontal, 8)
        .background(
            RoundedRectangle(cornerRadius: 4)
                .fill(isSelected ? Color.accentColor.opacity(0.05) : Color.clear)
        )
        .contentShape(Rectangle())
        .onTapGesture {
            if isMounted {
                selectedSchemaId = schema.schemaId
                editingConfig = manager.mountConfigs[schema.schemaId]
            }
        }
    }

    // ── MARK: 参数配置 ──

    private var parameterConfigSection: some View {
        GroupBox("参数配置 — \(selectedSchemaId ?? "")") {
            if editingConfig != nil {
                configEditor(binding: Binding(
                    get: { editingConfig! },
                    set: { editingConfig = $0 }
                ))
            } else {
                Text("选择一个已挂载的方案来编辑参数")
                    .foregroundColor(.secondary)
                    .padding(8)
            }
        }
    }

    private func configEditor(binding: Binding<SchemaGrammarConfig>) -> some View {
        VStack(alignment: .leading, spacing: 12) {
            // Grammar 参数
            Text("Grammar 参数")
                .font(.headline)

            stepperRow("搭配最大长度", value: binding.collocationMaxLength, range: 3...10)
            stepperRow("搭配最小长度", value: binding.collocationMinLength, range: 1...5)
            stepperRow("搭配惩罚", value: binding.collocationPenalty, range: -64...0)
            stepperRow("非搭配惩罚", value: binding.nonCollocationPenalty, range: -64...0)
            stepperRow("弱搭配惩罚", value: binding.weakCollocationPenalty, range: -200...0)
            stepperRow("后置惩罚", value: binding.rearPenalty, range: -100...0)

            Divider()

            // Translator 参数
            Text("Translator 参数")
                .font(.headline)

            Toggle("启用上下文建议", isOn: binding.contextualSuggestions)

            stepperRow("同音词数", value: binding.maxHomophones, range: 1...20)
            stepperRow("同形词数", value: binding.maxHomographs, range: 1...20)

            Divider()

            HStack {
                Button("保存配置") {
                    guard let schemaId = selectedSchemaId,
                          let configToSave = editingConfig else { return }
                    do {
                        try manager.updateConfig(for: schemaId, config: configToSave)
                        onSave()
                    } catch {
                        onError("保存失败: \(error.localizedDescription)")
                    }
                }
                .buttonStyle(.borderedProminent)
                .disabled(isSaving)

                Button("重置为默认") {
                    editingConfig = SchemaGrammarConfig.defaultConfig(for: selectedSchemaId ?? "").withModel(model)
                }
                .buttonStyle(.bordered)
            }
        }
        .padding(8)
    }

    private func stepperRow(_ label: String, value: Binding<Int>, range: ClosedRange<Int>) -> some View {
        HStack {
            Text(label)
                .frame(width: 140, alignment: .trailing)
            Stepper("\(value.wrappedValue)", value: value, in: range)
                .frame(width: 120)
        }
    }

    // ── MARK: Helpers ──

    private func infoRow(label: String, value: String) -> some View {
        HStack {
            Text(label)
                .foregroundColor(.secondary)
                .frame(width: 80, alignment: .trailing)
            Text(": ")
                .foregroundColor(.secondary)
            Text(value)
                .font(.system(.body, design: .monospaced))
        }
    }
}
