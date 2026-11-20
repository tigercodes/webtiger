# 014 — A page asks to use the camera, the microphone or where I am (W1)

| | |
|---|---|
| **Status** | In progress: the store's record of what was allowed (§5), the contract (1.10) and the core (§3) are built and tested. No shell reports a page asking yet, and none draws the prompt (§11). The design is to be read before the rest is built: §9 lists what it decides, and §10 what cannot be tested here and why |
| **Phase** | W1 of [000](000-master-plan.md) §9: "permission prompts". Site permissions per login set (000 §5.6, `wt-permissions`) in their first form, and the first of a tab's prompts |
| **Contract** | 1.10: `PermissionRequested`, `PermissionWithdrawn`; the host request `PermissionAnswer`; `TabView.prompt`; `AnswerPrompt`; `QueryPermissions`, `ForgetPermissions` |
| **Last updated** | 2026-11-19 |

## 1. User story

A page wants my camera for a call. WebTiger asks me, and says which site is asking. I allow it, and the call starts; the next time I am on that site it does not ask again. If I say no, the page is told no, and it does not get to ask me over and over. Later I can see which sites I allowed, and take it back.

## 2. What is in it, and what is not

**In:** a page's request for the camera, the microphone, or the device's location reaches the core; the core answers from what was allowed before, or asks the person, in the tab, naming the site; "Allow" is remembered for that site in that login set; a list of what was allowed, where it can be taken back.

**Not in, each with where it goes:**

- **Notifications and web push.** Not offered on a system web view (000 §4.7): a page that asks is told no by the engine.
- **Location on Apple.** WebKit hands the app no public way to answer a page's request for location: it asks the person itself, or refuses. Nothing is built for it, and no test asks for it (§10).
- **"Only this time", and "Never for this site".** There are two answers: allow, which is remembered, and not now, which is not. The others arrive with the sheet a site's own settings will have (with certificate pages, where the padlock gains a sheet).
- **A frame from another site asking**, as an embedded call does: refused without asking (§3). Handing a frame the page's permission (`allow="camera"`) arrives when a site that needs it is in front of us.
- **A mark on a tab that is using the camera or the microphone**: "audio and capture state" (000 §5.5), its own feature. The system's own mark is there meanwhile.
- **Motion and orientation** (WebKit asks for it on an iPhone, by itself), **protected media** and **MIDI** (Android): left to the engine, or refused.
- **Asking before a site's first download** (013 §2): the same kind of prompt. Built on this, next.
- **Alerts, confirmations and other things a page puts in front of a person**: later prompts of the tab. This feature makes the place for them (§4).

## 3. Who does what

The engine notices that a page asks, and says so. The core decides: it answers at once from what it knows, or it puts a prompt in the tab's snapshot and waits for the person. The shell draws the prompt and sends the answer. The engine is then told to let the page have what it asked for, or not.

| What the shell reports | What the core does |
|---|---|
| `PermissionRequested`: a web view's page asks, with an id of the shell's own, who asks (the origin of the frame that asked: scheme, host and port) and for what (one or more of camera, microphone, location) | Applies the rules below. Answers the shell with `PermissionAnswer`, now or when the person has answered |
| `PermissionWithdrawn`: the page no longer waits for that answer | Takes the prompt away; nothing is answered |

**The rules, in order.** A request is refused, with nobody asked, when:

1. the web view is not one the core has, or its tab has no committed page;
2. **who asks is not the page**: the origin is not the origin of the address the tab has committed. A frame from another site, and a request that arrives after the tab went somewhere else, are both this;
3. the origin is not `https`, and is not this device itself (`localhost`, a loopback address). The engines do not let such a page ask; the core does not rely on it;
4. it names nothing the core knows how to ask about;
5. the person said no to any of the same kinds on this page: until the tab commits a new page, it is not asked again;
6. the tab has a prompt waiting already and three more behind it.

Otherwise:

