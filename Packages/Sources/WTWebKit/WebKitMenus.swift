// What a menu saves (022), and on a Mac the menu itself. WebKit builds the menu of a link and
// of a picture there, and does not tell an app what it is for; its items that save a file
// hand the download over in a way that is not open to an app, and drop it when nobody takes
// it. So the adapter learns what was pressed from a script of its own, and gives those items
// their work. The file is a download like any other (013), said to be the person's own.
public import Foundation
import WebKit
import WTContract

public extension WebKitEngine {
    /// The person chose, in a menu, to save a file of a web view's page: a picture, or what a
    /// link leads to. WebKit fetches it in the page's own session, and it is reported as a
    /// download the person asked for (022): whether it starts, and under what name, is the
    /// core's to say. Only an address of the web is fetched.
    func saveFile(webViewID: UInt64, url: URL) {
        guard let webView = webView(for: webViewID), ["http", "https"].contains(url.scheme?.lowercased() ?? "") else { return }
        // The download is the adapter's from the moment WebKit has made it: one that nobody
        // has taken by the time this returns is dropped.
        webView.startDownload(using: URLRequest(url: url)) { [weak self] download in
            self?.downloads.adopt(download, from: webViewID, chosen: true)
        }
    }

    /// The person chose, in the menu of words they selected in a web view's page, to search
    /// for them (023). The core is told: where the search goes, and whether a tab opens for
    /// it, are its to say.
    func search(webViewID: UInt64, words: String) {
        var asked = WTSearchRequested()
        asked.text = words
        asked.userGesture = true
        report(webViewID, .searchRequested(asked))
    }
}

