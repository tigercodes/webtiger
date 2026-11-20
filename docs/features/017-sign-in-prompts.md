# 017 — A server asks who is there (W1)

| | |
|---|---|
| **Status** | In progress: the contract (1.13) and the core's rule (§3) are built and tested. Both shells report a server that asks and draw the prompt. Run end to end on Android (an emulator, against the real System WebView) and on Apple against real WebKit: the adapter's suite on the Mac and the iPhone simulator, and the app on both. Windows is not started |
| **Phase** | W1 of [000](000-master-plan.md) §9: "sign-in prompts". A server may ask for a name and a password before it answers at all (HTTP authentication): routers, printers, staging sites and many an intranet do |
| **Contract** | 1.13: `SignInRequested`; the host request `SignInAnswer`; `SignInPrompt`, a fourth kind of `PromptView`; one field of `AnswerPrompt` |
| **Builds on** | [014](014-permissions.md): a tab's prompt, its queue, and the rule that it is not answered in the moment it appears. [016](016-page-dialogs.md): a prompt with a field |
| **Last updated** | 2026-11-20 |

## 1. User story

I go to an address, and before it shows me anything its server asks who I am. WebTiger asks me, in that tab, for a name and a password, and says plainly which server will get them, and whether they can be read on the way. I type them and the page loads. If they are wrong I am told so and asked again. If I would rather not, I say so, and see what the server shows to those it does not know.

## 2. What is in it, and what is not

**In:** a server's question (HTTP authentication that a name and a password answer) reaches the core; the tab asks for both, under the server's name; the answer goes back to the engine, which keeps it for that server until the app quits, in the tab's login set only.

**Not in:**

- **Remembering a password**, or offering one that is known: the platform's passwords and passkeys are their own W1 feature. What is typed here is not written down anywhere by WebTiger.
- **Signing out.** The engine keeps the answer for the server until the app quits; there is no button that makes it forget sooner.
- **Proxies.** A proxy that asks for a password is not answered.
- **Certificates**, and whatever else a name and a password do not answer: the engine's, as before. A server's certificate that cannot be trusted is the next feature's.
- **The server's own words** (its "realm"). They are not shown (D2).

## 3. Who does what

The engine notices that a server asks, and says so; the load waits. The core decides whether the person is asked. The shell draws the prompt in the tab and sends the answer; the engine gives it to the server.

**The rule.** A server's question is put before the person only when the server is **the tab's own**:

1. the server the tab's load is heading for (redirects followed: where the load is now going, not where it began), or
2. the server of the page the tab shows.

Any other server is given no answer, with nobody asked: a picture or a frame from elsewhere must not be able to ask for a password under this page's roof. So is a web view that is no tab's, and a tab that already has as many prompts as it may (014 §3).

"The same server" is the same origin where the engine names the server's origin (WebKit). The System WebView names only its host: there it is the same host, and the server is taken to be at the scheme and port of the address that matched (§9).

**What the prompt says** is all the core's: the server, written as an address field writes a site (so with `http://` when it is not `https`); whether the connection is private (`https`, or this device itself), and when it is not, the shell says that what is typed can be read on the way; and whether the server has refused an answer already. Nothing in the prompt is the server's own words.

**The answer.** "Sign In" sends the name and the password to the engine, for that request. "Cancel" gives the server no answer: the engine shows what the server sent with its refusal. Either way **every other question of the same server and realm that waits in the login set has the same answer**, in whatever tab: five tabs of one intranet, restored together, are signed in to once.

A wrong answer makes the server ask again: the engine reports it as a new question, and says it is not the first. There is no limit to how often; "Cancel" ends it.

**It ends with its load.** A question that waits is over when its web view starts another load, when a load of its fails or is stopped, when a page commits, and when its renderer is lost: the engine is told to give no answer (it usually has none to give by then). When its tab closes, the web view is destroyed, and the adapter gives the engine its answer for whatever waited there.

What a page said (016) and what it asked to use (014) are the page's, and outlive a load that only begins; a server's question does not.

The rest is 014's, unchanged: one prompt at a time, three behind it; not answered in its first half second, nor from a tab behind.

## 4. The contract (1.13)

Added; nothing is changed or removed.

