import SwiftUI

struct ThemeEditorView: View {
    @EnvironmentObject var appState: AppState
    @State private var selectedTab: Tab = .light
    @State private var editingScheme: RimeColorScheme?
    @State private var isNewScheme = false
    @State private var showDeleteAlert = false
    @State private var schemeToDelete: String?

    enum Tab: String, CaseIterable {
        case light = "亮色主题"
        case dark = "暗色主题"
        case style = "样式"
    }

    var body: some View {
        VStack(spacing: 0) {
            Picker("主题", selection: $selectedTab) {
                ForEach(Tab.allCases, id: \.self) { tab in
                    Text(tab.rawValue).tag(tab)
                }
            }
            .pickerStyle(.segmented)
            .padding()

            Divider()

            HSplitView {
                VStack(spacing: 0) {
                    toolbar
                    ScrollView {
                        switch selectedTab {
                        case .light:
                            SchemeListView(
                                schemes: appState.configManager.colorSchemes,
                                selectedId: Binding(
                                    get: { appState.selectedLightScheme },
                                    set: {
                                        appState.selectedLightScheme = $0
                                        appState.hasUnsavedChanges = true
                                    }
                                ),
                                onEdit: { scheme in
                                    editingScheme = scheme
                                    isNewScheme = false
                                },
                                onCopy: { scheme in
                                    duplicateScheme(scheme)
                                },
                                onDelete: { name in
                                    schemeToDelete = name
                                    showDeleteAlert = true
                                }
                            )
                        case .dark:
                            SchemeListView(
                                schemes: appState.configManager.darkColorSchemes,
                                selectedId: Binding(
                                    get: { appState.selectedDarkScheme },
                                    set: {
                                        appState.selectedDarkScheme = $0
                                        appState.hasUnsavedChanges = true
                                    }
                                ),
                                onEdit: { scheme in
                                    editingScheme = scheme
                                    isNewScheme = false
                                },
                                onCopy: { scheme in
                                    duplicateScheme(scheme)
                                },
                                onDelete: { name in
                                    schemeToDelete = name
                                    showDeleteAlert = true
                                }
                            )
                        case .style:
                            StyleEditorView()
                        }
                    }
                }
                .frame(minWidth: 280, idealWidth: 320)

                CandidatePreviewView()
                    .frame(minWidth: 400)
            }
        }
        .sheet(item: $editingScheme) { scheme in
            SchemeDetailView(
                scheme: scheme,
                isNew: isNewScheme,
                onSave: { updated in
                    updateScheme(updated)
                    appState.hasUnsavedChanges = true
                }
            )
        }
        .alert("确认删除", isPresented: $showDeleteAlert) {
            Button("取消", role: .cancel) {}
            Button("删除", role: .destructive) {
                if let name = schemeToDelete {
                    deleteScheme(name: name)
                }
            }
        } message: {
            Text("确定要删除主题 \"\(schemeToDelete ?? "")\" 吗？")
        }
    }

    private var toolbar: some View {
        HStack {
            Text(selectedTab == .light ? "亮色主题" : selectedTab == .dark ? "暗色主题" : "样式设置")
                .font(.headline)

            Spacer()

            if selectedTab != .style {
                Button {
                    createNewScheme()
                } label: {
                    Image(systemName: "plus")
                }
                .help("新建主题")
            }
        }
        .padding(.horizontal)
        .padding(.vertical, 8)
    }

    private func createNewScheme() {
        var newScheme = RimeColorScheme(name: "custom_\(Date().timeIntervalSince1970)")
        newScheme.backColor = RimeColor(r: 255, g: 255, b: 255)
        newScheme.candidateTextColor = RimeColor(r: 0, g: 0, b: 0)
        newScheme.hilitedCandidateBackColor = RimeColor(r: 0, g: 122, b: 255)
        newScheme.hilitedCandidateTextColor = RimeColor(r: 255, g: 255, b: 255)

        editingScheme = newScheme
        isNewScheme = true
    }

