// The Mac app's own start and memory on the standard session (000 §7, 060): the two of the
// plan's budgets that are measured in an app.
//
//   swift Tools/perf/mac-app.swift <WebTiger.app> <start address> [runs]
//
// It starts the app in fake mode on the standard session (`WEBTIGER_FAKE_STANDARD=1`: five
// Spaces, two hundred Sites, forty tabs), with the one page that is alive at the start
// address, which is to be a page of the test site served on this machine: nothing else is
// fetched, and nothing of anybody's is in the app, whose store is in memory. For each run it
// says how long the app took from being started to having a window on the screen, and what
// its own process holds some seconds later; and it ends the app it started, by its number.
// `Tools/perf/mac.sh` builds the app, serves the site and runs this.
//
// It says what the engine's processes hold too (063): WebKit's, for pages, the network and
// drawing. The system starts those, and they are not the app's children. The ones that are
// the app's are told by this: they were not running before the app was started, and they
// are gone when it has ended. One that outlives the app is somebody else's, and is not
// counted. The plan's budget is for the app's own process; the engine's are said beside it.
//
// What it reads of the screen is who owns each window and how large it is, and nothing else:
// no titles and no contents, and so it needs no permission.
//
// The start it measures is of a program whose files the system has read before: a start
// after the machine has been turned on is slower, and is not measured here. The first start
// of a program that has just been built is slower too (the system looks a new program over
// before it runs it): every run is said, and the slowest, but it is the middle one that is
// held to the budget.
import CoreGraphics
import Darwin
import Foundation

/// The budgets (000 §7), for a Mac.
let startBudget = 0.5
let memoryBudget = 120.0 * 1024 * 1024
/// How long the app is left to load its page and settle before its memory is read.
let settle = 8.0

func fail(_ words: String) -> Never {
    FileHandle.standardError.write(Data((words + "\n").utf8))
    exit(2)
}

let arguments = CommandLine.arguments
guard arguments.count >= 3 else { fail("usage: swift Tools/perf/mac-app.swift <WebTiger.app> <start address> [runs]") }
let app = URL(fileURLWithPath: arguments[1])
let start = arguments[2]
let runs = arguments.count > 3 ? max(1, Int(arguments[3]) ?? 5) : 5
guard start.hasPrefix("http://127.0.0.1:") || start.hasPrefix("http://localhost:") else {
    fail("the start address is to be a page served on this machine: nothing else may be fetched")
}
guard let name = Bundle(url: app)?.executableURL else { fail("no app at \(app.path)") }

/// Whether a process has a window of its own on the screen: an ordinary one, of some size.
func hasWindow(_ pid: pid_t) -> Bool {
    let windows = CGWindowListCopyWindowInfo([.optionOnScreenOnly, .excludeDesktopElements], kCGNullWindowID) as? [[String: Any]] ?? []
    return windows.contains { window in
        guard window[kCGWindowOwnerPID as String] as? pid_t == pid, window[kCGWindowLayer as String] as? Int == 0,
              let bounds = window[kCGWindowBounds as String] as? [String: CGFloat] else { return false }
        return (bounds["Width"] ?? 0) > 200 && (bounds["Height"] ?? 0) > 200
    }
}

/// What a process holds, as the system counts it against the process: its footprint.
func footprint(_ pid: pid_t) -> Double? {
    var usage = rusage_info_v4()
    let read = withUnsafeMutablePointer(to: &usage) { pointer in
        pointer.withMemoryRebound(to: rusage_info_t?.self, capacity: 1) { proc_pid_rusage(pid, RUSAGE_INFO_V4, $0) }
    }
    return read == 0 ? Double(usage.ri_phys_footprint) : nil
}

/// The engine's processes that are running now, each by its number: WebKit's helpers, whoever
/// they work for.
func enginesProcesses() -> Set<pid_t> {
    var all = [pid_t](repeating: 0, count: 16384)
    let bytes = proc_listallpids(&all, Int32(MemoryLayout<pid_t>.size * all.count))
    var found: Set<pid_t> = []
    var path = [CChar](repeating: 0, count: 4096)
    for pid in all.prefix(max(0, Int(bytes))) where pid > 0 {
        guard proc_pidpath(pid, &path, UInt32(path.count)) > 0 else { continue }
        if (String(cString: path) as NSString).lastPathComponent.hasPrefix("com.apple.WebKit.") {
            found.insert(pid)
        }
    }
    return found
}

