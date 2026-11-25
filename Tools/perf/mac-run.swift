// The run (000 §7, 063): the Mac app with a list of sites open, measured.
//
//   swift Tools/perf/mac-run.swift [--web] <WebTiger.app> <list> <runs> <seconds between> <seconds idle>
//
// For each run it starts the app's own program in fake mode (the store in memory: nothing of
// anybody's is read, written or left), with the list for the app to open by itself, an
// address every so many seconds, each in a tab, as a link from another app is opened; and
// with this machine's own limit of live tabs, as the app has outside fake mode. Nothing in
// the app is automated. It measures twice: when the last address has had its seconds, and
// again after the idle time. Each time: what the system counts against the app's own
// process (its footprint), what it counts against the engine's processes, and the energy
// each has used since it began. Then it ends the app, by its number.
//
// The engine's processes are WebKit's, for pages, the network and drawing. The system
// starts them, and they are not the app's children. The ones that are the app's are told
// by this: they were not running before the app was started, and they are gone when it has
// ended. One that outlives the app is somebody else's, and is not counted. A page's process
// that ended while the app ran (its tab was put to sleep) holds nothing any more; the energy
// it used is counted as it was last read, a second or less before it ended.
//
// An address that is not on this machine is loaded only with --web. The list's addresses
// are in no line this prints: it says the list's file, and how many.
import Darwin
import Foundation

func fail(_ words: String) -> Never {
    FileHandle.standardError.write(Data((words + "\n").utf8))
    exit(2)
}

var arguments = Array(CommandLine.arguments.dropFirst())
let web = arguments.first == "--web"
if web { arguments.removeFirst() }
guard arguments.count == 5, let runs = Int(arguments[2]), let every = Int(arguments[3]), let idle = Int(arguments[4]),
      runs > 0, every > 0, idle >= 0
else {
    fail("usage: swift Tools/perf/mac-run.swift [--web] <WebTiger.app> <list> <runs> <seconds between> <seconds idle>")
}
let app = URL(fileURLWithPath: arguments[0])
let listFile = URL(fileURLWithPath: arguments[1])
guard let program = Bundle(url: app)?.executableURL else { fail("no app at \(app.path)") }
guard let written = try? String(contentsOf: listFile, encoding: .utf8) else { fail("no list at \(listFile.path)") }
let addresses = written.split(whereSeparator: \.isNewline)
    .map { $0.trimmingCharacters(in: .whitespaces) }
    .filter { !$0.isEmpty && !$0.hasPrefix("#") }
guard !addresses.isEmpty else { fail("the list has no address") }
for (line, address) in addresses.enumerated() {
    guard let url = URL(string: address), let scheme = url.scheme, ["http", "https"].contains(scheme), let host = url.host,
          !address.contains(where: \.isWhitespace)
    else {
        fail("the list's address \(line + 1) is no address of the web")
    }
    let here = host == "127.0.0.1" || host == "localhost"
    if !here, !web {
        fail("the list's address \(line + 1) is not on this machine: with --web the run loads other people's sites")
    }
}

/// What the system counts against a process: what it holds (its footprint), and the energy
/// it has used since it began.
struct Usage {
    var bytes = 0.0
    var joules = 0.0
}