    private func duplicateScheme(_ scheme: RimeColorScheme) {
        let existingNames = selectedTab == .light
            ? appState.configManager.colorSchemes.keys
            : appState.configManager.darkColorSchemes.keys
        var copied = scheme
        copied.name = SchemeCopyNaming.uniqueCopyName(for: scheme.name, existingNames: existingNames)
        editingScheme = copied
        isNewScheme = true
    }

    private func updateScheme(_ scheme: RimeColorScheme) {
        if selectedTab == .light {
            appState.configManager.colorSchemes[scheme.name] = scheme
        } else {
            appState.configManager.darkColorSchemes[scheme.name] = scheme
        }
    }

    private func deleteScheme(name: String) {
        if selectedTab == .light {
            appState.configManager.colorSchemes.removeValue(forKey: name)
            if appState.selectedLightScheme == name {
                appState.selectedLightScheme = "native"
            }
        } else {
            appState.configManager.darkColorSchemes.removeValue(forKey: name)
            if appState.selectedDarkScheme == name {
                appState.selectedDarkScheme = "native"
            }
        }
        appState.hasUnsavedChanges = true
    }
}

struct SchemeListView: View {
    let schemes: [String: RimeColorScheme]
    @Binding var selectedId: String
    let onEdit: (RimeColorScheme) -> Void
    let onCopy: (RimeColorScheme) -> Void
    let onDelete: (String) -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            ForEach(Array(schemes.keys.sorted()), id: \.self) { key in
                if let scheme = schemes[key] {
                    SchemeRow(
                        scheme: scheme,
                        isSelected: selectedId == key,
                        onSelect: { selectedId = key },
                        onEdit: { onEdit(scheme) },
                        onCopy: { onCopy(scheme) },
                        onDelete: { onDelete(key) }
                    )
                }
            }
        }
        .padding()
    }
}

struct SchemeRow: View {
    let scheme: RimeColorScheme
    let isSelected: Bool
    let onSelect: () -> Void
    let onEdit: () -> Void
    let onCopy: () -> Void
    let onDelete: () -> Void

    @State private var isHovering = false

    var body: some View {
        HStack(spacing: 10) {
            RoundedRectangle(cornerRadius: 4)
                .fill(scheme.backColor?.swiftUIColor ?? Color.gray)
                .frame(width: 36, height: 24)
                .overlay(
                    RoundedRectangle(cornerRadius: 4)
                        .stroke(isSelected ? Color.accentColor : Color.clear, lineWidth: 2)
                )

            VStack(alignment: .leading, spacing: 2) {
                Text(scheme.name)
                    .font(.system(size: 13, weight: .medium))
                    .foregroundColor(.primary)
                if !scheme.author.isEmpty {
                    Text(scheme.author)
                        .font(.system(size: 11))
                        .foregroundColor(.secondary)
                }
            }

            Spacer()

            if isHovering {
                HStack(spacing: 4) {
                    Button {
                        onEdit()
                    } label: {
                        Image(systemName: "pencil")
                            .font(.system(size: 11))
                    }
                    .buttonStyle(.plain)
                    .help("编辑")

                    Button {
                        onCopy()
                    } label: {
                        Image(systemName: "doc.on.doc")
                            .font(.system(size: 11))
                    }
                    .buttonStyle(.plain)
                    .help("复制")

                    Button {
                        onDelete()
                    } label: {
                        Image(systemName: "trash")
                            .font(.system(size: 11))
                            .foregroundColor(.red)
                    }
                    .buttonStyle(.plain)
                    .help("删除")
                }
            }

            if isSelected {
                Image(systemName: "checkmark.circle.fill")
                    .foregroundColor(.accentColor)
            }
        }
        .padding(.vertical, 6)
        .padding(.horizontal, 8)
        .background(
            RoundedRectangle(cornerRadius: 6)
                .fill(isSelected ? Color.accentColor.opacity(0.1) : Color.clear)
        )
        .contentShape(Rectangle())
        .onTapGesture { onSelect() }
        .onHover { isHovering = $0 }
    }
}
