import SwiftUI

struct CandidatePreviewView: View {
    @EnvironmentObject var appState: AppState
    @State private var preeditText: String = "nihao"
    @State private var selectedIndex: Int = 0
    @State private var showDarkPreview = false

    private var currentScheme: RimeColorScheme? {
        let name = showDarkPreview ? appState.selectedDarkScheme : appState.selectedLightScheme
        let schemes = showDarkPreview
            ? appState.configManager.darkColorSchemes
            : appState.configManager.colorSchemes
        return schemes[name]
    }

    private var sampleCandidates: [(label: String, text: String, comment: String)] {
        [
            ("1", "你好", "nǐ hǎo"),
            ("2", "拟好", "nǐ hǎo"),
            ("3", "尼豪", "ní háo"),
            ("4", "妮好", "nī hǎo"),
            ("5", "呢耗", "ne hào"),
            ("6", "泥毫", "ní háo"),
        ]
    }

    var body: some View {
        VStack(spacing: 20) {
            HStack {
                Text("候选窗口预览")
                    .font(.headline)
                Spacer()
                Toggle("暗色预览", isOn: $showDarkPreview)
                    .toggleStyle(.switch)
                    .controlSize(.small)
            }
            .padding(.top)

            ZStack {
                if showDarkPreview {
                    Color(NSColor.windowBackgroundColor)
                        .opacity(0.3)
                } else {
                    Color(NSColor.textBackgroundColor)
                }

                if let scheme = currentScheme {
                    CandidatePanel(
                        scheme: scheme,
                        style: appState.configManager.squirrelStyle,
                        preedit: preeditText,
                        candidates: sampleCandidates,
                        selectedIndex: selectedIndex
                    )
                    .padding()
                } else {
                    Text("选择一个主题查看预览")
                        .foregroundColor(.secondary)
                }
            }
            .frame(maxWidth: .infinity, minHeight: 250)
            .cornerRadius(8)
            .overlay(
                RoundedRectangle(cornerRadius: 8)
                    .stroke(Color.gray.opacity(0.2), lineWidth: 1)
            )

            previewControls
        }
        .padding()
    }

    private var previewControls: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("预览控制")
                .font(.headline)

            HStack {
                Text("输入码:")
                TextField("nihao", text: $preeditText)
                    .textFieldStyle(.roundedBorder)
                    .frame(maxWidth: 200)
            }

            HStack {
                Text("选中:")
                Picker("", selection: $selectedIndex) {
                    ForEach(0..<sampleCandidates.count, id: \.self) { i in
                        Text("\(i+1). \(sampleCandidates[i].text)").tag(i)
                    }
                }
                .labelsHidden()
                .frame(maxWidth: 250)
            }
        }
        .padding()
        .background(Color(NSColor.controlBackgroundColor))
        .cornerRadius(8)
    }
}

struct CandidatePanel: View {
    let scheme: RimeColorScheme
    let style: RimeStyle
    let preedit: String
    let candidates: [(label: String, text: String, comment: String)]
    let selectedIndex: Int

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            if !preedit.isEmpty && !style.inlinePreedit {
                HStack {
                    Text(preedit)
                        .font(.system(size: CGFloat(style.fontPoint)))
                        .foregroundColor(scheme.textColor?.swiftUIColor ?? Color.primary)
                    Spacer()
                }
                .padding(.horizontal, 12)
                .padding(.top, 8)
                .padding(.bottom, 4)
            }

            if preedit.isEmpty && style.inlinePreedit {
                HStack {
                    Text("nihao")
                        .font(.system(size: CGFloat(style.fontPoint)))
                        .foregroundColor(scheme.textColor?.swiftUIColor ?? Color.secondary)
                    Spacer()
                }
                .padding(.horizontal, 12)
                .padding(.top, 8)
                .padding(.bottom, 4)
            }

            VStack(alignment: .leading, spacing: CGFloat(style.lineSpacing)) {
                ForEach(Array(candidates.enumerated()), id: \.offset) { index, candidate in
                    HStack(spacing: 6) {
                        Text(candidate.label)
                            .font(.system(size: CGFloat(style.labelFontPoint)))
                            .foregroundColor(
                                index == selectedIndex
                                    ? (scheme.hilitedCandidateLabelColor?.swiftUIColor ?? Color.white)
                                    : (scheme.candidateLabelColor?.swiftUIColor ?? Color.secondary)
                            )
                            .frame(width: 20, alignment: .leading)

                        Text(candidate.text)
                            .font(.system(size: CGFloat(style.fontPoint)))
                            .foregroundColor(
                                index == selectedIndex
                                    ? (scheme.hilitedCandidateTextColor?.swiftUIColor ?? Color.white)
                                    : (scheme.candidateTextColor?.swiftUIColor ?? Color.primary)
                            )

                        if !candidate.comment.isEmpty {
                            Text(candidate.comment)
                                .font(.system(size: CGFloat(style.commentFontPoint)))
                                .foregroundColor(
                                    index == selectedIndex
                                        ? (scheme.hilitedCommentTextColor?.swiftUIColor ?? Color.white)
                                        : (scheme.commentTextColor?.swiftUIColor ?? Color.secondary)
                                )
                        }
                    }
                    .padding(.horizontal, 12)
                    .padding(.vertical, 4)
                    .background(
                        index == selectedIndex
                            ? (scheme.hilitedCandidateBackColor?.swiftUIColor ?? Color.accentColor)
                            : Color.clear
                    )
                    .cornerRadius(index == selectedIndex ? style.hilitedCornerRadius : 0)
                }
            }
            .padding(.vertical, 8)
        }
        .background(scheme.backColor?.swiftUIColor ?? Color(NSColor.windowBackgroundColor))
        .cornerRadius(style.cornerRadius)
        .shadow(color: .black.opacity(0.15), radius: style.shadowSize, x: 0, y: 2)
    }
}
