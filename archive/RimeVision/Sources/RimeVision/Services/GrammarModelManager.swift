import Foundation

@MainActor
final class GrammarModelManager: ObservableObject {

    // ── Published State ──
    @Published var models: [GrammarModel] = []
    /// key = schemaId，记录每个方案的 grammar 挂载状态
    @Published var mountConfigs: [String: SchemaGrammarConfig] = [:]

    let rimeUserDir: URL

    init(rimeUserDir: URL = FileManager.default.homeDirectoryForCurrentUser.appendingPathComponent("Library/Rime")) {
        self.rimeUserDir = rimeUserDir
    }

    // ── MARK: 加载 ──

    func loadAll(schemaIds: [String]) {
        scanGramFiles()
        loadAllMountConfigs(schemaIds: schemaIds)
    }

    func scanGramFiles() {
        let fm = FileManager.default
        guard let urls = try? fm.contentsOfDirectory(
            at: rimeUserDir,
            includingPropertiesForKeys: [.fileSizeKey, .contentModificationDateKey],
            options: .skipsHiddenFiles
        ) else {
            models = []
            return
        }

        models = urls.compactMap { url -> GrammarModel? in
            guard url.pathExtension == "gram" else { return nil }
            let resourceValues = try? url.resourceValues(forKeys: [.fileSizeKey, .contentModificationDateKey])
            return GrammarModel(
                filename: url.deletingPathExtension().lastPathComponent,
                fileURL: url,
                fileSize: Int64(resourceValues?.fileSize ?? 0),
                modificationDate: resourceValues?.contentModificationDate ?? Date.distantPast
            )
        }.sorted { $0.filename < $1.filename }
    }

    // ── MARK: 导入 ──

    func importModel(from sourceURL: URL) throws {
        let accessing = sourceURL.startAccessingSecurityScopedResource()
        defer { if accessing { sourceURL.stopAccessingSecurityScopedResource() } }

        let fileName = sourceURL.lastPathComponent
        let destURL = rimeUserDir.appendingPathComponent(fileName)

        // 如果目标已存在，先删除
        if FileManager.default.fileExists(atPath: destURL.path) {
            try FileManager.default.removeItem(at: destURL)
        }
        try FileManager.default.copyItem(at: sourceURL, to: destURL)
        scanGramFiles()
    }

    // ── MARK: 删除 ──

    func deleteModel(_ model: GrammarModel, schemaIds: [String]) throws {
        // 先卸载所有使用该模型的方案
        for schemaId in schemaIds {
            if let config = mountConfigs[schemaId], config.mountedModel == model.filename {
                try unmount(from: schemaId)
            }
        }
        try FileManager.default.removeItem(at: model.fileURL)
        scanGramFiles()
    }

    // ── MARK: 挂载/卸载（单方案）──

    func mount(model: GrammarModel, to schemaId: String, config: SchemaGrammarConfig) throws {
        let customURL = rimeUserDir.appendingPathComponent("\(schemaId).custom.yaml")
        try RimeConfigStore.savePatch(customURL: customURL) { patch in
            // 写入 grammar/* 键
            patch["grammar/language"] = model.filename
            patch["grammar/collocation_max_length"] = config.collocationMaxLength
            patch["grammar/collocation_min_length"] = config.collocationMinLength
            patch["grammar/collocation_penalty"] = config.collocationPenalty
            patch["grammar/non_collocation_penalty"] = config.nonCollocationPenalty
            patch["grammar/weak_collocation_penalty"] = config.weakCollocationPenalty
            patch["grammar/rear_penalty"] = config.rearPenalty
            // 写入 translator/* 键
            patch["translator/contextual_suggestions"] = config.contextualSuggestions
            patch["translator/max_homophones"] = config.maxHomophones
            patch["translator/max_homographs"] = config.maxHomographs
        }
        loadMountConfig(for: schemaId)
    }

