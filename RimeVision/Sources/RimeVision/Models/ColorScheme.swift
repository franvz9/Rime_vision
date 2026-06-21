import Foundation

struct RimeColorScheme: Identifiable, Equatable {
    var id: String { name }

    var name: String
    var author: String
    var colorSpace: String

    var backColor: RimeColor?
    var borderColor: RimeColor?
    var textColor: RimeColor?
    var hilitedTextColor: RimeColor?
    var hilitedBackColor: RimeColor?
    var hilitedCandidateBackColor: RimeColor?
    var candidateTextColor: RimeColor?
    var hilitedCandidateTextColor: RimeColor?
    var candidateLabelColor: RimeColor?
    var hilitedCandidateLabelColor: RimeColor?
    var commentTextColor: RimeColor?
    var hilitedCommentTextColor: RimeColor?
    var preeditBackColor: RimeColor?
    var candidateBackColor: RimeColor?

    var translucency: Bool?
    var mutualExclusive: Bool?
    var shadowSize: Double?
    var lineSpacing: Double?
    var alpha: Double?
    var spacing: Double?
    var candidateListLayout: String?
    var inlinePreedit: Bool?
    var candidateFormat: String?
    var cornerRadius: Double?
    var hilitedCornerRadius: Double?
    var borderWidth: Double?
    var borderHeight: Double?
    var fontFace: String?
    var fontPoint: Double?
    var labelFontFace: String?
    var labelFontPoint: Double?
    var commentFontFace: String?
    var commentFontPoint: Double?

    init(name: String, author: String = "") {
        self.name = name
        self.author = author
        self.colorSpace = "srgb"
    }

    static func == (lhs: RimeColorScheme, rhs: RimeColorScheme) -> Bool {
        lhs.name == rhs.name &&
        lhs.author == rhs.author &&
        lhs.backColor == rhs.backColor &&
        lhs.textColor == rhs.textColor &&
        lhs.hilitedCandidateBackColor == rhs.hilitedCandidateBackColor &&
        lhs.candidateTextColor == rhs.candidateTextColor &&
        lhs.hilitedCandidateTextColor == rhs.hilitedCandidateTextColor &&
        lhs.candidateLabelColor == rhs.candidateLabelColor &&
        lhs.commentTextColor == rhs.commentTextColor
    }
}

extension RimeColorScheme {
    static func from(dict: [String: Any], name: String) -> RimeColorScheme {
        var scheme = RimeColorScheme(name: name)
        scheme.author = dict["author"] as? String ?? ""
        scheme.colorSpace = dict["color_space"] as? String ?? "srgb"

        scheme.backColor = RimeColor.from(hex: dict["back_color"] as? String ?? "")
        scheme.borderColor = RimeColor.from(hex: dict["border_color"] as? String ?? "")
        scheme.textColor = RimeColor.from(hex: dict["text_color"] as? String ?? "")
        scheme.hilitedTextColor = RimeColor.from(hex: dict["hilited_text_color"] as? String ?? "")
        scheme.hilitedBackColor = RimeColor.from(hex: dict["hilited_back_color"] as? String ?? "")
        scheme.hilitedCandidateBackColor = RimeColor.from(hex: dict["hilited_candidate_back_color"] as? String ?? "")
        scheme.candidateTextColor = RimeColor.from(hex: dict["candidate_text_color"] as? String ?? "")
        scheme.hilitedCandidateTextColor = RimeColor.from(hex: dict["hilited_candidate_text_color"] as? String ?? "")
        scheme.candidateLabelColor = RimeColor.from(hex: dict["label_color"] as? String ?? "")
        scheme.hilitedCandidateLabelColor = RimeColor.from(hex: dict["hilited_candidate_label_color"] as? String ?? "")
        scheme.commentTextColor = RimeColor.from(hex: dict["comment_text_color"] as? String ?? "")
        scheme.hilitedCommentTextColor = RimeColor.from(hex: dict["hilited_comment_text_color"] as? String ?? "")
        scheme.preeditBackColor = RimeColor.from(hex: dict["preedit_back_color"] as? String ?? "")
        scheme.candidateBackColor = RimeColor.from(hex: dict["candidate_back_color"] as? String ?? "")

        if let v = dict["translucency"] as? Bool { scheme.translucency = v }
        if let v = dict["mutual_exclusive"] as? Bool { scheme.mutualExclusive = v }
        if let v = dict["shadow_size"] as? Double { scheme.shadowSize = v }
        if let v = dict["line_spacing"] as? Double { scheme.lineSpacing = v }
        if let v = dict["alpha"] as? Double { scheme.alpha = v }
        if let v = dict["spacing"] as? Double { scheme.spacing = v }
        if let v = dict["candidate_list_layout"] as? String { scheme.candidateListLayout = v }
        if let v = dict["inline_preedit"] as? Bool { scheme.inlinePreedit = v }
        if let v = dict["candidate_format"] as? String { scheme.candidateFormat = v }
        if let v = dict["corner_radius"] as? Double { scheme.cornerRadius = v }
        if let v = dict["hilited_corner_radius"] as? Double { scheme.hilitedCornerRadius = v }
        if let v = dict["border_width"] as? Double { scheme.borderWidth = v }
        if let v = dict["border_height"] as? Double { scheme.borderHeight = v }
        if let v = dict["font_face"] as? String { scheme.fontFace = v }
        if let v = dict["font_point"] as? Double { scheme.fontPoint = v }
        if let v = dict["label_font_face"] as? String { scheme.labelFontFace = v }
        if let v = dict["label_font_point"] as? Double { scheme.labelFontPoint = v }
        if let v = dict["comment_font_face"] as? String { scheme.commentFontFace = v }
        if let v = dict["comment_font_point"] as? Double { scheme.commentFontPoint = v }

        return scheme
    }

