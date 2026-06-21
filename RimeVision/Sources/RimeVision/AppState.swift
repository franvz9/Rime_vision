import SwiftUI

enum SidebarItem: String, CaseIterable, Identifiable {
    case generalSettings = "通用设置"
    case themeEditor = "主题外观"
    case schemaManager = "方案管理"
    case schemaSettings = "方案配置"
    case keybinding = "快捷键"
    case punctuation = "标点符号"
    case settings = "高级设置"

    var id: String { rawValue }

    var icon: String {
        switch self {
        case .generalSettings: return "gearshape"
        case .themeEditor: return "paintbrush.fill"
        case .schemaManager: return "character.textbox"
        case .schemaSettings: return "slider.horizontal.3"
        case .keybinding: return "keyboard"
        case .punctuation: return "textformat.abc"
        case .settings: return "gearshape.fill"
        }
    }
}

@MainActor
public final class AppState: ObservableObject {
    @Published var selectedSidebar: SidebarItem = .generalSettings
    @Published var selectedLightScheme: String = "native"
    @Published var selectedDarkScheme: String = "native"
    @Published public var hasUnsavedChanges: Bool = false
    @Published public var isDeploying: Bool = false
    @Published public var configManager: ConfigManager

    public init() {
        self.configManager = ConfigManager.shared
        configManager.loadAll()
        selectedLightScheme = configManager.squirrelStyle.colorSchemeName
        selectedDarkScheme = configManager.squirrelStyle.colorSchemeDarkName
    }

    public init(configManager: ConfigManager) {
        self.configManager = configManager
        configManager.loadAll()
        selectedLightScheme = configManager.squirrelStyle.colorSchemeName
        selectedDarkScheme = configManager.squirrelStyle.colorSchemeDarkName
    }

    func save() {
        configManager.squirrelStyle.colorSchemeName = selectedLightScheme
        configManager.squirrelStyle.colorSchemeDarkName = selectedDarkScheme
        do {
            try configManager.saveSquirrelStyle()
            hasUnsavedChanges = false
        } catch {
            print("Error saving squirrel style: \(error)")
        }
    }

    public func deploy() {
        if hasUnsavedChanges {
            save()
        }
        isDeploying = true
        RimeDeployer.shared.deploy()
        DispatchQueue.main.asyncAfter(deadline: .now() + 1.5) {
            self.isDeploying = false
        }
    }

    func reload() {
        configManager.loadAll()
        selectedLightScheme = configManager.squirrelStyle.colorSchemeName
        selectedDarkScheme = configManager.squirrelStyle.colorSchemeDarkName
    }
}
