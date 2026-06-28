import SwiftUI
import AppKit

/// A view that captures keyboard shortcut combinations (e.g., Control+p, Shift+Up).
/// Draws as a rounded text-field-like rectangle. On click it becomes first responder;
/// the next key-down event is captured, translated to a Rime-compatible string, and
/// written back to SwiftUI. Escape cancels capture.
struct KeyCaptureField: NSViewRepresentable {
    @Binding var text: String
    var placeholder: String = ""

    func makeNSView(context: Context) -> KeyCaptureView {
        let view = KeyCaptureView()
        view.placeholder = placeholder
        view.displayedText = text
        view.onKeyCapture = { combo in
            text = combo
        }
        return view
    }

    func updateNSView(_ nsView: KeyCaptureView, context: Context) {
        nsView.displayedText = text
        nsView.placeholder = placeholder
        nsView.needsDisplay = true
    }
}

final class KeyCaptureView: NSView {
    var onKeyCapture: ((String) -> Void)?
    var placeholder: String = ""
    var displayedText: String = ""

    private var isActive = false

    override var acceptsFirstResponder: Bool { true }

    override func mouseDown(with event: NSEvent) {
        if isActive {
            deactivate()
        } else {
            activate()
        }
    }

    override func keyDown(with event: NSEvent) {
        guard isActive else {
            super.keyDown(with: event)
            return
        }

        if event.keyCode == 53 {
            deactivate()
            return
        }

        let combo = KeyCaptureField.buildKeyCombo(from: event)
        displayedText = combo
        onKeyCapture?(combo)
        deactivate()
    }

    override func flagsChanged(with event: NSEvent) {
        guard isActive else {
            super.flagsChanged(with: event)
            return
        }

        let modifierPreview = KeyCaptureField.modifierNames(from: event.modifierFlags).joined(separator: "+")
        if !modifierPreview.isEmpty {
            displayedText = modifierPreview
            needsDisplay = true
        }
    }

    override func resignFirstResponder() -> Bool {
        deactivate()
        return true
    }

    private func activate() {
        isActive = true
        needsDisplay = true
        NSApp.activate(ignoringOtherApps: true)
        window?.makeKey()
        window?.makeFirstResponder(self)
    }

    private func deactivate() {
        guard isActive else { return }
        isActive = false
        needsDisplay = true
    }

    override func viewDidMoveToWindow() {
        super.viewDidMoveToWindow()
        if window == nil {
            deactivate()
        }
    }

    override func draw(_ dirtyRect: NSRect) {
        let bounds = self.bounds
        let cornerRadius: CGFloat = 4
        let path = NSBezierPath(roundedRect: bounds.insetBy(dx: 1, dy: 1),
                                 xRadius: cornerRadius, yRadius: cornerRadius)

        if isActive {
            NSColor.controlAccentColor.withAlphaComponent(0.15).setFill()
        } else {
            NSColor.textBackgroundColor.setFill()
        }
        path.fill()

        if isActive {
            NSColor.controlAccentColor.setStroke()
        } else {
            NSColor.separatorColor.setStroke()
        }
        path.lineWidth = 1
        path.stroke()

        let textRect = bounds.insetBy(dx: 8, dy: 3)
        if displayedText.isEmpty {
            let attrs: [NSAttributedString.Key: Any] = [
                .foregroundColor: NSColor.placeholderTextColor,
                .font: NSFont.systemFont(ofSize: NSFont.systemFontSize)
            ]
            placeholder.draw(in: textRect, withAttributes: attrs)
        } else {
            let attrs: [NSAttributedString.Key: Any] = [
                .foregroundColor: NSColor.textColor,
                .font: NSFont.systemFont(ofSize: NSFont.systemFontSize)
            ]
            displayedText.draw(in: textRect, withAttributes: attrs)
        }
    }

    override var intrinsicContentSize: NSSize {
        NSSize(width: 180, height: 24)
    }
}

extension KeyCaptureField {

    static func buildKeyCombo(from event: NSEvent) -> String {
        var parts = modifierNames(from: event.modifierFlags)
        parts.append(keyNameForEvent(event))
        return parts.joined(separator: "+")
    }

    static func modifierNames(from modifiers: NSEvent.ModifierFlags) -> [String] {
        var parts: [String] = []
        if modifiers.contains(.control) { parts.append("Control") }
        if modifiers.contains(.option) { parts.append("Option") }
        if modifiers.contains(.shift) { parts.append("Shift") }
        if modifiers.contains(.command) { parts.append("Command") }
        return parts
    }

    private static func keyNameForEvent(_ event: NSEvent) -> String {
        if let special = event.specialKey {
            let name = specialKeyName(special)
            if !name.isEmpty { return name }
        }

        if let chars = event.charactersIgnoringModifiers?.lowercased(),
           let first = chars.first,
           first.isASCII {
            return String(first)
        }

        return keyCodeName(Int(event.keyCode))
    }

    private static func specialKeyName(_ key: NSEvent.SpecialKey) -> String {
        switch key {
        case .upArrow: return "Up"
        case .downArrow: return "Down"
        case .leftArrow: return "Left"
        case .rightArrow: return "Right"
        case .f1: return "F1"
        case .f2: return "F2"
        case .f3: return "F3"
        case .f4: return "F4"
        case .f5: return "F5"
        case .f6: return "F6"
        case .f7: return "F7"
        case .f8: return "F8"
        case .f9: return "F9"
        case .f10: return "F10"
        case .f11: return "F11"
        case .f12: return "F12"
        case .pageUp: return "Page_Up"
        case .pageDown: return "Page_Down"
        case .home: return "Home"
        case .end: return "End"
        case .delete: return "Delete"
        case .tab: return "Tab"
        default: return ""
        }
    }

    private static func keyCodeName(_ code: Int) -> String {
        switch code {
        case 0:  return "a";       case 1:  return "s"
        case 2:  return "d";       case 3:  return "f"
        case 4:  return "h";       case 5:  return "g"
        case 6:  return "z";       case 7:  return "x"
        case 8:  return "c";       case 9:  return "v"
        case 11: return "b";       case 12: return "q"
        case 13: return "w";       case 14: return "e"
        case 15: return "r";       case 16: return "y"
        case 17: return "t";       case 18: return "1"
        case 19: return "2";       case 20: return "3"
        case 21: return "4";       case 22: return "6"
        case 23: return "5";       case 24: return "equal"
        case 25: return "9";       case 26: return "7"
        case 27: return "minus";   case 28: return "8"
        case 29: return "0";       case 30: return "bracketright"
        case 31: return "o";       case 32: return "u"
        case 33: return "bracketleft"; case 34: return "i"
        case 35: return "p";       case 37: return "l"
        case 38: return "j";       case 39: return "apostrophe"
        case 40: return "k";       case 41: return "semicolon"
        case 42: return "backslash"; case 43: return "comma"
        case 44: return "slash";   case 45: return "n"
        case 46: return "m";       case 47: return "period"
        case 50: return "grave";   case 48: return "Tab"
        case 49: return "space";   case 51: return "BackSpace"
        case 36: return "Return";  case 53: return "Escape"
        case 114: return "Help";   case 115: return "Home"
        case 116: return "Page_Up"; case 117: return "Delete"
        case 118: return "F4";     case 119: return "End"
        case 120: return "F2";     case 121: return "Page_Down"
        case 122: return "F1";     case 123: return "Left"
        case 124: return "Right";  case 125: return "Down"
        case 126: return "Up"
        default: return "key_\(code)"
        }
    }
}
