import Foundation

/// 代表磁盘上的一个 .gram 语言模型文件
struct GrammarModel: Identifiable, Equatable, Hashable {
    var id: String { filename }

    /// 不含 .gram 扩展名的文件名，即 grammar/language 的值
    let filename: String
    /// 完整文件路径
    let fileURL: URL
    /// 文件大小（字节）
    let fileSize: Int64
    /// 文件最后修改日期
    let modificationDate: Date

    /// 友好显示名称
    var displayName: String {
        filename
    }

    /// 格式化的文件大小字符串
    var formattedSize: String {
        ByteCountFormatter.string(fromByteCount: fileSize, countStyle: .file)
    }
}

/// 某个方案的 grammar 挂载配置
struct SchemaGrammarConfig: Identifiable, Equatable {
    var id: String { schemaId }

    var schemaId: String
    /// 挂载的模型名（grammar/language 值），nil 表示未挂载
    var mountedModel: String?

    // grammar/* 参数
    var collocationMaxLength: Int
    var collocationMinLength: Int
    var collocationPenalty: Int
    var nonCollocationPenalty: Int
    var weakCollocationPenalty: Int
    var rearPenalty: Int

    // translator/* 参数
    var contextualSuggestions: Bool
    var maxHomophones: Int
    var maxHomographs: Int

    var isMounted: Bool { mountedModel != nil }

    /// 所有参数的默认值
    static let `default` = SchemaGrammarConfig(
        schemaId: "",
        mountedModel: nil,
        collocationMaxLength: 5,
        collocationMinLength: 2,
        collocationPenalty: -16,
        nonCollocationPenalty: -8,
        weakCollocationPenalty: -100,
        rearPenalty: -20,
        contextualSuggestions: true,
        maxHomophones: 7,
        maxHomographs: 7
    )

    /// 基于默认值创建指定 schemaId 的配置
    static func defaultConfig(for schemaId: String) -> SchemaGrammarConfig {
        var config = SchemaGrammarConfig.default
        config.schemaId = schemaId
        return config
    }

    /// 返回挂载指定模型的新配置
    func withModel(_ model: GrammarModel) -> SchemaGrammarConfig {
        var config = self
        config.mountedModel = model.filename
        return config
    }
}