- **If every kind asked for was allowed before** for this origin in this login set, it is granted, with nobody asked.
- **Else the person is asked**: the tab gains a prompt (§4). One prompt is shown at a time; later ones wait in order. A tab that is not in front keeps its prompt until it is: its page waits.
- **"Allow"** grants the request, and writes down each kind it asked for, for the origin and the login set. **"Don't Allow"** refuses it, and is remembered for this page only (rule 5).
- **A request is granted whole or not at all**: a page that asks for camera and microphone gets both or neither.
- **A new page in the tab ends what was waiting**, and so does the loss of its renderer: each request is refused, and its prompt goes. Nothing is written down. **A tab that closes** takes its prompts with it; the shell refuses what waits with a web view it destroys, and the core says nothing more of it.
- **What else waited for the same answer has it.** After a yes, every request of that origin that wanted no more than is now allowed is granted, in whatever tab of the login set it waited. After a no, whatever else in that tab wants something that was refused is refused.
- **A request said twice is asked about once**, and an answer names the prompt the tab shows: one that waits its turn cannot be answered ahead of it.

**What was allowed** is listed by `QueryPermissions` (the sites of a window's login set, each with what it may use) and taken back by `ForgetPermissions` (one site's, or all). Taking it back stops nothing that is running: the page asks again the next time it wants it.

**Nothing of this is logged**: who asked is an address, and is marked for redaction.

## 4. The contract (1.10)

Added; nothing is changed or removed.

```proto
// engine.proto
enum PermissionKind {
  PERMISSION_KIND_UNSPECIFIED = 0;
  PERMISSION_KIND_CAMERA = 1;
  PERMISSION_KIND_MICROPHONE = 2;
  PERMISSION_KIND_LOCATION = 3;
}

// A page asks to use something of the device's. The page waits until the
// shell is told how to answer it (PermissionAnswer).
message PermissionRequested {
  uint64 request_id = 1;                    // the shell's own, never used twice
  string origin = 2;                        // redacted. The frame that asked: scheme, host, port
  repeated PermissionKind kinds = 3;
}

// The page no longer waits for an answer to that request.
message PermissionWithdrawn {
  uint64 request_id = 1;
}

// host.proto: WebViews. How the page that asked is answered.
message PermissionAnswer {
  uint64 request_id = 1;
  bool granted = 2;
}

// browser.proto
// What a tab asks the person: TabView.prompt. Not set when it asks nothing.
message PromptView {
  uint64 prompt_id = 1;                     // the core's own
  oneof kind {
    PermissionPrompt permission = 2;
  }
}

message PermissionPrompt {
  string site = 1;                          // redacted. Who asks, as the address field shows a host
  repeated PermissionKind kinds = 2;
}

// The person's answer to the prompt a tab shows. "prompt.unknown" when the
// tab shows no such prompt: it was answered, or its page went away.
message AnswerPrompt {
  uint64 tab_id = 1;
  uint64 prompt_id = 2;
  bool allow = 3;
}

message QueryPermissions { uint64 window_id = 1; }

message PermissionsPage {
  repeated SitePermissions sites = 1;       // by site, in the order of their names
}

message SitePermissions {
  string origin = 1;                        // redacted. What ForgetPermissions names it by
  string site = 2;                          // redacted. As the address field shows a host
  repeated PermissionKind kinds = 3;        // what it was allowed
}

message ForgetPermissions {
  uint64 window_id = 1;
  oneof what {
    string origin = 2;                      // redacted
    bool everything = 3;
  }
}
```

`PromptView` is a tab's prompt of any kind: later prompts (a sign-in, a page's alert, a download a site did not ask for by a click) are further cases of its `kind`, answered by further requests.

## 5. The store

Migration 6 adds what was allowed, per login set and origin.

```sql
CREATE TABLE site_permission (
    login_set_id BLOB NOT NULL REFERENCES login_set (id),
    origin TEXT NOT NULL,                  -- scheme://host[:port]
    kind TEXT NOT NULL,                    -- 'camera', 'microphone', 'location'
    verdict TEXT NOT NULL CHECK (verdict IN ('allow', 'refuse')),
    PRIMARY KEY (login_set_id, origin, kind)
) STRICT, WITHOUT ROWID;
```

By origin and not by site: what `meet.example.com` was allowed, `ads.example.com` was not. Only `allow` is written by this feature; `refuse` is there for "Never for this site" (§2). Another login set's rows are not read: a Space that was never asked is asked.

It is not `site_setting` (012), whose rows are one to a site: these are several to an origin.

## 6. The engines

