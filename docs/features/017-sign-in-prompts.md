# 017 — A server asks who is there (W1)

| | |
|---|---|
| **Status** | In progress: the contract (1.13) and the core's rule (§3) are built and tested. No adapter reports a server asking yet, and no shell draws it (§9): until then WebKit and the System WebView give a server that asks no answer, and show what it sent with its refusal |
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

What each does in fact, as observed, is written here when built.

## 6. The shells

The same bar (014 §7). "Sign in to *server*" on a line of its own; when the server refused before, that the name or password was not accepted; when the connection is not private, that what is typed could be read by others; a field for the name and one for the password, which shows dots; **Cancel** and **Sign In**. No button is the default while the page has the keyboard; Return in the password's field is Sign In.

What is typed is the shell's only until it is sent: it is in no snapshot, no log, and no picture.

## 7. Tests

| Where | What |
|---|---|
| Core (`wt-app/tests/sign_in.rs`, 16 through `FakeShell`) | The server a load is heading for asks in the tab, under its own name and not the page's, and the load waits. The name and the password are the server's answer; Cancel is none, whatever was typed; a password with no name. The page's own server may ask. Any other is given no answer with nobody asked: another server while the page sits there or while a load heads elsewhere, the page's host at another port or not over `https`, an origin that cannot be read, a server nobody names, a web view that is no tab's. Named by its host alone, a server is the tab's own by its host, however the host is written, and is taken to be where the load is going before where the page is. Where a redirect took the load is who may ask and who is named. A server not over `https` is named with its scheme and said to be read on the way; this device itself is not. A server that refused asks again and the prompt says so. Not answered in the moment it appears. Every question of the same server and realm that waits has the same answer, in whatever tab, and no other server's or realm's has. A question is over when another load begins, when one is stopped or fails, when a page arrives, and when the renderer is lost; what a page said outlives a load that only begins. Said twice, asked once; four wait and a fifth is given no answer. A server, a page's dialog and a page's request that the shell gave one number are not taken for each other, and a yes to the camera is no answer to a server. A realm is told apart by its first two hundred characters. Twenty-three parts of the rule checked by breaking them; a twenty-fourth showed a condition that did nothing, which was removed |
| The test site | A page that answers only to a name and a password, the same in all three of its servers |
| Adapter suites (WebKit on the Mac and the iPhone simulator; System WebView on the emulator) | A load to that page is reported with who asks and where the load is heading, and waits; the right answer loads the page; a wrong one asks again and says so; no answer shows the server's refusal; a second load needs no second answer; **another login set is asked again** (isolation, as a security property); a web view destroyed while it waits |
| The shells | Against a scripted core: the bar, its lines, its two fields and what each button sends. Pictures. The apps, with the real core and engines: an address that asks, the bar under the server's name, the answer typed, the page |

## 8. Decisions

| # | Decision | Why |
|---|---|---|
| D1 | Only the tab's own server may ask | A password typed in a tab is taken to be for what the tab shows or is going to. Anything else is how a picture in a forum post asks for a bank's password |
| D2 | The server's realm is not shown | It is the server's words, in the one prompt where words must not mislead: "Your bank's session expired" is a realm any server can send |
| D3 | The prompt names the server, not the page | It is the server that gets the password |
| D4 | The engine keeps the answer, for the session | Every engine does it already, per login set; WebTiger's own remembering belongs with passwords, behind the platform's lock |
| D5 | The adapter says where the load is heading | Only the engine knows where a redirect has taken a load; the core is not told of each hop |

## 9. What is not verified

Written when built. Known now:

- **The System WebView names a host, not an origin.** A server at another port of the page's host is asked about as if it were the page's own, and named with the page's scheme and port.

## 10. Done checklist

- [x] Contract 1.13 and its version in every place it is written.
- [x] The core: the rule of §3. 16 tests through `FakeShell`.
- [ ] The test site's page.
- [ ] Android: the adapter, on the JVM and the device; the bar; pictures; a device test.
- [ ] Apple: the adapter and its suite; the bar; pictures; a UI test.
- [ ] `Tools/test.sh all` green, and `android-device`.
- [ ] Windows shell.
