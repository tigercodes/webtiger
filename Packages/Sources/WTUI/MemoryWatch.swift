// What the system says of its memory, passed on to the core (055). The shell decides
// nothing: it says which of two words it heard, short or very short, each time it hears one,
// and the core puts to sleep the tabs that can be.
import Dispatch
import Foundation
import WTContract
import WTCoreClient
#if canImport(UIKit)
    import UIKit
#endif

/// Listens to the system for word that it is short of memory, and says so to the core.
///
/// Two things are listened to. The system's memory-pressure source, on every one of Apple's
/// systems: its warning is short, its critical is very short. It tells of a change and not of
/// a state: an app that starts while the system is short already hears nothing until that
/// changes. And, on an iPhone and an iPad, the app's memory warning. That has one word, and
/// comes with the source's warning as well as with its critical, and when the app alone is
/// near what it may use: it is passed on as short, and the worse word is left to the source.
@MainActor
final class MemoryWatch {
    private let core: any CoreClient
    private let centre: NotificationCenter
    private var pressure: (any DispatchSourceMemoryPressure)?
    private var warnings: (any NSObjectProtocol)?

    /// `centre`: where the app's memory warning is posted. A test passes one of its own.
    init(core: any CoreClient, centre: NotificationCenter = .default) {
        self.core = core
        self.centre = centre
    }

    /// Starts listening to the system. Not for fake mode: a suite's run must not go by how
    /// busy the machine it runs on is.
    func listen() {
        listenForPressure()
        listenForWarnings()
    }

    /// Stops listening.
    func stop() {
        pressure?.cancel()
        pressure = nil
        if let warnings {
            centre.removeObserver(warnings)
        }
        warnings = nil
    }

    /// The system's memory-pressure source, heard on the main queue.
    func listenForPressure() {
        guard pressure == nil else { return }
        let source = DispatchSource.makeMemoryPressureSource(eventMask: [.warning, .critical], queue: .main)
        source.setEventHandler { [weak self] in
            // The source's queue is the main one.
            MainActor.assumeIsolated {
                guard let self, let source = self.pressure else { return }
                self.heard(source.data)
            }
        }
        pressure = source
        source.activate()
    }

    /// The app's memory warning (an iPhone's and an iPad's): short. A Mac has none.
    func listenForWarnings() {
        #if canImport(UIKit)
            guard warnings == nil else { return }
            warnings = centre
                .addObserver(forName: UIApplication.didReceiveMemoryWarningNotification, object: nil, queue: .main) { [weak self] _ in
                    MainActor.assumeIsolated {
                        self?.say(.short)
                    }
                }
        #endif
    }

    /// The memory-pressure source said something: passed on when it is one of the two words.
    func heard(_ event: DispatchSource.MemoryPressureEvent) {
        if let level = Self.level(of: event) {
            say(level)
        }
    }

    /// The source's word as the contract's: its warning is short, its critical is very short,
    /// and the worse of the two when it says both. Its "normal" is no word at all: nothing is
    /// said to the core when the shortage ends, since nothing lasted (055).
    static func level(of event: DispatchSource.MemoryPressureEvent) -> WTMemoryLevel? {
        if event.contains(.critical) {
            return .veryShort
        }
        if event.contains(.warning) {
            return .short
        }
        return nil
    }

    /// Says to the core that the system is short of memory, so short. Nothing is waited for.
    func say(_ level: WTMemoryLevel) {
        var short = WTMemoryShort()
        short.level = level
        _ = core.send(.memoryShort(short))
    }
}