    func toDict() -> [String: Any] {
        var dict: [String: Any] = [:]
        dict["name"] = name
        dict["author"] = author
        if colorSpace != "srgb" { dict["color_space"] = colorSpace }

        if let v = backColor { dict["back_color"] = v.hexString }
        if let v = borderColor { dict["border_color"] = v.hexString }
        if let v = textColor { dict["text_color"] = v.hexString }
        if let v = hilitedTextColor { dict["hilited_text_color"] = v.hexString }
        if let v = hilitedBackColor { dict["hilited_back_color"] = v.hexString }
        if let v = hilitedCandidateBackColor { dict["hilited_candidate_back_color"] = v.hexString }
        if let v = candidateTextColor { dict["candidate_text_color"] = v.hexString }
        if let v = hilitedCandidateTextColor { dict["hilited_candidate_text_color"] = v.hexString }
        if let v = candidateLabelColor { dict["label_color"] = v.hexString }
        if let v = hilitedCandidateLabelColor { dict["hilited_candidate_label_color"] = v.hexString }
        if let v = commentTextColor { dict["comment_text_color"] = v.hexString }
        if let v = hilitedCommentTextColor { dict["hilited_comment_text_color"] = v.hexString }
        if let v = preeditBackColor { dict["preedit_back_color"] = v.hexString }
        if let v = candidateBackColor { dict["candidate_back_color"] = v.hexString }

        if let v = translucency { dict["translucency"] = v }
        if let v = mutualExclusive { dict["mutual_exclusive"] = v }
        if let v = shadowSize { dict["shadow_size"] = v }
        if let v = lineSpacing { dict["line_spacing"] = v }
        if let v = alpha { dict["alpha"] = v }
        if let v = spacing { dict["spacing"] = v }
        if let v = candidateListLayout { dict["candidate_list_layout"] = v }
        if let v = inlinePreedit { dict["inline_preedit"] = v }
        if let v = candidateFormat { dict["candidate_format"] = v }
        if let v = cornerRadius { dict["corner_radius"] = v }
        if let v = hilitedCornerRadius { dict["hilited_corner_radius"] = v }
        if let v = borderWidth { dict["border_width"] = v }
        if let v = borderHeight { dict["border_height"] = v }
        if let v = fontFace { dict["font_face"] = v }
        if let v = fontPoint { dict["font_point"] = v }
        if let v = labelFontFace { dict["label_font_face"] = v }
        if let v = labelFontPoint { dict["label_font_point"] = v }
        if let v = commentFontFace { dict["comment_font_face"] = v }
        if let v = commentFontPoint { dict["comment_font_point"] = v }

        return dict
    }
}