```proto
// engine.proto
message SignInRequested {
  uint64 request_id = 1;                    // the shell's own, never used twice
  string host = 2;                          // redacted. The server that asks
  string origin = 3;                        // redacted. Its origin, where the engine names it; else empty
  string realm = 4;                         // redacted. The server's name for what it protects. Never shown
  string heading = 5;                       // redacted. Where the web view's load was heading, redirects
                                            // followed; empty when no load was in flight
  bool again = 6;                           // the server refused an answer already
}

// host.proto
message SignInAnswer {
  uint64 request_id = 1;
  bool accepted = 2;                        // false: no answer is given to the server
  string name = 3;                          // redacted
  string password = 4;                      // redacted
}

// browser.proto
message PromptView {
  ...
  oneof kind {
    ...
    SignInPrompt sign_in = 5;               // since 1.13
  }
}

message SignInPrompt {
  string site = 1;                          // redacted. The server, as an address field shows a site
  bool confidential = 2;                    // what is typed reaches it unread: https, or this device
  bool again = 3;                           // the server refused what was sent before
}

message AnswerPrompt {
  ...
  string secret = 6;                        // redacted. Since 1.13: the password, with `text` as the name
}
```

`confidential` is false unless the core says otherwise: a shell that is told nothing warns.

An adapter does not report a proxy's question, nor one that a name and a password do not answer.

## 5. The engines

| | WebKit | System WebView |
|---|---|---|
| Where it is told | The navigation delegate's authentication challenge | `WebViewClient.onReceivedHttpAuthRequest` |
| Left alone | No answer is given: the server's refusal is shown | The same |
| Who asks | The challenge's protection space: scheme, host, port, realm | Host and realm only |
| Not the first time | The challenge's count of failures | The handler says the stored answer is no longer to be used |
| The answer | A credential kept for the session | `proceed(name, password)` |
| No answer | The challenge is cancelled | `cancel()` |