func usage(_ pid: pid_t) -> Usage? {
    var info = rusage_info_v6()
    let read = withUnsafeMutablePointer(to: &info) { pointer in
        pointer.withMemoryRebound(to: rusage_info_t?.self, capacity: 1) { proc_pid_rusage(pid, RUSAGE_INFO_V6, $0) }
    }
    return read == 0 ? Usage(bytes: Double(info.ri_phys_footprint), joules: Double(info.ri_energy_nj) / 1_000_000_000) : nil
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

/// One reading of a run: the app's own process, and the engine's processes that began after
/// it. Those that are running hold something; every one that has run has used energy.
struct Reading {
    var own = Usage()
    var held: [pid_t: Double] = [:]
    var used: [pid_t: Double] = [:]
}

/// A reading, of the engine's processes that were the app's.
struct Measure {
    var own = 0.0
    var engines = 0.0
    var processes = 0
    var ownJoules = 0.0
    var enginesJoules = 0.0
    var both: Double { own + engines }
    var joules: Double { ownJoules + enginesJoules }

    init(_ reading: Reading, its: Set<pid_t>) {
        own = reading.own.bytes
        ownJoules = reading.own.joules
        for (pid, bytes) in reading.held where its.contains(pid) {
            engines += bytes
            processes += 1
        }
        for (pid, joules) in reading.used where its.contains(pid) {
            enginesJoules += joules
        }
    }
}

let megabyte = 1024.0 * 1024.0
func said(_ measure: Measure) -> String {
    String(
        format: "the app %.1f MB, the engine's %d processes %.1f MB, both %.1f MB; %.1f J used",
        measure.own / megabyte, measure.processes, measure.engines / megabyte, measure.both / megabyte, measure.joules
    )
}

func middle(_ numbers: [Double]) -> Double { numbers.sorted()[numbers.count / 2] }

var size = 0
sysctlbyname("hw.model", nil, &size, nil, 0)
var model = [CChar](repeating: 0, count: max(size, 1))
sysctlbyname("hw.model", &model, &size, nil, 0)
let information = Bundle(url: app)?.infoDictionary ?? [:]
let version = "\(information["CFBundleShortVersionString"] ?? "?") (\(information["CFBundleVersion"] ?? "?"))"
let engine = Bundle(path: "/System/Library/Frameworks/WebKit.framework")?.infoDictionary?["CFBundleVersion"] as? String ?? "?"
let day = ISO8601DateFormatter.string(from: Date(), timeZone: .current, formatOptions: [.withFullDate])
print("The run, \(day): WebTiger \(version) on \(String(cString: model)) with \(ProcessInfo.processInfo.physicalMemory / 1024 / 1024 / 1024) GB, "
    + "macOS \(ProcessInfo.processInfo.operatingSystemVersionString), WebKit \(engine).")
print("The list: \(listFile.lastPathComponent), \(addresses.count) addresses\(web ? "" : ", all on this machine"). One is opened every \(every) seconds; "
    + "the second measure is \(idle) seconds after the first. Fake mode, with this machine's own limit of live tabs; nothing blocked.")

var opened: [Measure] = []
var rested: [Measure] = []
for run in 1 ... runs {
    let before = enginesProcesses()
    let process = Process()
    process.executableURL = program
    var environment = ProcessInfo.processInfo.environment
    environment["WEBTIGER_FAKE"] = "1"
    environment["WEBTIGER_FAKE_OPEN"] = addresses.joined(separator: " ")
    environment["WEBTIGER_FAKE_OPEN_EVERY"] = String(every)
    environment["WEBTIGER_FAKE_DEVICE_LIMITS"] = "1"
    process.environment = environment
    // A Mac puts back the windows an app had when it last ended: each run is to start from
    // the one window the app opens by itself.
    process.arguments = ["-ApplePersistenceIgnoreState", "YES"]
    process.standardOutput = FileHandle.nullDevice
    process.standardError = FileHandle.nullDevice
    do {
        try process.run()
    } catch {
        fail("the app would not start: \(error.localizedDescription)")
    }
    let began = Date()
    var reading = Reading()
    var readings: [Reading] = []
    // The last address is opened after as many times the seconds as there are addresses,
    // and has the same seconds to arrive; then the idle time.
    let moments = [Double(every * (addresses.count + 1)), Double(every * (addresses.count + 1) + idle)]
    for moment in moments {
        repeat {
            guard process.isRunning else { fail("run \(run): the app ended by itself") }
            if let own = usage(process.processIdentifier) { reading.own = own }
            reading.held = [:]
            for pid in enginesProcesses().subtracting(before) {
                guard let read = usage(pid) else { continue }
                reading.held[pid] = read.bytes
                reading.used[pid] = read.joules
            }
            if Date().timeIntervalSince(began) < moment { Thread.sleep(forTimeInterval: 1) }
        } while Date().timeIntervalSince(began) < moment
        readings.append(reading)
    }
    process.terminate()
    process.waitUntilExit()
    // The engine's processes that began after the app and have gone with it were the app's.
    let began_after = Set(reading.used.keys)
    var left = enginesProcesses()
    let waitedSince = Date()
    while !left.isDisjoint(with: began_after), Date().timeIntervalSince(waitedSince) < 5 {
        usleep(100_000)
        left = enginesProcesses()
    }
    let its = began_after.subtracting(left)
    let first = Measure(readings[0], its: its)
    let second = Measure(readings[1], its: its)
    opened.append(first)
    rested.append(second)
    let others = began_after.count - its.count
    print("run \(run): all opened: \(said(first))")
    print("run \(run): \(idle) seconds on: \(said(second))" + (others == 0 ? "" : " (\(others) of the engine's processes began meanwhile and outlived the app: somebody else's)"))
    Thread.sleep(forTimeInterval: 1)
}

for (when, measures) in [("all opened", opened), ("\(idle) seconds on", rested)] {
    print(String(
        format: "the middle of %d runs, %@: the app %.1f MB, the engine's processes %.1f MB, both %.1f MB (from %.1f to %.1f); %.1f J used",
        runs, when, middle(measures.map(\.own)) / megabyte, middle(measures.map(\.engines)) / megabyte,
        middle(measures.map(\.both)) / megabyte, (measures.map(\.both).min() ?? 0) / megabyte, (measures.map(\.both).max() ?? 0) / megabyte,
        middle(measures.map(\.joules))
    ))
}
