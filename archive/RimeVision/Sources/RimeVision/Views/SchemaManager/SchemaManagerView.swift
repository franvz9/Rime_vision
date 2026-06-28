import SwiftUI

struct SchemaManagerView: View {
    @EnvironmentObject var appState: AppState
    @State private var schemas: [RimeSchema] = []
    @State private var searchText: String = ""
    @State private var selectedSchema: RimeSchema?
    @State private var showDependencyGraph = false
    @State private var showImportSheet = false
    @State private var showExportAlert = false
    @State private var showSaved = false

    private var filteredSchemas: [RimeSchema] {
        if searchText.isEmpty {
            return schemas
        }
        return schemas.filter {
            $0.schemaId.localizedCaseInsensitiveContains(searchText) ||
            $0.name.localizedCaseInsensitiveContains(searchText)
        }
    }

    var body: some View {
        VStack(spacing: 0) {
            toolbar
            Divider()
            HSplitView {
                schemaList
                if showDependencyGraph {
                    DependencyGraphView(schemas: schemas)
                        .frame(minWidth: 250)
                }
            }

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
        .navigationTitle("方案管理")
        .onAppear {
            schemas = appState.configManager.schemas
        }
        .sheet(item: $selectedSchema) { schema in
            SchemaDetailView(
                schema: schema,
                allSchemas: schemas
            )
        }
        .fileImporter(
            isPresented: $showImportSheet,
            allowedContentTypes: [.yaml, .plainText],
            allowsMultipleSelection: false
        ) { result in
            handleImport(result)
        }
        .alert("导出方案列表", isPresented: $showExportAlert) {
            Button("导出为 YAML") {
                exportSchemas()
            }
            Button("取消", role: .cancel) {}
        } message: {
            Text("将导出当前方案列表配置")
        }
    }

    private var toolbar: some View {
        HStack(spacing: 12) {
            HStack {
                Image(systemName: "magnifyingglass")
                    .foregroundColor(.secondary)
                TextField("搜索方案...", text: $searchText)
                    .textFieldStyle(.plain)
            }
            .padding(6)
            .background(Color(NSColor.controlBackgroundColor))
            .cornerRadius(6)
            .frame(maxWidth: 250)

            Spacer()

            Button {
                showDependencyGraph.toggle()
            } label: {
                Image(systemName: "point.3.connected.trianglepath.dotted")
            }
            .help("依赖关系图")
            .toggleStyle(.button)

            Divider()
                .frame(height: 20)

            Button {
                showImportSheet = true
            } label: {
                Image(systemName: "square.and.arrow.down")
            }
            .help("导入方案")

            Button {
                showExportAlert = true
            } label: {
                Image(systemName: "square.and.arrow.up")
            }
            .help("导出方案列表")

            Divider()
                .frame(height: 20)

            Text("\(schemas.filter(\.enabled).count)/\(schemas.count) 已启用")
                .font(.caption)
                .foregroundColor(.secondary)
        }
        .padding(.horizontal)
        .padding(.vertical, 8)
    }

    private var schemaList: some View {
        List {
            Section {
                ForEach(filteredSchemas) { schema in
                    SchemaRowView(schema: schema)
                        .contextMenu {
                            Button("查看详情") {
                                selectedSchema = schema
                            }
                            Button(schema.enabled ? "禁用" : "启用") {
                                toggleSchema(schema)
                            }
                            Divider()
                            Button("上移") {
                                moveSchema(schema, direction: .up)
                            }
                            Button("下移") {
                                moveSchema(schema, direction: .down)
                            }
                            Divider()
                            Button("移除", role: .destructive) {
                                removeSchema(schema)
                            }
                        }
                        .onTapGesture(count: 2) {
                            selectedSchema = schema
                        }
                }
                .onMove { source, destination in
                    schemas.move(fromOffsets: source, toOffset: destination)
                    updateSchemas()
                }
            } header: {
                HStack {
                    Text("已启用方案")
                    Spacer()
                    Button {
                        showImportSheet = true
                    } label: {
                        Image(systemName: "plus")
                    }
                    .help("导入方案")
                }
            }
        }
        .listStyle(.inset(alternatesRowBackgrounds: true))
        .frame(minWidth: 280)
    }

    private func toggleSchema(_ schema: RimeSchema) {
        if let index = schemas.firstIndex(where: { $0.id == schema.id }) {
            schemas[index].enabled.toggle()
            updateSchemas()
        }
    }

    private func moveSchema(_ schema: RimeSchema, direction: MoveDirection) {
        guard let index = schemas.firstIndex(where: { $0.id == schema.id }) else { return }
        let newIndex = direction == .up ? index - 1 : index + 1
        guard newIndex >= 0 && newIndex < schemas.count else { return }
        schemas.swapAt(index, newIndex)
        updateSchemas()
    }

    private func removeSchema(_ schema: RimeSchema) {
        schemas.removeAll { $0.id == schema.id }
        updateSchemas()
    }

    private func updateSchemas() {
        appState.configManager.schemas = schemas
        appState.hasUnsavedChanges = true
    }

    private func save() {
        do {
            try appState.configManager.saveSchemas(schemas)
            appState.hasUnsavedChanges = false
            showSaved = true
            DispatchQueue.main.asyncAfter(deadline: .now() + 2) {
                showSaved = false
            }
        } catch {
            print("Error saving schemas: \(error)")
        }
    }

    private func handleImport(_ result: Result<[URL], Error>) {
        guard let urls = try? result.get(),
              let url = urls.first else { return }

        guard url.startAccessingSecurityScopedResource() else { return }
        defer { url.stopAccessingSecurityScopedResource() }

        if let content = try? String(contentsOf: url, encoding: .utf8),
           let dict = try? RimeConfigStore.parseYAML(content) {
            let source = (dict["patch"] as? [String: Any]) ?? dict
            if let schemaList = source["schema_list"] as? [[String: Any]] {
                for item in schemaList {
                    if let schemaId = item["schema"] as? String {
                        if !schemas.contains(where: { $0.schemaId == schemaId }) {
                            schemas.append(RimeSchema(schemaId: schemaId, enabled: true))
                        }
                    }
                }
                updateSchemas()
            }
        }
    }

    private func exportSchemas() {
        let panel = NSSavePanel()
        panel.allowedContentTypes = [.yaml, .plainText]
        panel.nameFieldStringValue = "default.custom.yaml"

        panel.begin { response in
            if response == .OK, let url = panel.url {
                let patch: [String: Any] = [
                    "schema_list": schemas.filter(\.enabled).map { ["schema": $0.schemaId] }
                ]
                let content = (try? RimeConfigStore.dumpYAML(["patch": patch])) ?? "patch:\n"
                try? content.write(to: url, atomically: true, encoding: .utf8)
            }
        }
    }
}

enum MoveDirection {
    case up, down
}

struct SchemaRowView: View {
    let schema: RimeSchema

    var body: some View {
        HStack(spacing: 10) {
            Image(systemName: schema.enabled ? "checkmark.circle.fill" : "circle.dashed")
                .foregroundColor(schema.enabled ? .green : .secondary)
                .font(.system(size: 16))

            VStack(alignment: .leading, spacing: 2) {
                Text(schema.schemaId)
                    .font(.system(.body, design: .monospaced))
                if !schema.name.isEmpty {
                    Text(schema.name)
                        .font(.caption)
                        .foregroundColor(.secondary)
                }
            }

            Spacer()

            if !schema.version.isEmpty {
                Text("v\(schema.version)")
                    .font(.caption2)
                    .foregroundColor(.secondary)
            }

            if !schema.dependencies.isEmpty {
                HStack(spacing: 2) {
                    Image(systemName: "arrow.triangle.2.circlepath")
                        .font(.caption2)
                    Text("\(schema.dependencies.count)")
                        .font(.caption2)
                }
                .foregroundColor(.secondary)
            }
        }
        .padding(.vertical, 4)
        .opacity(schema.enabled ? 1.0 : 0.6)
    }
}
