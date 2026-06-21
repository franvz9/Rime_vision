import SwiftUI

public struct ContentView: View {
    @EnvironmentObject var appState: AppState

    public init() {}

    public var body: some View {
        NavigationSplitView {
            SidebarView()
        } detail: {
            DetailView()
        }
        .toolbar {
            ToolbarItem(placement: .automatic) {
                HStack(spacing: 12) {
                    if appState.hasUnsavedChanges {
                        Button("保存") {
                            appState.save()
                        }
                        .buttonStyle(.borderedProminent)
                    }

                    Button {
                        appState.deploy()
                    } label: {
                        if appState.isDeploying {
                            ProgressView()
                                .controlSize(.small)
                        } else {
                            Text("重新部署")
                        }
                    }
                    .disabled(appState.isDeploying)
                }
            }
        }
    }
}

struct DetailView: View {
    @EnvironmentObject var appState: AppState

    var body: some View {
        switch appState.selectedSidebar {
        case .generalSettings:
            GeneralSettingsView()
        case .themeEditor:
            ThemeEditorView()
        case .schemaManager:
            SchemaManagerView()
        case .schemaSettings:
            SchemaSettingsView()
        case .keybinding:
            KeybindingEditorView()
        case .punctuation:
            PunctuationSettingsView()
        case .settings:
            SettingsView()
        }
    }
}
