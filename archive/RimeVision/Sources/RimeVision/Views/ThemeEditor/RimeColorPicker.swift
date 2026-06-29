import SwiftUI

struct RimeColorPicker: View {
    @Binding var color: RimeColor?
    let label: String
    @State private var isEditing = false
    @State private var hexInput: String = ""

    var body: some View {
        HStack(spacing: 8) {
            Text(label)
                .frame(width: 120, alignment: .trailing)
                .font(.system(size: 12))

            if let currentColor = color {
                RoundedRectangle(cornerRadius: 4)
                    .fill(currentColor.swiftUIColor)
                    .frame(width: 28, height: 20)
                    .overlay(
                        RoundedRectangle(cornerRadius: 4)
                            .stroke(Color.primary.opacity(0.2), lineWidth: 1)
                    )
                    .onTapGesture {
                        isEditing.toggle()
                        hexInput = currentColor.hexString
                    }
            } else {
                RoundedRectangle(cornerRadius: 4)
                    .stroke(Color.gray.opacity(0.4), lineWidth: 1)
                    .frame(width: 28, height: 20)
                    .overlay(
                        Image(systemName: "plus")
                            .font(.system(size: 10))
                            .foregroundColor(.secondary)
                    )
                    .onTapGesture {
                        isEditing.toggle()
                        hexInput = "0x000000"
                    }
            }

            if isEditing {
                ColorEditPanel(color: $color, hexInput: $hexInput)
            } else {
                if let c = color {
                    Text(c.hexString)
                        .font(.system(.caption, design: .monospaced))
                        .foregroundColor(.secondary)
                        .onTapGesture {
                            isEditing = true
                            hexInput = c.hexString
                        }
                } else {
                    Text("未设置")
                        .font(.caption)
                        .foregroundColor(.secondary)
                }
            }

            if color != nil {
                Button {
                    color = nil
                } label: {
                    Image(systemName: "xmark.circle")
                        .foregroundColor(.secondary)
                }
                .buttonStyle(.plain)
            }
        }
        .padding(.vertical, 2)
    }
}

struct ColorEditPanel: View {
    @Binding var color: RimeColor?
    @Binding var hexInput: String
    @State private var red: Double = 0
    @State private var green: Double = 0
    @State private var blue: Double = 0
    @State private var alpha: Double = 255

    var body: some View {
        VStack(spacing: 6) {
            HStack(spacing: 6) {
                TextField("0xRRGGBB", text: $hexInput)
                    .textFieldStyle(.roundedBorder)
                    .font(.system(.caption, design: .monospaced))
                    .frame(width: 100)
                    .onSubmit { applyHex() }

                Button("应用") {
                    applyHex()
                }
                .controlSize(.small)
            }

            HStack(spacing: 8) {
                ColorSlider(value: $red, color: .red, label: "R")
                    .frame(width: 60)
                ColorSlider(value: $green, color: .green, label: "G")
                    .frame(width: 60)
                ColorSlider(value: $blue, color: .blue, label: "B")
                    .frame(width: 60)
                AlphaSlider(value: $alpha)
                    .frame(width: 50)
            }
            .onChange(of: red) { _ in updateFromSliders() }
            .onChange(of: green) { _ in updateFromSliders() }
            .onChange(of: blue) { _ in updateFromSliders() }
            .onChange(of: alpha) { _ in updateFromSliders() }
        }
        .padding(8)
        .background(Color(NSColor.controlBackgroundColor))
        .cornerRadius(6)
        .overlay(
            RoundedRectangle(cornerRadius: 6)
                .stroke(Color.gray.opacity(0.3), lineWidth: 1)
        )
        .onAppear {
            if let c = color {
                red = Double(c.r)
                green = Double(c.g)
                blue = Double(c.b)
                alpha = Double(c.a)
            }
        }
    }

    private func applyHex() {
        if let parsed = RimeColor.from(hex: hexInput) {
            color = parsed
            red = Double(parsed.r)
            green = Double(parsed.g)
            blue = Double(parsed.b)
            alpha = Double(parsed.a)
        }
    }

    private func updateFromSliders() {
        color = RimeColor(
            r: Int(red),
            g: Int(green),
            b: Int(blue),
            a: Int(alpha)
        )
        hexInput = color?.hexString ?? "0x000000"
    }
}

struct ColorSlider: View {
    @Binding var value: Double
    let color: Color
    let label: String

    var body: some View {
        VStack(spacing: 2) {
            Text(label)
                .font(.system(size: 9))
                .foregroundColor(.secondary)
            Slider(value: $value, in: 0...255, step: 1)
                .tint(color)
            Text("\(Int(value))")
                .font(.system(size: 9, design: .monospaced))
                .foregroundColor(.secondary)
        }
    }
}

struct AlphaSlider: View {
    @Binding var value: Double

    var body: some View {
        VStack(spacing: 2) {
            Text("A")
                .font(.system(size: 9))
                .foregroundColor(.secondary)
            Slider(value: $value, in: 0...255, step: 1)
                .tint(.white)
                .overlay(
                    RoundedRectangle(cornerRadius: 2)
                        .stroke(Color.gray.opacity(0.3), lineWidth: 1)
                )
            Text("\(Int(value))")
                .font(.system(size: 9, design: .monospaced))
                .foregroundColor(.secondary)
        }
    }
}
