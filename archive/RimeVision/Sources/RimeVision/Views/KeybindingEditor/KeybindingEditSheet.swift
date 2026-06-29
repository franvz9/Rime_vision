import SwiftUI

struct KeyBindingEditSheet: View {
    @State var binding: ConfigManager.KeyBindingItem
    let onSave: (ConfigManager.KeyBindingItem) -> Void
    let onCancel: () -> Void

    @State private var when: String = "always"
    @State private var accept: String = ""
    @State private var actionType: String = "send"
    @State private var actionValue: String = ""

    private let whenOptions = ["always", "composing", "paging", "has_menu", "predicting"]

    var body: some View {
        VStack(spacing: 16) {
            Text("编辑快捷键")
                .font(.headline)

            VStack(spacing: 12) {
                HStack {
                    Text("触发条件:")
                        .frame(width: 80, alignment: .trailing)
                    Picker("", selection: $when) {
                        ForEach(whenOptions, id: \.self) { Text($0) }
                    }
                    .labelsHidden()
                }

                HStack {
                    Text("按键:")
                        .frame(width: 80, alignment: .trailing)
                    TextField("Control+Shift+1", text: $accept)
                        .textFieldStyle(.roundedBorder)
                }

                HStack {
                    Text("动作类型:")
                        .frame(width: 80, alignment: .trailing)
                    Picker("", selection: $actionType) {
                        Text("send").tag("send")
                        Text("toggle").tag("toggle")
                        Text("select").tag("select")
                    }
                    .labelsHidden()
                }

                HStack {
                    Text("动作值:")
                        .frame(width: 80, alignment: .trailing)
                    TextField(actionType == "send" ? "Up" : (actionType == "toggle" ? "ascii_mode" : ".next"), text: $actionValue)
                        .textFieldStyle(.roundedBorder)
                }
            }

            HStack {
                Button("取消") { onCancel() }
                Spacer()
                Button("保存") {
                    var updated = binding
                    updated.when = when
                    updated.accept = accept
                    if actionType == "send" {
                        updated.send = actionValue
                        updated.toggle = ""
                        updated.select = ""
                    } else if actionType == "toggle" {
                        updated.send = ""
                        updated.toggle = actionValue
                        updated.select = ""
                    } else {
                        updated.send = ""
                        updated.toggle = ""
                        updated.select = actionValue
                    }
                    onSave(updated)
                }
                .buttonStyle(.borderedProminent)
                .disabled(accept.isEmpty || actionValue.isEmpty)
            }
        }
        .padding()
        .frame(width: 400, height: 250)
        .onAppear {
            when = binding.when
            accept = binding.accept
            if !binding.toggle.isEmpty {
                actionType = "toggle"
                actionValue = binding.toggle
            } else if !binding.select.isEmpty {
                actionType = "select"
                actionValue = binding.select
            } else {
                actionType = "send"
                actionValue = binding.send
            }
        }
    }
}