    func unmount(from schemaId: String) throws {
        let customURL = rimeUserDir.appendingPathComponent("\(schemaId).custom.yaml")
        try RimeConfigStore.savePatch(customURL: customURL) { patch in
            // 移除整个 grammar section
            // 使用 expandedPatch 展开后找到 grammar 键
            let expanded = RimePatch.expandedPatch(patch)

            // 从原始 patch 中移除 grammar 相关键
            let keysToRemove = patch.keys.filter { key in
                key == "grammar" || key.hasPrefix("grammar/")
            }
            for key in keysToRemove {
                patch.removeValue(forKey: key)
            }

            // 同时检查展开后的 grammar 键
            if expanded["grammar"] != nil {
                // 移除嵌套的 grammar 字典
                patch.removeValue(forKey: "grammar")
            }

            // 精准移除 translator 中与 grammar 相关的 3 个键
            if var translator = patch["translator"] as? [String: Any] {
                translator.removeValue(forKey: "contextual_suggestions")
                translator.removeValue(forKey: "max_homophones")
                translator.removeValue(forKey: "max_homographs")
                if translator.isEmpty {
                    patch.removeValue(forKey: "translator")
                } else {
                    patch["translator"] = translator
                }
            }

            // 处理 translator/ 前缀的 slash-path 键
            let translatorKeysToRemove = patch.keys.filter { key in
                key == "translator/contextual_suggestions"
                    || key == "translator/max_homophones"
                    || key == "translator/max_homographs"
            }
            for key in translatorKeysToRemove {
                patch.removeValue(forKey: key)
            }
        }
        loadMountConfig(for: schemaId)
    }

    // ── MARK: 批量操作 ──

    func batchMount(model: GrammarModel, to schemaIds: [String], config: SchemaGrammarConfig) throws {
        for schemaId in schemaIds {
            try mount(model: model, to: schemaId, config: config)
        }
    }

    func batchUnmount(from schemaIds: [String]) throws {
        for schemaId in schemaIds {
            try unmount(from: schemaId)
        }
    }

    // ── MARK: 配置更新（参数修改后保存）──

    func updateConfig(for schemaId: String, config: SchemaGrammarConfig) throws {
        guard let modelName = config.mountedModel,
              let grammarModel = models.first(where: { $0.filename == modelName }) else { return }
        try mount(model: grammarModel, to: schemaId, config: config)
    }

    /// 返回使用指定模型的方案数量
    func mountedSchemaCount(for model: GrammarModel) -> Int {
        mountConfigs.values.filter { $0.mountedModel == model.filename }.count
    }

    // ── MARK: 内部方法 ──

    private func loadAllMountConfigs(schemaIds: [String]) {
        for schemaId in schemaIds {
            loadMountConfig(for: schemaId)
        }
    }

    private func loadMountConfig(for schemaId: String) {
        let customURL = rimeUserDir.appendingPathComponent("\(schemaId).custom.yaml")
        guard let dict = try? RimeConfigStore.loadYAML(from: customURL),
              let patch = dict["patch"] as? [String: Any] else {
            mountConfigs[schemaId] = SchemaGrammarConfig.defaultConfig(for: schemaId)
            return
        }

        let expanded = RimePatch.expandedPatch(patch)
        let grammar = expanded["grammar"] as? [String: Any] ?? [:]
        let translator = expanded["translator"] as? [String: Any] ?? [:]

        let config = SchemaGrammarConfig(
            schemaId: schemaId,
            mountedModel: grammar["language"] as? String,
            collocationMaxLength: grammar["collocation_max_length"] as? Int ?? SchemaGrammarConfig.default.collocationMaxLength,
            collocationMinLength: grammar["collocation_min_length"] as? Int ?? SchemaGrammarConfig.default.collocationMinLength,
            collocationPenalty: grammar["collocation_penalty"] as? Int ?? SchemaGrammarConfig.default.collocationPenalty,
            nonCollocationPenalty: grammar["non_collocation_penalty"] as? Int ?? SchemaGrammarConfig.default.nonCollocationPenalty,
            weakCollocationPenalty: grammar["weak_collocation_penalty"] as? Int ?? SchemaGrammarConfig.default.weakCollocationPenalty,
            rearPenalty: grammar["rear_penalty"] as? Int ?? SchemaGrammarConfig.default.rearPenalty,
            contextualSuggestions: translator["contextual_suggestions"] as? Bool ?? true,
            maxHomophones: translator["max_homophones"] as? Int ?? SchemaGrammarConfig.default.maxHomophones,
            maxHomographs: translator["max_homographs"] as? Int ?? SchemaGrammarConfig.default.maxHomographs
        )
        mountConfigs[schemaId] = config
    }
}