| | WebKit | System WebView |
|---|---|---|
| Camera, microphone | `WKUIDelegate`: `requestMediaCapturePermissionFor:initiatedByFrame:type:`, answered with grant or deny | `WebChromeClient.onPermissionRequest`, for video and audio capture; `grant` or `deny` |
| Location | WebKit's own (§2) | `onGeolocationPermissionsShowPrompt`, answered through its callback; never "retained": the core remembers, not the WebView |
| Withdrawn | The web view going away | `onPermissionRequestCanceled`, `onGeolocationPermissionsHidePrompt` |
| Anything else a page asks | WebKit's own answer | Refused |

**The system's own permission is a second thing.** Each system keeps its own answer to whether WebTiger, the app, may use the camera, the microphone or the location, and asks the person with a dialog of its own the first time.

- **Apple**: the system asks when WebKit first starts a camera or a microphone, by itself. The app must say why in its `Info.plist` (`NSCameraUsageDescription`, `NSMicrophoneUsageDescription`), or the system ends it on the spot.
- **Android**: the app must name the permissions in its manifest and ask for them itself. When the core grants a request and the app does not hold what it needs, the adapter asks the system first; if the system says no, the page is told no. What the core wrote down stands: the site was allowed, and the device was not.

**No test may raise those dialogs** (the repository's rule, and [007](007-outside-links.md) §8 is why). So:

- An engine grants a capture only when it was made to: the app, started for real, makes its engine so. Every engine a test makes, and fake mode's, **answers the page "no" whatever the core said**, on Apple; on Android it uses what the app already holds and never asks the system. A test cannot bring the system's dialog up by a mistake of its own.
- What "granted" does is tested at the seam between the adapter and the engine, and on Android with a permission the test is given without a dialog (the test rule that grants it).

## 7. The shells

**The prompt** is a bar under the toolbar, in the tab that asks, like the find and zoom bars: a sentence that names the site and what it wants ("example.com wants to use your camera"), and two buttons, **Don't Allow** and **Allow**. It is not a dialog: the rest of the window works, and going elsewhere in the tab is an answer (no).

- The site is named as the address field names it, by the core. The bar shows nothing the page wrote.
- Neither button is the default for Return, and the bar does not take the keyboard from the page or the field by appearing: a key meant for the page must not answer for the person.

**What was allowed** is a third list in the sheet behind the History button, beside History and Downloads (013): each site with what it may use, and a button to take that back; one button takes all back. The Library (W3) will be its home, and a site's own sheet at the padlock a nearer one (§2).

## 8. Tests

| Where | What |
|---|---|
| Store (`wt-store`, 9) | The migration from version 5, which keeps what was there. What an origin was allowed is written and read back, a kind once; an origin is itself, and not its site's other names, another scheme or another port; another login set was never asked; the origins listed in order, each with what it may use; taking back one origin's, and all of a login set's; nothing is written for a login set the store does not have; a refusal in the store is not read as an allowance, and a verdict that is neither is not written; what was taken back is not left in the store's files. Three of the rules checked by breaking them, and a fourth that no test noticed until one was written for it |
| An address's origin (`wt-core`, 2) | The scheme, the host, and a port that is not the scheme's own; never the part before an `@`; none for an address that is not the web's. An address is its own over `https`, or when it never left the device: `localhost` and what ends in it, the loopback addresses; not a private network's, and not a name that only looks local |
| Core (`wt-app/tests/permissions.rs`, 23 through `FakeShell`) | A page that asks is asked about in its tab, by the site's name, and waits. "Allow" grants it and is remembered: the next request is granted unasked, on another page of the origin too, and after the app starts again over the same store; another origin, and another kind, were not allowed by it. "Don't Allow" refuses it, writes nothing, and the page is not asked for the same again, also after it changes its own address, until another page has loaded. Who asks must be the page: a frame from elsewhere, another of the site's names, another scheme or port, no origin, and the page the tab has left are refused; the page's own origin is itself however a shell spells it. A page not over `https` is refused unless it is this device, which is named with its scheme and port. A web view that is no tab's, and a tab with no page. Nothing, no kind, a later contract's kind, and one of each. Camera and microphone are asked about together and granted together, each named once; one of two allowed is not a grant in part. Four wait, a fifth is told no, and they are asked one at a time in order. What else waited for the same answer has it, after a yes and after a no; in another tab too, and not another origin's. A new page refuses what waited, and a page that only changes its address goes on asking; a lost renderer. A tab behind keeps its prompt until it is in front. A tab that closes. An answer to a prompt the tab does not show, to one that waits its turn, and twice. Withdrawn, and not by another web view's word. Said twice. What was allowed is listed in the order of the names shown, and taken back, one origin's and all; naming nothing; a window there is not. Seventeen of the rules checked by breaking them: two had no test until that showed it |
| WebKit adapter suite | On a Mac that has a camera or a microphone: a page that asks is reported with its origin and what it asks for, and waits; told no, its promise fails as refused. With no such device the engine never asks, and the test says it was skipped. What "granted" does, at the seam. Never run: a capture that starts (§10) |
| System WebView adapter | On the JVM: requests reported and answered, the system's permission asked only when the core granted, and "no" when it says no. On the emulator: camera, microphone, both, and location, each reported with its origin and waiting; refused; granted, with the permission given to the test, and the page gets a stream from the emulated camera |
| The shells' models and bars | Against `ScriptedCore`: the bar is there when the snapshot has a prompt and gone when it has not; each button sends its answer with the prompt's id; the words for each kind and for camera and microphone together; Return answers nothing. Pictures of the bar, light and dark. The list of what was allowed: its model, drawn, and pictures |
| The apps | In fake mode, with the real core and engine: a page asks; the bar names the site; "Don't Allow" and the page says it was refused; asked again, the bar does not come back. On Android also "Allow", with the permission given to the test |

A page of the test site asks for each thing by a button, and writes what became of it where a test can read it.

## 9. Decisions

| # | Decision | Why |
|---|---|---|
| D1 | The core decides and remembers; the engines' own memory is not used | One rule for every shell, per login set, in the store that is cleared with the login set. An engine that remembered by itself would answer for a Space that was never asked |
| D2 | By origin, and only for the page itself | The origin is what the web's own rules give a permission to. A frame from elsewhere asking in a page's name is the oldest way to borrow a trust |
| D3 | A bar in the tab, not a dialog | A dialog over the window can be put up by any tab and answered by a key meant for something else. A bar belongs to its tab and waits |
| D4 | "Allow" is remembered and "Don't Allow" is not | A call that is made every day should not ask every day. A no that could not be taken back without a settings screen would be a trap; the list of what was allowed is that screen for a yes |
| D5 | Engines made by tests cannot start a capture | A test that raised the system's dialog stopped this Mac's UI suite for hours (007 §8). The seam makes it impossible, not merely not done |

## 10. What will not be verified here, and why

- **Another login set, through the core.** As for zoom and downloads: one Space and one login set until W2. That one login set's permissions are not another's is the store's test.

- **A capture that starts, on Apple.** Granting one makes the system ask, with its own dialog, whether the app may use the camera: no test may raise it, and nobody is at this Mac to answer it. That WebKit starts the capture on "granted", and that the system then asks, will be written and not seen until a person runs the app.
- **Apple, on a machine with no camera or microphone**: WebKit does not ask at all there, and that is every iPhone simulator and GitHub's Macs. The adapter's suite can run only on a Mac that has one.
- **Location on Apple**: not built, and not asked for by any test, since asking may raise the system's own dialog.
- **The system's dialog on Android**: asked through a seam that the tests replace. Its real form is run only by a person.
- **A phone.** Android's part runs on an emulator, with an emulated camera.

## 11. Done checklist

- [x] The store: migration 6; what was allowed. 9 tests.
- [x] Contract 1.10 and its version in every place it is written.
- [x] The core: the rules of §3, the tab's prompt, what was allowed. 23 tests through `FakeShell`, and 2 of an address's origin.
- [ ] The test site's page.
- [ ] Apple: the adapter and its suite; the reasons in `Info.plist`; the bar, its model's tests and pictures; the list; a UI test.
- [ ] Android: the adapter, the system's permission behind a seam, on the JVM and the device; the bar; the list; pictures; a device test.
- [ ] `Tools/test.sh all` green, and `android-device`.
- [ ] Windows shell.