**As built on Apple** (`SignIns` in the WebKit adapter; the navigation delegate's authentication challenge), and as seen against real WebKit on the Mac and the iPhone simulator:

- **WebKit names the server's origin**: the challenge's protection space has its scheme, host and port. The adapter writes it without the port the scheme implies, as an address does.
- Only a server's question that a name and a password answer is reported. A proxy's, a certificate's and the rest are left to WebKit's own handling, as before.
- **The load waits**, as on Android. **Where the load is heading** is WebKit's own address for the web view while a load has not committed: it follows redirects.
- **A wrong answer** asks again within the same load, with WebKit's count of failures above nought: reported as "again".
- **No answer** is given as "go on without one": what the server sent with its refusal is shown, as a page at the address asked for. (Told instead to cancel, WebKit showed the same page: the suite does not tell the two apart.)
- **WebKit keeps an answer** given with "for the session", and it stays in its login set: another web view of the same login set was not asked; one of another login set was.
- **After a redirect to another server, WebKit asks twice.** The answer given to a server a load was sent on to was not used for that page's own picture: the server asked once more, now as the page's own server, and not as a refusal. That second answer is kept. So a person sent on to a protected server may be asked twice in a row there. Seen here; why is WebKit's.
- **A picture from another server is not asked about at all**, as on Android.
- A web view that is destroyed gives WebKit its answer for what waited: WebKit stops an app that never does.

**As built on Android** (`SignIns` in the adapter; `onReceivedHttpAuthRequest`), and as seen on the emulator (System WebView 133):

- **The WebView names the server's host and nothing more**: for the test site, `127.0.0.1`, with no port and no scheme. So the adapter reports a host and no origin, and the core goes by the host (§3, §9).
- **The load waits.** Half a second after the server asked, no page had come of the load; it came when the core's word did.
- **Where the load is heading** is what the adapter's own tracker knows: the address it was told to load, and after that each address the navigation policy let the main frame go on to. A load sent on to the server's other name was reported as heading there. Once a page has arrived nothing is heading anywhere, and a picture of the page's own server that asks is reported so.
- **A wrong answer** makes the server ask again within the same load: the WebView says its stored answer is no longer to be used, and the adapter reports that as "again".
- **No answer** (`cancel()`): what the server sent with its refusal is shown, as a page at the address asked for, and nobody is asked again.
- **The WebView keeps an answer** for the server, and it stays in its login set: another web view of the same login set loaded the page with nobody asked; a web view of another login set was asked. Tested as isolation is.
- **A picture from another server is not asked about at all**: this WebView gives it no answer by itself, and reports nothing. (The core's rule would say the same of an engine that did ask.)
- A web view that is destroyed gives the WebView its answer for what waited: none.

## 6. The shells

The same bar (014 §7). "Sign in to *server*" on a line of its own; when the server refused before, that the name or password was not accepted; when the connection is not private, that what is typed could be read by others; a field for the name and one for the password, which shows dots; **Cancel** and **Sign In**. No button is the default while the page has the keyboard; Return in the password's field is Sign In.

What is typed is the shell's only until it is sent: it is in no snapshot, no log, and no picture.

**As built on Apple** (`SignInBar`; `BrowserModel.answerPrompt`): the same bar on the Mac and the iPhone, with the system's own fields: the name's, and a secure one for the password, each with its name shown in it while empty and read as its label. Return in the name's field goes to the password; in the password's, it signs in. Each prompt is a bar of its own, so a server that asks again starts with empty fields. The fields are not marked as a name and a password for the system's own passwords to fill: that is the passwords feature's (§2).

**As built on Android** (`SignInBar`; `BrowserModel.answerPrompt`): as above. Each field's name is shown in it while it is empty, and is what a screen reader calls it. The password is drawn as dots, and the field is a password to the keyboard and to assistive services. The keyboard's Next, in the name's field, goes to the password; its Done there signs in. Cancel passes on nothing that was typed. What is typed is kept in the bar and not in the window's saved state: a password is not written down to be restored, so turning the device empties the fields. A server that asks again is a new prompt, and starts empty.

## 7. Tests

| Where | What |
|---|---|
| Core (`wt-app/tests/sign_in.rs`, 16 through `FakeShell`) | The server a load is heading for asks in the tab, under its own name and not the page's, and the load waits. The name and the password are the server's answer; Cancel is none, whatever was typed; a password with no name. The page's own server may ask. Any other is given no answer with nobody asked: another server while the page sits there or while a load heads elsewhere, the page's host at another port or not over `https`, an origin that cannot be read, a server nobody names, a web view that is no tab's. Named by its host alone, a server is the tab's own by its host, however the host is written, and is taken to be where the load is going before where the page is. Where a redirect took the load is who may ask and who is named. A server not over `https` is named with its scheme and said to be read on the way; this device itself is not. A server that refused asks again and the prompt says so. Not answered in the moment it appears. Every question of the same server and realm that waits has the same answer, in whatever tab, and no other server's or realm's has. A question is over when another load begins, when one is stopped or fails, when a page arrives, and when the renderer is lost; what a page said outlives a load that only begins. Said twice, asked once; four wait and a fifth is given no answer. A server, a page's dialog and a page's request that the shell gave one number are not taken for each other, and a yes to the camera is no answer to a server. A realm is told apart by its first two hundred characters. Twenty-three parts of the rule checked by breaking them; a twenty-fourth showed a condition that did nothing, which was removed |
| The test site | A page that answers only to a name and a password, the same in all three of its servers |
| System WebView adapter, on the JVM (`SignInsTest`, 4) | Through the engine, as the core speaks to it: a server that asks is reported with its host, no origin, its realm, where the load is heading now and not where it began, and whether it refused before; and the load waits. With no load on its way nothing is said of where one is heading. The core's word is the server's answer, a name and a password or none, whatever came with a no; each question has a name of its own and is answered once. A web view that is destroyed gives its servers no answer, and not another's. Ten of the rules checked by breaking them |
| System WebView adapter suite (`SystemWebViewSignInTest`, 8, on the emulator) | Against the real WebView and the test site. A server that asks is reported and the load waits for the core's word; its page then loads, and its picture asks nothing more. A wrong answer asks again and says so. No answer shows what the server sent with its refusal. **What the engine keeps of an answer stays in its login set**: the same login set is not asked again, another is. Where a redirect took the load is where it is heading. A picture of the page's own server asks with no load on its way; a picture from another server is given no answer by the WebView itself. A web view destroyed while a server waits |
| WebKit adapter suite (`WebKitSignInTests`, 10, on the Mac and the iPhone simulator) | Against real WebKit and the test site. A server that asks is reported with its origin, its realm and where the load is heading, and the load waits; its page then loads, and its picture asks nothing more. A wrong answer asks again and says so. No answer shows what the server sent with its refusal. **What WebKit keeps of an answer stays in its login set.** Where a redirect took the load is where it is heading, and there WebKit asks once more for the page's own picture. A picture of the page's own server asks with no load on its way; one from another server is given no answer by WebKit itself. A web view destroyed while a server waits; and, at the seam where WebKit's answer can be seen, that its servers get none, another web view's still waits, the core's yes is the name and the password each as itself, and its no is no answer. A server is written as an origin without the port its scheme implies, and a certificate is not a question a password answers. Twelve of the rules checked by breaking them; one more (cancel in place of "go on without") made no difference WebKit showed |
| The Android bar (`SignInBarTest`, 3, drawn on the JVM against `ScriptedCore`) | The bar names the server and asks for a name and a password, in two empty fields each read by its name, the second a password; nothing has the keyboard by appearing. Sign In sends both, Cancel neither; the password is drawn as dots; the keyboard's Done in the password's field signs in; another question starts empty. A server that refused before and a connection that is not private are each said, and neither without cause. Pictures of the bar, and of the bar saying both, light and dark, each looked at. Ten of the rules checked by breaking them |
| The Android app, on the emulator | With the real core and WebView, in fake mode: an address that asks is typed; the tab asks under the server's name while the page before it stays, and Android's accessibility checks pass over the bar; a wrong password is said to be one, with the fields empty again; the right one loads the page; and in a new tab the same address loads with nobody asked |
| The Apple bar (`SignInModelTests`, 2, against `ScriptedCore`) | The model has the server's question, with what the core says of it; an answer carries the tab, the prompt's name, the name and the password, and a no carries neither; the prompt stays until the core says it is gone. The bar's first line is the server. Pictures of the bar, and of the bar saying both things, light and dark, on the Mac and the iPhone, each looked at (the taller one in a taller picture). Three of the rules checked by breaking them |
| The Apple apps | With the real core and WebKit, in fake mode: the app starts on an address that asks; the tab asks under the server's name, and Apple's audit passes over the bar; a name and a wrong password are typed, and the bar says they were not accepted, the audit passing again; the right one is typed, and the page loads with nothing more asked |

## 8. Decisions

| # | Decision | Why |
|---|---|---|
| D1 | Only the tab's own server may ask | A password typed in a tab is taken to be for what the tab shows or is going to. Anything else is how a picture in a forum post asks for a bank's password |
| D2 | The server's realm is not shown | It is the server's words, in the one prompt where words must not mislead: "Your bank's session expired" is a realm any server can send |
| D3 | The prompt names the server, not the page | It is the server that gets the password |
| D4 | The engine keeps the answer, for the session | Every engine does it already, per login set; WebTiger's own remembering belongs with passwords, behind the platform's lock |
| D5 | The adapter says where the load is heading | Only the engine knows where a redirect has taken a load; the core is not told of each hop |

## 9. What is not verified

- **The System WebView names a host, not an origin.** A server at another port of the page's host is asked about as if it were the page's own, and named with the page's scheme and port. Not tried: the test site has one port.
- **Only HTTP Basic was run.** Digest and NTLM are answered by the engines with the same name and password; neither was tried against a server.
- **A proxy that asks.** Not reported by an adapter that can tell (WebKit); the System WebView does not say which a question is, and the core gives a proxy no answer because its host is not the tab's. Not tried.
- **That the keyboard is told the field is a password**, so that it suggests and learns nothing: how the field is written. No test on the JVM sees it; a test that makes the field an ordinary one in every other way was not noticed.
- **What is typed, in memory.** A password is held as an ordinary string in the bar, in the contract's messages and in the core until each is dropped; none of them is wiped.
- **The tall bar and the keyboard**, on a small screen held sideways: not tried.
- **Android: a phone.** It ran on an emulator.
- **What an Apple bar's buttons send.** That Cancel passes on nothing typed, and that Return in the password's field signs in, are how the bar is written; the app's test presses Sign In. (On Android both are tested.)
- **The system's offer to save a password**, on a device that has its passwords set up: the fields are not marked for it, and no simulator here has them set up, so none was seen.

## 10. Done checklist

- [x] Contract 1.13 and its version in every place it is written.
- [x] The core: the rule of §3. 16 tests through `FakeShell`.
- [x] The test site's pages: one that answers only to a name and a password, its picture, a redirect to it, and a page that shows the picture from its own server or another.
- [x] Android: the adapter, on the JVM (4) and the device (8); the bar and its tests (3); pictures; a device test of the app.
- [x] Apple: the adapter and its suite against real WebKit (10); the bar and its model's tests (2); pictures; a UI test.
- [x] `Tools/test.sh all` green, and `android-device` (120 on the emulator).
- [ ] Windows shell.