var starts: [Double] = []
var held: [Double] = []
var enginesHeld: [Double] = []
for run in 1 ... runs {
    let enginesBefore = enginesProcesses()
    let process = Process()
    process.executableURL = name
    var environment = ProcessInfo.processInfo.environment
    environment["WEBTIGER_FAKE"] = "1"
    environment["WEBTIGER_FAKE_STANDARD"] = "1"
    environment["WEBTIGER_FAKE_START"] = start
    process.environment = environment
    // A Mac puts back the windows an app had when it last ended: each run is to start from
    // the one window the app opens by itself.
    process.arguments = ["-ApplePersistenceIgnoreState", "YES"]
    process.standardOutput = FileHandle.nullDevice
    process.standardError = FileHandle.nullDevice
    let began = Date()
    do {
        try process.run()
    } catch {
        fail("the app would not start: \(error.localizedDescription)")
    }
    var shown: Double?
    while process.isRunning, Date().timeIntervalSince(began) < 30 {
        if hasWindow(process.processIdentifier) {
            shown = Date().timeIntervalSince(began)
            break
        }
        usleep(2000)
    }
    guard let shown else {
        process.terminate()
        fail("run \(run): the app showed no window in thirty seconds")
    }
    Thread.sleep(forTimeInterval: settle)
    let bytes = footprint(process.processIdentifier)
    // The engine's processes that began after the app, each with what it holds now.
    var enginesBegun: [pid_t: Double] = [:]
    for pid in enginesProcesses().subtracting(enginesBefore) {
        enginesBegun[pid] = footprint(pid)
    }
    process.terminate()
    process.waitUntilExit()
    guard let bytes else { fail("run \(run): the app's memory could not be read") }
    // Those of them that have gone with the app were the app's.
    var left = enginesProcesses()
    let waitedSince = Date()
    while !left.isDisjoint(with: enginesBegun.keys), Date().timeIntervalSince(waitedSince) < 5 {
        usleep(100_000)
        left = enginesProcesses()
    }
    let its = enginesBegun.filter { !left.contains($0.key) }
    let engines = its.values.reduce(0, +)
    starts.append(shown)
    held.append(bytes)
    enginesHeld.append(engines)
    let others = enginesBegun.count - its.count
    let apart = others == 0 ? "" : " (\(others) more began meanwhile and outlived the app: somebody else's)"
    let megabyte = 1024.0 * 1024.0
    let said = String(
        format: "run %d: a window on the screen after %.0f ms; the app's own process holds %.1f MB; the engine's %d processes hold %.1f MB",
        run, shown * 1000, bytes / megabyte, its.count, engines / megabyte
    )
    print(said + apart)
    Thread.sleep(forTimeInterval: 1)
}

var size = 0
sysctlbyname("hw.model", nil, &size, nil, 0)
var model = [CChar](repeating: 0, count: max(size, 1))
sysctlbyname("hw.model", &model, &size, nil, 0)
let machine = String(cString: model)
let memory = ProcessInfo.processInfo.physicalMemory / 1024 / 1024 / 1024
let middle = starts.sorted()[starts.count / 2]
let most = held.max() ?? 0
print(String(
    format: "the standard session, on %@ with %d GB: a window after %.0f ms (the middle of %d, from %.0f to %.0f; %.0f allowed); the app's own process holds %.1f MB at most (%.0f allowed); the engine's processes, with the one page, hold %.1f MB at most (counted apart)",
    machine, Int(memory), middle * 1000, runs, (starts.min() ?? 0) * 1000, (starts.max() ?? 0) * 1000, startBudget * 1000,
    most / 1024 / 1024, memoryBudget / 1024 / 1024, (enginesHeld.max() ?? 0) / 1024 / 1024
))
exit(middle <= startBudget && most <= memoryBudget ? 0 : 1)
