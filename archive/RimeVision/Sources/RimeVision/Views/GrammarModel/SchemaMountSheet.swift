import SwiftUI

struct SchemaMountSheet: View {
    let model: GrammarModel
    @ObservedObject var manager: GrammarModelManager
    let schemas: [RimeSchema]
    let onDismiss: () -> Void

    @State private var selectedSchemaIds: Set<String> = []
    @State private var config = SchemaGrammarConfig.default
    @State private var isProcessing = false
    @State private var errorMessage: String?
    @State private var showError = false

    var body: some View {
        VStack(spacing: 0) {
            // Header
            HStack {
                Image(systemName: "rectangle.stack.badge.plus")
                    .foregroundColor(.accentColor)
                Text("批量操作 — \(model.displayName)")
                    .font(.headline)
                Spacer()
            }
            .padding()

            Divider()

            HStack(spacing: 0) {
                // Left: schema selection list
                VStack(alignment: .leading, spacing: 0) {
                    HStack {
                        Text("选择方案")
                            .font(.subheadline)
                            .foregroundColor(.secondary)
                        Spacer()
                        Button("全选") {
                            selectedSchemaIds = Set(schemas.map(\.schemaId))
                        }
                        .buttonStyle(.borderless)
                        .font(.caption)
                        Button("清除") {
                            selectedSchemaIds.removeAll()
                        }
                        .buttonStyle(.borderless)
                        .font(.caption)
                    }
                    .padding(.horizontal)
                    .padding(.top, 8)

                    List(schemas, selection: $selectedSchemaIds) { schema in
                        let mounted = manager.mountConfigs[schema.schemaId]?.mountedModel == model.filename
                        HStack {
                            Text(schema.schemaId)
                                .font(.system(.body, design: .monospaced))
                            Spacer()
                            if mounted {
                                Text("已挂载")
                                    .font(.caption)
                                    .foregroundColor(.green)
                            }
                        }
                        .tag(schema.schemaId)
                    }
                }
                .frame(width: 280)

                Divider()

                // Right: parameter config
                ScrollView {
                    VStack(alignment: .leading, spacing: 12) {
                        Text("参数配置")
                            .font(.subheadline)
                            .foregroundColor(.secondary)

                        paramRow("搭配最大长度", value: $config.collocationMaxLength, range: 3...10)
                        paramRow("搭配最小长度", value: $config.collocationMinLength, range: 1...5)
                        paramRow("搭配惩罚", value: $config.collocationPenalty, range: -64...0)
                        paramRow("非搭配惩罚", value: $config.nonCollocationPenalty, range: -64...0)
                        paramRow("弱搭配惩罚", value: $config.weakCollocationPenalty, range: -200...0)
                        paramRow("后置惩罚", value: $config.rearPenalty, range: -100...0)

                        Divider()

                        Toggle("启用上下文建议", isOn: $config.contextualSuggestions)

                        paramRow("同音词数", value: $config.maxHomophones, range: 1...20)
                        paramRow("同形词数", value: $config.maxHomographs, range: 1...20)

                        Divider()

                        HStack(spacing: 12) {
                            Button("批量挂载") {
                                performBatchMount()
                            }
                            .buttonStyle(.borderedProminent)
                            .disabled(selectedSchemaIds.isEmpty || isProcessing)

                            Button("批量卸载") {
                                performBatchUnmount()
                            }
                            .buttonStyle(.bordered)
                            .disabled(selectedSchemaIds.isEmpty || isProcessing)

                            Spacer()

                            Button("取消") {
                                onDismiss()
                            }
                            .buttonStyle(.bordered)
                        }
                    }
                    .padding()
                }
                .frame(minWidth: 220)
            }
        }
        .frame(width: 540, height: 450)
        .alert("错误", isPresented: $showError) {
            Button("确定") {}
        } message: {
            Text(errorMessage ?? "未知错误")
        }
    }

    private func paramRow(_ label: String, value: Binding<Int>, range: ClosedRange<Int>) -> some View {
        HStack {
            Text(label)
                .frame(width: 120, alignment: .trailing)
            Stepper("\(value.wrappedValue)", value: value, in: range)
                .frame(width: 120)
        }
    }

    private func performBatchMount() {
        isProcessing = true
        defer { isProcessing = false }
        do {
            try manager.batchMount(
                model: model,
                to: Array(selectedSchemaIds),
                config: config.withModel(model)
            )
            onDismiss()
        } catch {
            errorMessage = "批量挂载失败: \(error.localizedDescription)"
            showError = true
        }
    }

    private func performBatchUnmount() {
        isProcessing = true
        defer { isProcessing = false }
        do {
            try manager.batchUnmount(from: Array(selectedSchemaIds))
            onDismiss()
        } catch {
            errorMessage = "批量卸载失败: \(error.localizedDescription)"
            showError = true
        }
    }
}
