// The Downloads list (013): what was downloaded, newest first; how far each one that is coming
// has got, and how each one that ended did. One that is coming can be stopped; one that has
// ended can be taken off the list, which touches no file.
public import SwiftUI
import WTContract
import WTDesign

public struct DownloadsView: View {
    @State private var model: DownloadsModel
    private let switcher: AnyView?
    private let done: () -> Void

    /// `done` is called when a person is finished. With a `switcher`, it stands where the
    /// title would: the list is then one of two a person can choose between.
    public init(model: DownloadsModel, switcher: AnyView? = nil, done: @escaping () -> Void) {
        _model = State(initialValue: model)
        self.switcher = switcher
        self.done = done
    }

    public var body: some View {
        VStack(spacing: 0) {
            header
            Divider()
            content
        }
        .task { await model.watch() }
    }

    private var header: some View {
        ListHeader(stacked: switcher != nil && ListHeader<AnyView, AnyView>.stacksASwitch) {
            if let switcher {
                switcher
            } else {
                Text("downloads.title", bundle: .module)
                    .font(.headline)
                    .accessibilityAddTraits(.isHeader)
            }
        } buttons: {
            Button {
                model.forgetEverything()
            } label: {
                Text("downloads.clear", bundle: .module)
                    .frame(minHeight: BrowserWindowView.hitSize)
            }
            // Neutral, as every button with words is (F14).
            .buttonStyle(.bordered)
            .tint(.primary)
            .disabled(model.entries.isEmpty)
            .accessibilityIdentifier("downloads-clear")
            Button(action: done) {
                Text("downloads.done", bundle: .module)
                    .frame(minHeight: BrowserWindowView.hitSize)
            }
            .buttonStyle(.bordered)
            .tint(.primary)
            .keyboardShortcut(.cancelAction)
            .accessibilityIdentifier("downloads-done")
        }
        .padding(.horizontal, Tokens.Space.m)
        .padding(.vertical, Tokens.Space.s)
    }

    @ViewBuilder private var content: some View {
        if !model.loaded {
            // Nothing is said until the core has answered: not that nothing was downloaded.
            Spacer()
        } else if model.entries.isEmpty {
            Spacer()
            Text("downloads.empty", bundle: .module)
                .multilineTextAlignment(.center)
                .padding(Tokens.Space.l)
                .accessibilityIdentifier("downloads-nothing")
            Spacer()
        } else {
            ScrollView {
                LazyVStack(spacing: 0) {
                    ForEach(model.entries, id: \.id) { entry in
                        row(entry)
                        Divider().padding(.leading, Tokens.Space.m)
                    }
                    if model.more {
                        Button {
                            model.showOlder()
                        } label: {
                            Text("downloads.older", bundle: .module)
                                .frame(maxWidth: .infinity, minHeight: BrowserWindowView.hitSize)
                        }
                        // Plain, so its words are in the primary colour: they must be read (F14).
                        .buttonStyle(.plain)
                        .accessibilityIdentifier("downloads-older")
                    }
                }
            }
        }
    }

