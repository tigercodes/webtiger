// The Permissions list (014 §7): the sites that were allowed to use the camera, the microphone
// or the location, each with what it may use, and a button to take that back. Taking it back
// stops nothing that is running: the site asks again the next time.
public import SwiftUI
import WTContract
import WTDesign

public struct PermissionsView: View {
    @State private var model: PermissionsModel
    private let switcher: AnyView?
    private let done: () -> Void
    @Environment(\.locale) private var locale

    /// `done` is called when a person is finished. With a `switcher`, it stands where the
    /// title would: the list is then one of several a person can choose between.
    public init(model: PermissionsModel, switcher: AnyView? = nil, done: @escaping () -> Void) {
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
        .task { model.reload() }
    }

    private var header: some View {
        ListHeader(stacked: switcher != nil && ListHeader<AnyView, AnyView>.stacksASwitch) {
            if let switcher {
                switcher
            } else {
                Text("permissions.title", bundle: .module)
                    .font(.headline)
                    .accessibilityAddTraits(.isHeader)
            }
        } buttons: {
            Button {
                model.forgetEverything()
            } label: {
                Text("permissions.clear", bundle: .module)
                    // On more lines where one is not enough, and never cut short.
                    .fixedSize(horizontal: false, vertical: true)
                    .frame(minHeight: BrowserWindowView.hitSize)
            }
            // Neutral, as every button with words is (F14).
            .buttonStyle(.bordered)
            .tint(.primary)
            .disabled(model.sites.isEmpty)
            .accessibilityIdentifier("permissions-clear")
            Button(action: done) {
                Text("permissions.done", bundle: .module)
                    .frame(minHeight: BrowserWindowView.hitSize)
            }
            .buttonStyle(.bordered)
            .tint(.primary)
            .keyboardShortcut(.cancelAction)
            .accessibilityIdentifier("permissions-done")
        }
        .padding(.horizontal, Tokens.Space.m)
        .padding(.vertical, Tokens.Space.s)
    }

    @ViewBuilder private var content: some View {
        if !model.loaded {
            // Nothing is said until the core has answered: not that no site was allowed anything.
            Spacer()
        } else if model.sites.isEmpty {
            Spacer()
            Text("permissions.empty", bundle: .module)
                .multilineTextAlignment(.center)
                // On as many lines as it needs at any text size, and never cut short.
                .fixedSize(horizontal: false, vertical: true)
                .padding(Tokens.Space.l)
                .accessibilityIdentifier("permissions-nothing")
            Spacer()
        } else {
            ScrollView {
                LazyVStack(spacing: 0) {
                    ForEach(model.sites, id: \.origin) { site in
                        row(site)
                        Divider().padding(.leading, Tokens.Space.m)
                    }
                }
            }
        }
    }

    /// One site: its name, as the core names it, and what it may use; a button takes that back.
    private func row(_ site: WTSitePermissions) -> some View {
        HStack(spacing: Tokens.Space.s) {
            VStack(alignment: .leading, spacing: Tokens.Space.xs) {
                // A name may be any length: it is cut in its middle, where its start and its
                // ending both stay.
                Text(site.site)
                    .lineLimit(2)
                    .truncationMode(.middle)
                Text(Self.allowed(site, locale: locale))
                    .font(.footnote)
            }
            .frame(maxWidth: .infinity, minHeight: BrowserWindowView.hitSize, alignment: .leading)
            .accessibilityElement(children: .combine)
            .accessibilityIdentifier("permission-site")
            Button {
                model.forget(site)
            } label: {
                Label { Text("permissions.forget", bundle: .module) } icon: { Image(systemName: "trash") }
                    .labelStyle(.iconOnly)
                    .frame(minWidth: BrowserWindowView.hitSize, minHeight: BrowserWindowView.hitSize)
                    .contentShape(Rectangle())
            }
            .buttonStyle(.borderless)
            .accessibilityIdentifier("permission-forget")
        }
        .padding(.horizontal, Tokens.Space.m)
        .padding(.vertical, Tokens.Space.xs)
    }

    /// What a site may use, in words, joined as the person's language joins a list. A kind
    /// this build has no words for is left out.
    static func allowed(_ site: WTSitePermissions, locale: Locale = .current) -> String {
        let names: [String] = site.kinds.compactMap { kind in
            let key: String.LocalizationValue? =
                switch kind {
                case .camera: "permissions.kind.camera"
                case .microphone: "permissions.kind.microphone"
                case .location: "permissions.kind.location"
                default: nil
                }
            return key.map { String(localized: $0, bundle: .module, locale: locale) }
        }
        let list = ListFormatter()
        list.locale = locale
        return list.string(from: names) ?? names.joined(separator: ", ")
    }
}
