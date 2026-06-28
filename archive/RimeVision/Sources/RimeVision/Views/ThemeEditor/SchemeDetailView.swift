import SwiftUI

public enum SchemeDetailLayout {
    public static let sheetMinWidth: CGFloat = 680
    public static let sheetIdealWidth: CGFloat = 760
    public static let sheetMinHeight: CGFloat = 720
    public static let labelWidth: CGFloat = 96
}

struct SchemeDetailView: View {
    @Environment(\.dismiss) private var dismiss
    let scheme: RimeColorScheme
    let isNew: Bool
    let onSave: (RimeColorScheme) -> Void

    @State private var draftScheme: RimeColorScheme
    @State private var editName: String
    @State private var editAuthor: String

    init(scheme: RimeColorScheme, isNew: Bool, onSave: @escaping (RimeColorScheme) -> Void) {
        self.scheme = scheme
        self.isNew = isNew
        self.onSave = onSave
        _draftScheme = State(initialValue: scheme)
        _editName = State(initialValue: scheme.name)
        _editAuthor = State(initialValue: scheme.author)
    }

    var body: some View {
        VStack(spacing: 0) {
            header
            Divider()
            ScrollView {
                Form {
                    Section("基本信息") {
                        HStack {
                            Text("名称:")
                                .frame(width: SchemeDetailLayout.labelWidth, alignment: .trailing)
                            TextField("scheme_name", text: $editName)
                                .textFieldStyle(.roundedBorder)
                                .disabled(!isNew)
                                .frame(maxWidth: .infinity)
                        }
                        HStack {
                            Text("作者:")
                                .frame(width: SchemeDetailLayout.labelWidth, alignment: .trailing)
                            TextField("Author", text: $editAuthor)
                                .textFieldStyle(.roundedBorder)
                                .frame(maxWidth: .infinity)
                        }
                    }

                    Section("背景") {
                        RimeColorPicker(color: $draftScheme.backColor, label: "背景色")
                        RimeColorPicker(color: $draftScheme.borderColor, label: "边框色")
                        RimeColorPicker(color: $draftScheme.preeditBackColor, label: "预编辑背景")
                        RimeColorPicker(color: $draftScheme.candidateBackColor, label: "候选背景")
                    }

                    Section("文字") {
                        RimeColorPicker(color: $draftScheme.textColor, label: "文字色")
                        RimeColorPicker(color: $draftScheme.hilitedTextColor, label: "高亮文字色")
                    }

                    Section("候选词") {
                        RimeColorPicker(color: $draftScheme.candidateTextColor, label: "候选文字色")
                        RimeColorPicker(color: $draftScheme.hilitedCandidateTextColor, label: "选中文字色")
                        RimeColorPicker(color: $draftScheme.hilitedCandidateBackColor, label: "选中背景色")
                    }

                    Section("序号") {
                        RimeColorPicker(color: $draftScheme.candidateLabelColor, label: "序号色")
                        RimeColorPicker(color: $draftScheme.hilitedCandidateLabelColor, label: "选中序号色")
                    }

                    Section("注释") {
                        RimeColorPicker(color: $draftScheme.commentTextColor, label: "注释色")
                        RimeColorPicker(color: $draftScheme.hilitedCommentTextColor, label: "选中注释色")
                    }

                    Section("高亮背景") {
                        RimeColorPicker(color: $draftScheme.hilitedBackColor, label: "高亮背景色")
                    }
                }
                .formStyle(.grouped)
                .padding()
            }
        }
        .frame(
            minWidth: SchemeDetailLayout.sheetMinWidth,
            idealWidth: SchemeDetailLayout.sheetIdealWidth,
            minHeight: SchemeDetailLayout.sheetMinHeight
        )
    }

    private var header: some View {
        HStack {
            Button("取消") {
                dismiss()
            }

            Spacer()

            Text(isNew ? "新建主题" : "编辑主题")
                .font(.headline)

            Spacer()

            Button("保存") {
                draftScheme.name = editName
                draftScheme.author = editAuthor
                onSave(draftScheme)
                dismiss()
            }
            .buttonStyle(.borderedProminent)
            .disabled(editName.isEmpty)
        }
        .padding()
    }
}