    private func row(_ entry: WTDownloadEntry) -> some View {
        HStack(spacing: Tokens.Space.s) {
            // Each on a line of its own, as in the History list: the file's name, where it
            // came from, and how it stands.
            VStack(alignment: .leading, spacing: Tokens.Space.xs) {
                Text(entry.fileName)
                    .lineLimit(2)
                    .truncationMode(.middle)
                    .accessibilityIdentifier("download-name")
                Group {
                    if !entry.host.isEmpty {
                        Text(entry.host)
                            .lineLimit(1)
                            .truncationMode(.middle)
                    }
                    Text(Self.standing(entry))
                        .accessibilityIdentifier("download-standing")
                    // Why its file did not open when it was last asked to (030), in words.
                    if let refusal = model.refusals[entry.id] {
                        Text(Self.notOpened(refusal))
                            .accessibilityIdentifier("download-not-opened")
                    }
                }
                .font(.footnote)
                if entry.state == .coming {
                    progress(entry)
                }
            }
            .frame(maxWidth: .infinity, minHeight: BrowserWindowView.hitSize, alignment: .leading)
            .accessibilityElement(children: .combine)
            .accessibilityIdentifier("download-entry")
            if entry.state == .coming {
                button("downloads.stop", symbol: "xmark.circle", identifier: "download-stop") { model.stop(entry) }
            } else {
                // A file that is whole is opened with whatever the Mac has for its kind (030).
                // Not on an iPhone yet, which has no way to hand a file of an app's on.
                if Self.opensFiles, entry.state == .finished {
                    button("downloads.open", symbol: "arrow.up.forward.square", identifier: "download-open") {
                        Task { await model.open(entry) }
                    }
                }
                button("downloads.forget", symbol: "trash", identifier: "download-forget") { model.forget(entry) }
            }
        }
        .padding(.horizontal, Tokens.Space.m)
        .padding(.vertical, Tokens.Space.xs)
    }

    /// Whether this system's list can open a file (030): a Mac's.
    static var opensFiles: Bool {
        #if os(macOS)
            true
        #else
            false
        #endif
    }

    /// Why a file did not open, in words, by the core's code (030). A code this build has no
    /// words for still says something, and never shows the code's own text.
    static func notOpened(_ code: String) -> String {
        let key = switch code {
        case "system.no_handler": "error.system.no_handler"
        case "download.missing": "error.download.missing"
        default: "downloads.open_failed"
        }
        return Bundle.module.localizedString(forKey: key, value: nil, table: nil)
    }

    /// How far a download has got, drawn, when its size is known. One whose size is not known
    /// has only its words: a bar that moved without saying anything would be the one thing in
    /// the list that never stood still.
    @ViewBuilder private func progress(_ entry: WTDownloadEntry) -> some View {
        if entry.totalBytes > 0 {
            ProgressView(value: min(Double(entry.receivedBytes), Double(entry.totalBytes)), total: Double(entry.totalBytes))
                .tint(Tokens.Palette.accent)
                .accessibilityHidden(true)
        }
    }

    private func button(_ title: LocalizedStringKey, symbol: String, identifier: String, action: @escaping () -> Void) -> some View {
        Button(action: action) {
            Label { Text(title, bundle: .module) } icon: { Image(systemName: symbol) }
                .labelStyle(.iconOnly)
                .frame(minWidth: BrowserWindowView.hitSize, minHeight: BrowserWindowView.hitSize)
                .contentShape(Rectangle())
        }
        .buttonStyle(.borderless)
        .accessibilityIdentifier(identifier)
    }

    /// How a download stands, in words: how much of it has arrived, how big it is, or why it
    /// failed. The core speaks codes; the words are here.
    static func standing(_ entry: WTDownloadEntry) -> String {
        switch entry.state {
        case .coming:
            if entry.totalBytes > 0 {
                return String(
                    format: String(localized: "downloads.coming", bundle: .module),
                    size(entry.receivedBytes),
                    size(entry.totalBytes)
                )
            }
            return String(format: String(localized: "downloads.coming_so_far", bundle: .module), size(entry.receivedBytes))
        case .finished:
            return entry.totalBytes > 0 ? size(entry.totalBytes) : ""
        case .failed, .unspecified, .UNRECOGNIZED:
            return words(for: entry.errorCode)
        }
    }

    /// A failure's words, by its code. A code this build has no words for still says something,
    /// and never shows the code's own text.
    static func words(for code: String) -> String {
        let known = ["download.cancelled", "download.network", "download.disk", "download.interrupted"]
        let key = known.contains(code) ? "error.\(code)" : "error.download.failed"
        return Bundle.module.localizedString(forKey: key, value: nil, table: nil)
    }

    /// A number of bytes as a person reads a file's size.
    static func size(_ bytes: UInt64) -> String {
        ByteCountFormatter.string(fromByteCount: Int64(min(bytes, UInt64(Int64.max))), countStyle: .file)
    }
}
