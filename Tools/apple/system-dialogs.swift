// Prints how many dialogs of the system's own are on this Mac's screen: the windows of
// CoreServicesUIAgent, which asks "are you sure you want to open it?" of a downloaded app and
// says "there is no application set to open the URL". It reads who owns each window and
// nothing else: no titles, no contents, and so no permission.
//
// `Tools/test.sh ui` asks before it starts the Mac's UI suite (007 §8).
import CoreGraphics

let windows = CGWindowListCopyWindowInfo([.optionOnScreenOnly, .excludeDesktopElements], kCGNullWindowID) as? [[String: Any]] ?? []
print(windows.count(where: { $0[kCGWindowOwnerName as String] as? String == "CoreServicesUIAgent" }))
