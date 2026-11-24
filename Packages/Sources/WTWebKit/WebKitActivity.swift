// What a page is doing that would keep its tab awake (052): that it plays sound, holds the
// camera or the microphone, and was typed into. WebKit tells an app of the camera and the
// microphone. Of sound and of typing it tells an app nothing, so the adapter hears of them
// from a script of its own, which says yes or no and never what: not what was played, and
// nothing of what was typed.
import WebKit
import WTContract

/// The adapter's ear for sound and for typing. Its script is in every page and every frame,
/// in a world of the adapter's own: a page's scripts cannot see it, call what it calls, or
/// say anything in its name. It hears that a key the person pressed went into a field (an
/// event the engine made, not one a script did), and that an element of the page began or
/// stopped playing something that can be heard, and says so: each frame for itself, by a
/// name it makes up, with all it knows as it now is.
@MainActor
final class ActivityReports: NSObject, WKScriptMessageHandler {
    static let world = WKContentWorld.world(name: "WebTigerActivity")
    static let name = "doing"

    static let script = """
    (function () {
        'use strict';
        var tell = window.webkit.messageHandlers.\(name);
        var frame = String(Math.random()).slice(2) + String(Date.now());
        var typed = false, sound = false;
        function say() { tell.postMessage({ frame: frame, sound: sound, typed: typed }); }

        // Typing: what goes into a line, a larger field, or a part of the page that can be
        // edited. A box that is ticked, a slider and a button are not typing.
        var notText = { checkbox: 1, radio: 1, range: 1, color: 1, file: 1, button: 1, submit: 1, reset: 1, image: 1, hidden: 1 };
        window.addEventListener('input', function (event) {
            if (typed || !event.isTrusted) { return; }
            var path = event.composedPath();
            var on = path.length ? path[0] : event.target;
            if (!on || on.nodeType !== 1) { return; }
            var field = on.localName === 'textarea' || (on.localName === 'input' && !notText[on.type]) || on.isContentEditable === true;
            if (!field) { return; }
            typed = true;
            say();
        }, true);

        // Sound: the players of the page that are playing, with a sound that is not turned
        // off or all the way down.
        var playing = [];
        function heard(player) { return !player.paused && !player.ended && !player.muted && player.volume > 0; }
        function mind(event) {
            var player = event.target;
            if (!player || player.nodeType !== 1 || (player.localName !== 'audio' && player.localName !== 'video')) { return; }
            var at = playing.indexOf(player);
            if (heard(player)) {
                if (at < 0) { playing.push(player); }
            } else if (at >= 0) {
                playing.splice(at, 1);
            }
            var now = playing.length > 0;
            if (now !== sound) { sound = now; say(); }
        }
        ['play', 'playing', 'pause', 'ended', 'volumechange', 'emptied'].forEach(function (name) {
            window.addEventListener(name, mind, true);
        });
        // A frame that goes takes its sound with it.
        window.addEventListener('pagehide', function () {
            if (sound) { playing = []; sound = false; say(); }
        }, true);
    }());
    """

    /// The page each web view is, to be told what its frames say. A web view that has gone
    /// takes its own away.
    private let pages = NSMapTable<WKWebView, Page>.weakToWeakObjects()

    /// Puts the script, and this ear, in what a web view's pages are given. Web views a page
    /// opens are given the same by WebKit.
    func listen(in controller: WKUserContentController) {
        controller.addUserScript(
            WKUserScript(source: Self.script, injectionTime: .atDocumentStart, forMainFrameOnly: false, in: Self.world)
        )
        controller.add(self, contentWorld: Self.world, name: Self.name)
    }

    /// What a page's frames say is told to that page from now.
    func attend(_ page: Page) {
        pages.setObject(page, forKey: page.webView)
    }

    func userContentController(_: WKUserContentController, didReceive message: WKScriptMessage) {
        guard let webView = message.webView, let body = message.body as? [String: Any], let frame = body["frame"] as? String
        else { return }
        pages.object(forKey: webView)?.heard(frame: frame, sound: body["sound"] as? Bool ?? false, typed: body["typed"] as? Bool ?? false)
    }
}

/// What the adapter knows of one page that would keep its tab awake: which of its frames are
/// playing sound, whether it was typed into, and what was last reported of it. It is of the
/// page: a commit starts it again.
struct PageDoing {
    /// The frames that are playing something that can be heard, by the names they gave
    /// themselves.
    private var playing: Set<String> = []
    /// The person typed into a field of the page, in whichever frame.
    private var typedInto = false
    /// What the core was last told. A page that has just committed has none of it, as the
    /// core takes it (engine.proto).
    private var reported = WTPageActivityChanged()

    /// A frame said what it is doing, all of it as it now is.
    mutating func heard(frame: String, sound: Bool, typed: Bool) {
        if sound {
            playing.insert(frame)
        } else {
            playing.remove(frame)
        }
        // What was typed stays typed for as long as the page is the tab's (052 D1).
        typedInto = typedInto || typed
    }

    /// Another page has committed: it is doing none of it yet.
    mutating func committed() {
        self = PageDoing()
    }

    /// What there is to tell the core now, with what WebKit says of the camera and the
    /// microphone: nil when it is what the core was last told.
    mutating func news(camera: WKMediaCaptureState, microphone: WKMediaCaptureState) -> WTPageActivityChanged? {
        var doing = WTPageActivityChanged()
        doing.playsSound = !playing.isEmpty
        doing.captures = Self.captures(camera: camera, microphone: microphone)
        doing.typedInto = typedInto
        guard doing != reported else { return nil }
        reported = doing
        return doing
    }

    /// Whether the page holds the camera or the microphone. One that is held and muted is
    /// held: the page has it still.
    static func captures(camera: WKMediaCaptureState, microphone: WKMediaCaptureState) -> Bool {
        camera != .none || microphone != .none
    }
}
