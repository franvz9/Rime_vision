import SwiftUI

struct StyleEditorView: View {
    @EnvironmentObject var appState: AppState

    private var style: Binding<RimeStyle> {
        Binding(
            get: { appState.configManager.squirrelStyle },
            set: {
                appState.configManager.squirrelStyle = $0
                appState.hasUnsavedChanges = true
            }
        )
    }

    var body: some View {
        Form {
            Section("布局") {
                Picker("候选排列", selection: style.candidateListLayout) {
                    Text("垂直 (stacked)").tag("stacked")
                    Text("水平 (linear)").tag("linear")
                    Text("表格 (tabled)").tag("tabled")
                }

                Picker("文字方向", selection: style.textOrientation) {
                    Text("水平").tag("horizontal")
                    Text("垂直").tag("vertical")
                }

                Toggle("内嵌预编辑", isOn: style.inlinePreedit)
                Toggle("内嵌候选词", isOn: style.inlineCandidate)
            }

            Section("外观") {
                StyleSlider(label: "圆角半径", value: style.cornerRadius, range: 0...30)
                StyleSlider(label: "高亮圆角", value: style.hilitedCornerRadius, range: 0...20)
                StyleSlider(label: "行间距", value: style.lineSpacing, range: 0...20)
                StyleSlider(label: "间距", value: style.spacing, range: 0...30)
                StyleSlider(label: "阴影", value: style.shadowSize, range: 0...20)
                StyleSlider(label: "透明度", value: style.alpha, range: 0...1, step: 0.05, format: "%.2f")

                Toggle("磨砂效果", isOn: style.translucency)
                Toggle("色不叠加", isOn: style.mutualExclusive)
            }

            Section("字体") {
                HStack {
                    Text("主字体:")
                    TextField("PingFang SC", text: style.fontFace)
                        .textFieldStyle(.roundedBorder)
                }

                StyleSlider(label: "字号", value: style.fontPoint, range: 10...30)

                HStack {
                    Text("标签字体:")
                    TextField("Lucida Grande", text: style.labelFontFace)
                        .textFieldStyle(.roundedBorder)
                }

                StyleSlider(label: "标签字号", value: style.labelFontPoint, range: 8...24)

                HStack {
                    Text("注释字体:")
                    TextField("PingFang SC", text: style.commentFontFace)
                        .textFieldStyle(.roundedBorder)
                }

                StyleSlider(label: "注释字号", value: style.commentFontPoint, range: 8...24)
            }

            Section("格式") {
                HStack {
                    Text("候选格式:")
                    TextField("[label]. [candidate] [comment]", text: style.candidateFormat)
                        .textFieldStyle(.roundedBorder)
                }

                HStack {
                    Text("状态消息:")
                    Picker("", selection: style.statusMessageType) {
                        Text("混合").tag("mix")
                        Text("长消息").tag("long")
                        Text("短消息").tag("short")
                    }
                    .labelsHidden()
                }
            }
        }
        .formStyle(.grouped)
        .padding()
    }
}

struct StyleSlider: View {
    let label: String
    @Binding var value: Double
    var range: ClosedRange<Double> = 0...100
    var step: Double = 1
    var format: String = "%.0f"

    var body: some View {
        HStack {
            Text(label + ":")
                .frame(width: 80, alignment: .trailing)
            Slider(value: $value, in: range, step: step)
            Text(String(format: format, value))
                .font(.system(.caption, design: .monospaced))
                .frame(width: 40, alignment: .trailing)
        }
    }
}
