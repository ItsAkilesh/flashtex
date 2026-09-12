// Lists the on-screen windows owned by one pid as JSON:
// [{"id": N, "name": "...", "layer": L, "bounds": [x, y, w, h]}]
// Compiled on the fly by run.sh (swiftc). No Accessibility permission needed:
// CGWindowListCopyWindowInfo is the same source launch-check.sh's probe uses.
import CoreGraphics
import Foundation

guard CommandLine.arguments.count > 1, let pid = Int32(CommandLine.arguments[1]) else {
    FileHandle.standardError.write("usage: window_probe <pid>\n".data(using: .utf8)!)
    exit(2)
}
let options: CGWindowListOption = [.optionOnScreenOnly, .excludeDesktopElements]
guard let list = CGWindowListCopyWindowInfo(options, kCGNullWindowID) as? [[String: AnyObject]] else {
    print("[]")
    exit(0)
}
var out: [[String: Any]] = []
for entry in list where (entry[kCGWindowOwnerPID as String] as? Int32) == pid {
    var bounds: [Double] = []
    if let b = entry[kCGWindowBounds as String] as? [String: Double] {
        bounds = [b["X"] ?? 0, b["Y"] ?? 0, b["Width"] ?? 0, b["Height"] ?? 0]
    }
    out.append(["id": entry[kCGWindowNumber as String] as? Int ?? 0,
                "name": entry[kCGWindowName as String] as? String ?? "",
                "layer": entry[kCGWindowLayer as String] as? Int ?? 0,
                "bounds": bounds])
}
let data = try! JSONSerialization.data(withJSONObject: out, options: [.sortedKeys])
print(String(data: data, encoding: .utf8)!)
