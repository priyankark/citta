import AppKit
import Foundation

// This process owns its window and reports only its own text/focus/window state.
// Usage: owned-fixture /private/path/state.json unique-marker
let args = CommandLine.arguments
if args.count != 3 { fputs("usage: owned-fixture STATE_JSON MARKER\n", stderr); exit(2) }
let path = URL(fileURLWithPath: args[1])
let marker = args[2]
let app = NSApplication.shared
app.setActivationPolicy(.regular)
let window = NSWindow(contentRect: NSRect(x: 150, y: 150, width: 640, height: 240),
                      styleMask: [.titled, .closable], backing: .buffered, defer: false)
window.title = "Citta owned desktop test — " + marker
window.isReleasedWhenClosed = false
let label = NSTextField(labelWithString: "Citta desktop verification: only this test field receives input.")
label.frame = NSRect(x: 20, y: 170, width: 600, height: 35)
let field = NSTextField(frame: NSRect(x: 20, y: 90, width: 600, height: 45))
field.font = NSFont.systemFont(ofSize: 18)
window.contentView?.addSubview(label)
window.contentView?.addSubview(field)
func report() {
    let responder = window.firstResponder
    let focused = (responder === field) || (field.currentEditor() != nil && responder === field.currentEditor())
    let state: [String: Any] = ["pid": ProcessInfo.processInfo.processIdentifier,
        "window_id": window.windowNumber, "marker": marker, "text": field.currentEditor()?.string ?? field.stringValue,
        "frontmost": NSWorkspace.shared.frontmostApplication?.processIdentifier == ProcessInfo.processInfo.processIdentifier,
        "field_focused": focused, "visible": window.isVisible,
        "updated_at": Date().timeIntervalSince1970]
    if let bytes = try? JSONSerialization.data(withJSONObject: state, options: [.sortedKeys]) {
        try? bytes.write(to: path, options: .atomic)
        try? FileManager.default.setAttributes([.posixPermissions: 0o600], ofItemAtPath: path.path)
    }
    if !window.isVisible { app.terminate(nil) }
}
window.makeKeyAndOrderFront(nil)
window.makeFirstResponder(field)
app.activate(ignoringOtherApps: true)
let timer = Timer.scheduledTimer(withTimeInterval: 0.1, repeats: true) { _ in report() }
report()
app.run()
