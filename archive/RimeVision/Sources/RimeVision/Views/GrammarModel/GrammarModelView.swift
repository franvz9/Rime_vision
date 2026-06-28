import SwiftUI
import UniformTypeIdentifiers

struct GrammarModelView: View {
    @EnvironmentObject var appState: AppState
    @State private var selectedModel: GrammarModel?
    @State private var showImportFilePicker = false
    @State private var showBatchSheet = false
    @State private var showSaved = false
    @State private var errorMessage: String?
    @State private var showError = false

    private var manager: GrammarModelManager {
        appState.grammarModelManager
    }

    private var schemas: [RimeSchema] {
        appState.configManager.schemas
    }

    var body: some View {
        VStack(spacing: 0) {
            toolbar
            Divider()
            HSplitView {
                modelListPane
                    .frame(minWidth: 220, idealWidth: 260, maxWidth: 300)

                if let model = selectedModel {
                    GrammarModelDetail(
                        model: model,
                        manager: manager,
                        schemas: schemas,
                        onSave: { showSaved = true },
                        onError: { msg in
                            errorMessage = msg
                            showError = true
                        }
                    )
                    .frame(minWidth: 400)
                } else {
                    Text("选择一个模型查看详情")
                        .foregroundColor(.secondary)
                        .frame(minWidth: 400, maxWidth: .infinity, maxHeight: .infinity)
                }
            }
        }
        .navigationTitle("语言模型")
        .onAppear {
            manager.loadAll(schemaIds: schemas.map(\.schemaId))
        }
        .fileImporter(
            isPresented: $showImportFilePicker,
            allowedContentTypes: [.item],
            allowsMultipleSelection: true
        ) { result in handleImport(result) }
        .sheet(isPresented: $showBatchSheet) {
            if let model = selectedModel {
                SchemaMountSheet(
                    model: model,
                    manager: manager,
                    schemas: schemas,
                    onDismiss: { showBatchSheet = false }
                )
            }
        }
        .overlay(alignment: .top) {
            if showSaved {
                Text("已保存")
                    .font(.caption)
                    .padding(.horizontal, 12)
                    .padding(.vertical, 6)
                    .background(.green.opacity(0.9))
                    .foregroundColor(.white)
                    .cornerRadius(6)
                    .padding(.top, 8)
                    .transition(.move(edge: .top).combined(with: .opacity))
                    .onAppear {
                        DispatchQueue.main.asyncAfter(deadline: .now() + 1.5) {
                            withAnimation { showSaved = false }
                        }
                    }
            }
        }
        .alert("错误", isPresented: $showError) {
            Button("确定") {}
        } message: {
            Text(errorMessage ?? "未知错误")
        }
    }

    // ── MARK: Toolbar ──

    private var toolbar: some View {
        HStack(spacing: 8) {
            Button {
                showImportFilePicker = true
            } label: {
                Label("导入模型", systemImage: "plus.circle")
            }
            .buttonStyle(.bordered)

            Button {
                showBatchSheet = true
            } label: {
                Label("批量挂载", systemImage: "rectangle.stack.badge.plus")
            }
            .buttonStyle(.bordered)
            .disabled(selectedModel == nil)

            Spacer()

            Button {
                manager.loadAll(schemaIds: schemas.map(\.schemaId))
            } label: {
                Label("刷新", systemImage: "arrow.clockwise")
            }
            .buttonStyle(.bordered)
        }
        .padding(.horizontal)
        .padding(.vertical, 8)
    }

    // ── MARK: Model List ──

    private var modelListPane: some View {
        VStack(spacing: 0) {
            Text("可用模型")
                .font(.headline)
                .padding()

            if manager.models.isEmpty {
                VStack(spacing: 12) {
                    Image(systemName: "cube.box")
                        .font(.system(size: 36))
                        .foregroundColor(.secondary)
                    Text("未找到语言模型文件")
                        .foregroundColor(.secondary)
                    Text("点击「导入模型」添加 .gram 文件")
                        .font(.caption)
                        .foregroundColor(.secondary)
                }
                .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else {
                List(manager.models, selection: $selectedModel) { model in
                    GrammarModelRow(
                        model: model,
                        mountedCount: manager.mountedSchemaCount(for: model),
                        isSelected: selectedModel?.id == model.id
                    )
                    .tag(model)
                }
            }
        }
    }

    // ── MARK: Import Handler ──

    private func handleImport(_ result: Result<[URL], Error>) {
        switch result {
        case .success(let urls):
            let gramURLs = urls.filter { $0.pathExtension == "gram" }
            if gramURLs.isEmpty {
                errorMessage = "未选择 .gram 文件"
                showError = true
                return
            }
            for url in gramURLs {
                do {
                    try manager.importModel(from: url)
                } catch {
                    errorMessage = "导入 \(url.lastPathComponent) 失败: \(error.localizedDescription)"
                    showError = true
                }
            }
            // 刷新选中状态
            if selectedModel != nil, let oldId = selectedModel?.id {
                selectedModel = manager.models.first { $0.id == oldId }
            }
        case .failure(let error):
            errorMessage = "导入失败: \(error.localizedDescription)"
            showError = true
        }
    }
}

// ── MARK: Model Row ──

struct GrammarModelRow: View {
    let model: GrammarModel
    let mountedCount: Int
    let isSelected: Bool

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            HStack {
                Image(systemName: "cube.box.fill")
                    .foregroundColor(.accentColor)
                Text(model.displayName)
                    .font(.system(.body, design: .monospaced))
                    .lineLimit(1)
            }
            HStack(spacing: 8) {
                Text(model.formattedSize)
                    .font(.caption)
                    .foregroundColor(.secondary)
                if mountedCount > 0 {
                    Text("\(mountedCount) 个方案已挂载")
                        .font(.caption)
                        .foregroundColor(.green)
                }
            }
        }
        .padding(.vertical, 4)
        .padding(.horizontal, 8)
        .background(
            RoundedRectangle(cornerRadius: 6)
                .fill(isSelected ? Color.accentColor.opacity(0.1) : Color.clear)
        )
    }
}
