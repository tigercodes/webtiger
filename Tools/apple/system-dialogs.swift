// Prints how many dialogs of the system's own are on this Mac's screen: the windows of
// CoreServicesUIAgent, which asks "are you sure you want to open it?" of a downloaded app and
// says "there is no application set to open the URL"; and of UserNotificationCenter, which
// shows the system's alerts, "… quit unexpectedly" among them: one of those appears when a
// process of the iPhone simulator's crashes, and sits over the top middle of the screen. It
// reads who owns each window and nothing else: no titles, no contents, and so no permission.
//
// `Tools/test.sh ui` asks before it starts the Mac's UI suite (007 §8).
import CoreGraphics

let askers: Set = ["CoreServicesUIAgent", "UserNotificationCenter"]
let windows = CGWindowListCopyWindowInfo([.optionOnScreenOnly, .excludeDesktopElements], kCGNullWindowID) as? [[String: Any]] ?? []
print(windows.count(where: { askers.contains($0[kCGWindowOwnerName as String] as? String ?? "") }))
