import SwiftUI

public struct SettingsView: View {
    @EnvironmentObject var appState: AppState
    @State private var showResetAlert = false

    public init() {}

    public var body: some View {
        Form {
            Section("Rime 目录") {
                HStack {
                    Text("用户目录:")
                    Text(ConfigManager.shared.rimeUserDir.path)
                        .font(.system(.body, design: .monospaced))
                        .foregroundColor(.secondary)
                    Spacer()
                    Button("打开") {
                        NSWorkspace.shared.open(ConfigManager.shared.rimeUserDir)
                    }
                }
            }

            Section("配置文件") {
                configFileRow("squirrel.yaml", isMain: true)
                configFileRow("squirrel.custom.yaml", isMain: false)
                configFileRow("default.yaml", isMain: true)
                configFileRow("default.custom.yaml", isMain: false)
            }

            Section("操作") {
                HStack {
                    Button("重新加载配置") {
                        appState.configManager.loadAll()
                        appState.selectedLightScheme = appState.configManager.squirrelStyle.colorSchemeName
                        appState.selectedDarkScheme = appState.configManager.squirrelStyle.colorSchemeDarkName
                    }

                    Button("同步用户数据") {
                        RimeDeployer.shared.sync()
                    }

                    Spacer()

                    Button("重置自定义配置", role: .destructive) {
                        showResetAlert = true
                    }
                }
            }

            Section("关于") {
                HStack {
                    Text("RimeVision")
                        .font(.headline)
                    Text("v0.1.0")
                        .foregroundColor(.secondary)
                }
                Text("Rime 输入法可视化配置工具")
                    .font(.caption)
                    .foregroundColor(.secondary)
            }
        }
        .formStyle(.grouped)
        .navigationTitle("设置")
        .alert("确认重置", isPresented: $showResetAlert) {
            Button("取消", role: .cancel) {}
            Button("重置", role: .destructive) {
                try? FileManager.default.removeItem(at: ConfigManager.shared.squirrelCustomYAML)
                try? FileManager.default.removeItem(at: ConfigManager.shared.defaultCustomYAML)
                appState.configManager.loadAll()
                appState.selectedLightScheme = appState.configManager.squirrelStyle.colorSchemeName
                appState.selectedDarkScheme = appState.configManager.squirrelStyle.colorSchemeDarkName
            }
        } message: {
            Text("将删除 squirrel.custom.yaml 和 default.custom.yaml，此操作不可撤销。")
        }
    }

    private func configFileRow(_ filename: String, isMain: Bool) -> some View {
        let url = ConfigManager.shared.rimeUserDir.appendingPathComponent(filename)
        let exists = FileManager.default.fileExists(atPath: url.path)

        return HStack {
            Image(systemName: exists ? "doc.fill" : "doc")
                .foregroundColor(exists ? .accentColor : .secondary)

            VStack(alignment: .leading) {
                Text(filename)
                    .font(.system(.body, design: .monospaced))
                if isMain {
                    Text("主配置文件")
                        .font(.caption)
                        .foregroundColor(.secondary)
                } else {
                    Text("自定义配置 (patch)")
                        .font(.caption)
                        .foregroundColor(.secondary)
                }
            }

            Spacer()

            if exists {
                Button("编辑") {
                    NSWorkspace.shared.open(url)
                }
                Button("查看") {
                    NSWorkspace.shared.selectFile(url.path, inFileViewerRootedAtPath: "")
                }
            } else {
                Text("保存设置后自动生成")
                    .font(.caption)
                    .foregroundColor(.secondary)
            }
        }
    }
}