#if os(macOS)
    import AppKit

    /// What a right-click was on, as the adapter's script said: the address of the picture,
    /// of the link, and of the film or sound, where there was one of the web's; and the words
    /// that were selected.
    struct Pressed: Equatable {
        var picture: URL?
        var link: URL?
        /// What a player of the page is playing: a film or a sound.
        var media: URL?
        /// The words selected in the page as the click happened (023). Never what is typed
        /// in a password's field.
        var words = ""
    }

    /// The adapter's ear for what was right-clicked. Its script is in every page and every
    /// frame, in a world of the adapter's own: a page's scripts cannot see it, call what it
    /// calls, or change what it reads. It hears the menu's event before any listener of the
    /// page's, takes only an event the engine made, and says what the event was on; WebKit
    /// opens its menu after that.
    @MainActor
    final class PressedReports: NSObject, WKScriptMessageHandler {
        static let world = WKContentWorld.world(name: "WebTigerMenus")
        static let name = "pressed"
        /// How long what was said holds: a menu opens at once, or not for that click.
        static let fresh: TimeInterval = 5

        static let script = """
        (function () {
            'use strict';
            var tell = window.webkit.messageHandlers.\(name);
            window.addEventListener('contextmenu', function (event) {
                if (!event.isTrusted) { return; }
                var picture = '', link = '', media = '';
                var path = event.composedPath();
                for (var i = 0; i < path.length; i += 1) {
                    var node = path[i];
                    if (!node || node.nodeType !== 1) { continue; }
                    if (!picture && node.localName === 'img') {
                        var shown = node.currentSrc || node.src;
                        if (typeof shown === 'string') { picture = shown; }
                    }
                    if (!link && (node.localName === 'a' || node.localName === 'area') && typeof node.href === 'string') {
                        link = node.href;
                    }
                    if (!media && (node.localName === 'video' || node.localName === 'audio')) {
                        var playing = node.currentSrc || node.src;
                        if (typeof playing === 'string') { media = playing; }
                    }
                }
                // The words that are selected, for a search for them. What is typed in a
                // password's field is nobody's to read: not when the click was in one, and not
                // when one has the keyboard, wherever in the page the field is. (In this world
                // the script sees into the parts of a page that are closed to the page's own.)
                var secret = false;
                var fields = path.slice();
                var active = document.activeElement;
                while (active && active.shadowRoot && active.shadowRoot.activeElement) { active = active.shadowRoot.activeElement; }
                fields.push(active);
                for (var j = 0; j < fields.length; j += 1) {
                    var field = fields[j];
                    if (field && field.nodeType === 1 && field.localName === 'input' && field.type === 'password') { secret = true; }
                }
                var words = secret ? '' : String(window.getSelection()).slice(0, 2000);
                tell.postMessage({ picture: picture, link: link, media: media, words: words });
            }, true);
        }());
        """

        private final class Told {
            let pressed: Pressed
            let at = Date()

            init(_ pressed: Pressed) {
                self.pressed = pressed
            }
        }

        /// What was last said of each web view. A web view that has gone takes its own away.
        private let said = NSMapTable<WKWebView, Told>.weakToStrongObjects()

        /// Puts the script, and this ear, in what a web view's pages are given. Web views a
        /// page opens are given the same by WebKit.
        func listen(in controller: WKUserContentController) {
            controller.addUserScript(
                WKUserScript(source: Self.script, injectionTime: .atDocumentStart, forMainFrameOnly: false, in: Self.world)
            )
            controller.add(self, contentWorld: Self.world, name: Self.name)
        }

        func userContentController(_: WKUserContentController, didReceive message: WKScriptMessage) {
            guard let webView = message.webView, let body = message.body as? [String: Any] else { return }
            let pressed = Pressed(
                picture: Self.address(body["picture"]), link: Self.address(body["link"]), media: Self.address(body["media"]),
                words: body["words"] as? String ?? ""
            )
            said.setObject(Told(pressed), forKey: webView)
        }

        /// What the last right-click in a web view was on. Said once, and only of a click
        /// that has just happened: nil when the script said nothing of it.
        func take(for webView: WKWebView, now: Date = Date()) -> Pressed? {
            guard let told = said.object(forKey: webView) else { return nil }
            said.removeObject(forKey: webView)
            return now.timeIntervalSince(told.at) < Self.fresh ? told.pressed : nil
        }

        /// An address of the web, as the script gave it. Anything else is no address to save.
        private static func address(_ said: Any?) -> URL? {
            guard let text = said as? String, let url = URL(string: text), ["http", "https"].contains(url.scheme?.lowercased() ?? "")
            else { return nil }
            return url
        }
    }

    /// What one of the menu's items does when it is chosen. The item holds it.
    @MainActor
    final class MenuChoice: NSObject {
        private let run: @MainActor () -> Void

        init(_ run: @escaping @MainActor () -> Void) {
            self.run = run
        }

        @objc func chosen(_: Any?) {
            run()
        }
    }

    /// The web view the adapter makes on a Mac. WebKit builds its menu itself. Two of its
    /// items open something in a new window, which the core opens as a tab (006): they are
    /// given the shell's words for what they do here. Two save a file, and do not: they are
    /// given the address the adapter's script named, or taken out when it named none; so is
    /// the one that saves a film or a sound. One searches for the words that are selected,
    /// with the system's search engine and in the system's browser: in its place stands the
    /// shell's, which tells the core (023).
    final class MenuWebView: WKWebView {
        /// WebKit's own names for its items. They are not declared constants: should WebKit
        /// rename one, that item is left as WebKit made it, and the app's test of the menu
        /// says so.
        static let opensLinkInNewWindow = "WKMenuItemIdentifierOpenLinkInNewWindow"
        static let opensImageInNewWindow = "WKMenuItemIdentifierOpenImageInNewWindow"
        static let downloadsImage = "WKMenuItemIdentifierDownloadImage"
        static let downloadsLinkedFile = "WKMenuItemIdentifierDownloadLinkedFile"
        static let downloadsMedia = "WKMenuItemIdentifierDownloadMedia"
        static let searchesWeb = "WKMenuItemIdentifierSearchWeb"

        override func willOpenMenu(_ menu: NSMenu, with event: NSEvent) {
            super.willOpenMenu(menu, with: event)
            guard let page = navigationDelegate as? Page, let engine = page.engine else { return }
            let webViewID = page.id
            Self.arrange(
                menu, pressed: engine.pressed.take(for: self), words: engine.linkMenu,
                save: { [weak engine] address in engine?.saveFile(webViewID: webViewID, url: address) },
                search: { [weak engine] words in engine?.search(webViewID: webViewID, words: words) }
            )
        }

        /// Makes WebKit's menu the app's: the shell's words on the items that open a tab; on
        /// the items that save, the address to save, with `save` to do it; and in place of
        /// the item that searches, the shell's, with `search` to do it. An item that has
        /// nothing to save or to search for, or no words of the shell's to say what it does,
        /// is taken out: it would do nothing, or go where the person did not choose.
        static func arrange(
            _ menu: NSMenu,
            pressed: Pressed?,
            words: LinkMenu?,
            save: @escaping @MainActor (URL) -> Void,
            search: @escaping @MainActor (String) -> Void = { _ in }
        ) {
            for item in menu.items {
                switch item.identifier?.rawValue {
                case opensLinkInNewWindow:
                    item.title = words?.openInNewTab ?? item.title
                case opensImageInNewWindow:
                    item.title = words?.openImageInNewTab ?? item.title
                case downloadsImage:
                    put(pressed?.picture.map { address in MenuChoice { save(address) } }, titled: item.title, for: item, of: menu)
                case downloadsLinkedFile:
                    put(pressed?.link.map { address in MenuChoice { save(address) } }, titled: item.title, for: item, of: menu)
                case downloadsMedia:
                    put(pressed?.media.map { address in MenuChoice { save(address) } }, titled: item.title, for: item, of: menu)
                case searchesWeb:
                    // Only for words there are, and under words of the shell's that name
                    // where the search goes: WebKit's name the system's engine.
                    let selected = pressed?.words ?? ""
                    let known = !selected.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty
                    let title = words?.searchWords
                    put(known && title != nil ? MenuChoice { search(selected) } : nil, titled: title ?? "", for: item, of: menu)
                default:
                    break
                }
            }
            tidy(menu)
        }

        /// Puts an item of the adapter's own where WebKit's stood, under `title` and WebKit's
        /// name for the item, that does `choice`; or takes WebKit's out when there is none.
        private static func put(_ choice: MenuChoice?, titled title: String, for item: NSMenuItem, of menu: NSMenu) {
            let place = menu.index(of: item)
            menu.removeItem(item)
            guard let choice else { return }
            let own = NSMenuItem(title: title, action: #selector(MenuChoice.chosen(_:)), keyEquivalent: "")
            own.identifier = item.identifier
            own.target = choice
            // An item does not hold its target: this does.
            own.representedObject = choice
            menu.insertItem(own, at: place)
        }

        /// Leaves no line at either end of the menu, nor two together, where items went.
        private static func tidy(_ menu: NSMenu) {
            var lastWasLine = true
            for item in menu.items {
                if item.isSeparatorItem, lastWasLine {
                    menu.removeItem(item)
                } else {
                    lastWasLine = item.isSeparatorItem
                }
            }
            if let last = menu.items.last, last.isSeparatorItem {
                menu.removeItem(last)
            }
        }
    }
#endif
