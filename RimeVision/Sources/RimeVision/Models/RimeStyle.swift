import Foundation

struct RimeStyle: Equatable {
    var colorSchemeName: String = "native"
    var colorSchemeDarkName: String = "native"
    var statusMessageType: String = "mix"
    var candidateFormat: String = "[label]. [candidate] [comment]"
    var textOrientation: String = "horizontal"
    var inlinePreedit: Bool = true
    var inlineCandidate: Bool = false
    var translucency: Bool = false
    var mutualExclusive: Bool = false
    var memorizeSize: Bool = true
    var showPaging: Bool = false
    var candidateListLayout: String = "stacked"
    var alpha: Double = 1.0
    var cornerRadius: Double = 10
    var hilitedCornerRadius: Double = 0
    var borderHeight: Double = 0
    var borderWidth: Double = 0
    var lineSpacing: Double = 5
    var spacing: Double = 10
    var shadowSize: Double = 0
    var fontFace: String = "PingFang SC"
    var fontPoint: Double = 16
    var labelFontFace: String = "Lucida Grande"
    var labelFontPoint: Double = 16
    var commentFontFace: String = "PingFang SC"
    var commentFontPoint: Double = 14

    init() {}

    static func == (lhs: RimeStyle, rhs: RimeStyle) -> Bool {
        lhs.colorSchemeName == rhs.colorSchemeName &&
        lhs.colorSchemeDarkName == rhs.colorSchemeDarkName &&
        lhs.statusMessageType == rhs.statusMessageType &&
        lhs.candidateFormat == rhs.candidateFormat &&
        lhs.textOrientation == rhs.textOrientation &&
        lhs.inlinePreedit == rhs.inlinePreedit &&
        lhs.inlineCandidate == rhs.inlineCandidate &&
        lhs.translucency == rhs.translucency &&
        lhs.mutualExclusive == rhs.mutualExclusive &&
        lhs.memorizeSize == rhs.memorizeSize &&
        lhs.showPaging == rhs.showPaging &&
        lhs.candidateListLayout == rhs.candidateListLayout &&
        lhs.alpha == rhs.alpha &&
        lhs.cornerRadius == rhs.cornerRadius &&
        lhs.hilitedCornerRadius == rhs.hilitedCornerRadius &&
        lhs.borderHeight == rhs.borderHeight &&
        lhs.borderWidth == rhs.borderWidth &&
        lhs.lineSpacing == rhs.lineSpacing &&
        lhs.spacing == rhs.spacing &&
        lhs.shadowSize == rhs.shadowSize &&
        lhs.fontFace == rhs.fontFace &&
        lhs.fontPoint == rhs.fontPoint &&
        lhs.labelFontFace == rhs.labelFontFace &&
        lhs.labelFontPoint == rhs.labelFontPoint &&
        lhs.commentFontFace == rhs.commentFontFace &&
        lhs.commentFontPoint == rhs.commentFontPoint
    }
}
