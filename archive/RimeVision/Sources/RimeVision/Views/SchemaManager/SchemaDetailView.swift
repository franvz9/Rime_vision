import SwiftUI

struct SchemaDetailView: View {
    let schema: RimeSchema
    let allSchemas: [RimeSchema]
    @Environment(\.dismiss) private var dismiss

    @State private var schemaContent: String = ""
    @State private var isLoading = true

    var body: some View {
        VStack(spacing: 0) {
            header
            Divider()
            ScrollView {
                VStack(alignment: .leading, spacing: 16) {
                    basicInfoSection
                    dependenciesSection
                    configPreviewSection
                }
                .padding()
            }
        }
        .frame(width: 600, height: 500)
        .frame(minWidth: 600, minHeight: 500)
        .onAppear {
            loadSchemaFile()
        }
    }

    private var header: some View {
        HStack {
            VStack(alignment: .leading) {
                Text(schema.schemaId)
                    .font(.title2)
                    .fontWeight(.semibold)
                if !schema.name.isEmpty {
                    Text(schema.name)
                        .font(.subheadline)
                        .foregroundColor(.secondary)
                }
            }

            Spacer()

            Button("关闭") {
                dismiss()
            }
            .keyboardShortcut(.escape)
        }
        .padding()
    }

    private var basicInfoSection: some View {
        GroupBox("基本信息") {
            VStack(alignment: .leading, spacing: 8) {
                DetailRow(label: "Schema ID", value: schema.schemaId)
                if !schema.name.isEmpty {
                    DetailRow(label: "名称", value: schema.name)
                }
                if !schema.version.isEmpty {
                    DetailRow(label: "版本", value: schema.version)
                }
                DetailRow(label: "状态", value: schema.enabled ? "已启用" : "已禁用")
            }
            .padding(8)
        }
    }

    private var dependenciesSection: some View {
        GroupBox("依赖关系") {
            if schema.dependencies.isEmpty {
                Text("无依赖")
                    .foregroundColor(.secondary)
                    .padding(8)
            } else {
                VStack(alignment: .leading, spacing: 6) {
                    ForEach(schema.dependencies, id: \.self) { depId in
                        HStack {
                            Image(systemName: "arrow.right.circle")
                                .foregroundColor(.accentColor)
                                .font(.caption)

                            Text(depId)
                                .font(.system(.body, design: .monospaced))

                            if let dep = allSchemas.first(where: { $0.schemaId == depId }) {
                                Text(dep.name)
                                    .font(.caption)
                                    .foregroundColor(.secondary)
                            }

                            Spacer()

                            if allSchemas.contains(where: { $0.schemaId == depId }) {
                                Image(systemName: "checkmark.circle.fill")
                                    .foregroundColor(.green)
                                    .font(.caption)
                            } else {
                                Image(systemName: "exclamationmark.circle.fill")
                                    .foregroundColor(.orange)
                                    .font(.caption)
                                    .help("未安装")
                            }
                        }
                        .padding(.vertical, 2)
                    }
                }
                .padding(8)
            }
        }
    }

    private var configPreviewSection: some View {
        GroupBox("配置文件预览") {
            if isLoading {
                HStack {
                    Spacer()
                    ProgressView("加载中...")
                    Spacer()
                }
                .frame(maxWidth: .infinity)
                .frame(height: 200)
            } else if schemaContent.isEmpty {
                VStack {
                    Spacer()
                    Text("无法加载配置文件")
                        .foregroundColor(.secondary)
                    Spacer()
                }
                .frame(maxWidth: .infinity, minHeight: 100)
            } else {
                ScrollView([.horizontal, .vertical]) {
                    Text(schemaContent)
                        .font(.system(.caption, design: .monospaced))
                        .textSelection(.enabled)
                        .padding(8)
                        .frame(maxWidth: .infinity, alignment: .leading)
                }
                .frame(maxHeight: 300)
            }
        }
    }

    private func loadSchemaFile() {
        isLoading = true
        let safeId = URL(fileURLWithPath: schema.schemaId).lastPathComponent
        let schemaFile = FileManager.default.homeDirectoryForCurrentUser
            .appendingPathComponent("Library/Rime")
            .appendingPathComponent("\(safeId).schema.yaml")

        Task.detached(priority: .userInitiated) {
            let content = (try? String(contentsOf: schemaFile, encoding: .utf8)) ?? ""
            await MainActor.run {
                self.schemaContent = content
                self.isLoading = false
            }
        }
    }
}

struct DetailRow: View {
    let label: String
    let value: String

    var body: some View {
        HStack(alignment: .top) {
            Text(label + ":")
                .foregroundColor(.secondary)
                .frame(width: 80, alignment: .trailing)
            Text(value)
                .font(.system(.body, design: .monospaced))
        }
    }
}
