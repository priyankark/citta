import AppKit
import CoreGraphics

func emit(_ value: [String: Any]) {
    if let data = try? JSONSerialization.data(withJSONObject: value, options: [.sortedKeys]) {
        print(String(data: data, encoding: .utf8)!)
    }
}
let front = NSWorkspace.shared.frontmostApplication
var displays = [CGDirectDisplayID](repeating: 0, count: 32)
var count: UInt32 = 0
let status = CGGetActiveDisplayList(32, &displays, &count)
let list: [[String: Any]] = displays.prefix(Int(count)).map { id in
    let b = CGDisplayBounds(id)
    return ["id": id, "x": b.origin.x, "y": b.origin.y,
            "width": b.width, "height": b.height, "main": id == CGMainDisplayID()]
}
emit(["frontmost_pid": front?.processIdentifier as Any? ?? NSNull(),
      "frontmost_bundle": front?.bundleIdentifier as Any? ?? NSNull(),
      "displays": list, "display_error": status.rawValue])
