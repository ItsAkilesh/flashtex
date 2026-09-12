// Lists on-screen windows owned by a process via CGWindowListCopyWindowInfo.
// Needs no Accessibility permission (window *contents* still need Screen
// Recording; this only reads ids, titles, bounds and layer).
// Usage: window_probe <pid>   -> JSON array on stdout
import CoreGraphics
import Foundation

guard CommandLine.arguments.count == 2, let pid = Int32(CommandLine.arguments[1]) else {
    FileHandle.standardError.write("usage: window_probe <pid>\n".data(using: .utf8)!)
    exit(2)
}
let options: CGWindowListOption = [.optionOnScreenOnly, .excludeDesktopElements]
let list = CGWindowListCopyWindowInfo(options, kCGNullWindowID) as? [[String: Any]] ?? []
var out: [[String: Any]] = []
for w in list {
    guard let owner = w[kCGWindowOwnerPID as String] as? Int32, owner == pid else { continue }
    let bounds = w[kCGWindowBounds as String] as? [String: Any] ?? [:]
    out.append([
        "window_id": w[kCGWindowNumber as String] as? Int ?? -1,
        "owner_name": w[kCGWindowOwnerName as String] as? String ?? "",
        "title": w[kCGWindowName as String] as? String ?? "",
        "layer": w[kCGWindowLayer as String] as? Int ?? -1,
        "alpha": w[kCGWindowAlpha as String] as? Double ?? -1,
        "bounds": bounds,
    ])
}
let data = try! JSONSerialization.data(withJSONObject: out, options: [.sortedKeys])
FileHandle.standardOutput.write(data)
FileHandle.standardOutput.write("\n".data(using: .utf8)!)
