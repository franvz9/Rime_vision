import SwiftUI

struct GeneralSettingsView: View {
    @EnvironmentObject var appState: AppState
    @State private var pageSize: Int = 6
    @State private var selectedSchema: String = ""
    @State private var enableEncoder: Bool = true
    @State private var enableSentence: Bool = true
    @State private var enableUserDict: Bool = true
    @State private var encodeCommitHistory: Bool = true
    @State private var switcherCaption: String = "〔方案切换〕"
    @State private var switcherHotkeys: String = "Control+grave,Control+Shift+grave"
    @State private var switcherFoldOptions: Bool = true
    @State private var switcherAbbreviateOptions: Bool = true
    @State private var capsLockAction: String = "commit_code"
    @State private var shiftLeftAction: String = "commit_code"
    @State private var shiftRightAction: String = "inline_ascii"
    @State private var goodOldCapsLock: Bool = true
    @State private var showSaved = false

    private let capsLockOptions = ["commit_code", "inline_ascii", "noop", "clear"]
    private let shiftOptions = ["commit_code", "inline_ascii", "noop"]

    var availableSchemas: [String] {
        appState.configManager.schemas.filter(\.enabled).map(\.schemaId)
    }

    var body: some View {
        Form {
            Section("默认输入方案") {
                Picker("当前方案:", selection: $selectedSchema) {
                    Text("不设置").tag("")
                    ForEach(availableSchemas, id: \.self) { schema in
                        Text(schema).tag(schema)
                    }
                }
            }

            Section("候选词") {
                HStack {
                    Text("每页候选词数:")
                    Stepper("\(pageSize)", value: $pageSize, in: 3...10)
                }
            }

            Section("翻译器") {
                Toggle("启用自动造词 (enable_encoder)", isOn: $enableEncoder)
                Toggle("启用自动句子输入 (enable_sentence)", isOn: $enableSentence)
                Toggle("启用用户词典 (enable_user_dict)", isOn: $enableUserDict)
                Toggle("自动编码上屏词语 (encode_commit_history)", isOn: $encodeCommitHistory)
            }

            Section("方案切换器 (switcher)") {
                HStack {
                    Text("切换标题:")
                    TextField("〔方案切换〕", text: $switcherCaption)
                        .textFieldStyle(.roundedBorder)
                }
                HStack {
                    Text("快捷键:")
                    TextField("Control+grave", text: $switcherHotkeys)
                        .textFieldStyle(.roundedBorder)
                }
                Text("macOS 默认 Control+Shift+`（反引号键，在 Esc 下方）")
                    .font(.caption)
                    .foregroundColor(.secondary)
                Toggle("折叠选项 (fold_options)", isOn: $switcherFoldOptions)
                Toggle("缩写选项 (abbreviate_options)", isOn: $switcherAbbreviateOptions)
            }

            Section("中英文切换 (ascii_composer)") {
                Toggle("经典 Caps Lock 模式", isOn: $goodOldCapsLock)
                Picker("Caps Lock:", selection: $capsLockAction) {
                    ForEach(capsLockOptions, id: \.self) { Text($0) }
                }
                Picker("左 Shift:", selection: $shiftLeftAction) {
                    ForEach(shiftOptions, id: \.self) { Text($0) }
                }
                Picker("右 Shift:", selection: $shiftRightAction) {
                    ForEach(shiftOptions, id: \.self) { Text($0) }
                }
            }

            Section {
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
            }
        }
        .formStyle(.grouped)
        .padding()
        .onAppear {
            loadFromConfig()
        }
    }

    private func loadFromConfig() {
        pageSize = appState.configManager.pageSize
        if let first = appState.configManager.schemas.first {
            selectedSchema = first.schemaId
        }

        let dict = (try? RimeConfigStore.loadEffective(
            baseURL: appState.configManager.defaultYAML,
            customURL: appState.configManager.defaultCustomYAML
        )) ?? [:]

        if !dict.isEmpty {            if let switcher = dict["switcher"] as? [String: Any] {
                if let hotkeys = switcher["hotkeys"] as? [String] {
                    switcherHotkeys = hotkeys.joined(separator: ", ")
                }
                if let caption = switcher["caption"] as? String {
                    switcherCaption = caption
                }
                if let fold = switcher["fold_options"] as? Bool {
                    switcherFoldOptions = fold
                }
                if let abbreviate = switcher["abbreviate_options"] as? Bool {
                    switcherAbbreviateOptions = abbreviate
                }
            }
            if let translator = dict["translator"] as? [String: Any] {
                if let v = translator["enable_encoder"] as? Bool { enableEncoder = v }
                if let v = translator["enable_sentence"] as? Bool { enableSentence = v }
                if let v = translator["enable_user_dict"] as? Bool { enableUserDict = v }
                if let v = translator["encode_commit_history"] as? Bool { encodeCommitHistory = v }
            }
            if let composer = dict["ascii_composer"] as? [String: Any],
               let switchKey = composer["switch_key"] as? [String: Any] {
                if let v = switchKey["Caps_Lock"] as? String { capsLockAction = v }
                if let v = switchKey["Shift_L"] as? String { shiftLeftAction = v }
                if let v = switchKey["Shift_R"] as? String { shiftRightAction = v }
            }
            if let composer = dict["ascii_composer"] as? [String: Any] {
                if let v = composer["good_old_caps_lock"] as? Bool { goodOldCapsLock = v }
            }
        }
    }

    private func save() {
        appState.configManager.pageSize = pageSize
        appState.hasUnsavedChanges = false

        let hotkeys = switcherHotkeys.split(separator: ",").map { $0.trimmingCharacters(in: .whitespaces) }

        do {
            try appState.configManager.saveGeneralSettings(
                menuPageSize: pageSize,
                translatorEncoder: enableEncoder,
                translatorSentence: enableSentence,
                translatorUserDict: enableUserDict,
                translatorCommitHistory: encodeCommitHistory,
                switcherCaption: switcherCaption,
                switcherHotkeys: hotkeys,
                switcherFold: switcherFoldOptions,
                switcherAbbreviate: switcherAbbreviateOptions,
                asciiCapsLock: capsLockAction,
                asciiShiftL: shiftLeftAction,
                asciiShiftR: shiftRightAction,
                asciiGoodOld: goodOldCapsLock
            )
            appState.hasUnsavedChanges = false
        } catch {
            print("Error saving general settings: \(error)")
        }

        showSaved = true
        DispatchQueue.main.asyncAfter(deadline: .now() + 2) {
            showSaved = false
        }
    }
}
