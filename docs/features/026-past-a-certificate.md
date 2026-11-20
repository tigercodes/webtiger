# 026 — Going on past a certificate that is not trusted (W1)

| | |
|---|---|
| **Status** | In progress. The contract and the core's rule are built and tested (§3, §4). No adapter holds a certificate yet, so nothing can be gone on past in an app: the engines and the shells are next |
| **Phase** | W1 of [000](000-master-plan.md) §9: "error and certificate pages". [021](021-certificate-errors.md) says which kind of trouble a certificate has; this is the way past it that 021 left for the owner to decide |
| **Contract** | 1.20: `NavigationFailed.certificate_held`; `TabView.certificate_passable`, `certificate_host`; `ProceedPastCertificate`; the host service `WebViewTrustCertificate` |
| **Builds on** | [021](021-certificate-errors.md): `net.tls.untrusted`, `net.tls.date`, `net.tls.name`. [001](001-foundations.md): the failed page, and the mark beside the address |
| **Last updated** | 2026-11-20 |

> **The owner's decision (2026-11-20): a person may go on past a certificate that is not trusted, with a warning before.** 021 made no way past, and left the question open (021 D1).

## 1. User story

My router's page, or a device on my own network, has a certificate that nobody signed. WebTiger does not show the page, and says why. I know why, and I want to go on: I ask for it, WebTiger tells me plainly what going on means, and I go on. The page loads, and the address field goes on saying that this connection is not one to trust.

I do not get there by accident: it takes two deliberate presses, and the second is on a button that says what it does.

## 2. What is in it, and what is not

**In:** a failed load of a tab's own page, for one of the three kinds of certificate trouble, can be gone on with; the warning first; the exception that is then made, and what it covers; the mark beside the address on every page loaded under it.

**Not in:**

- **Remembering it after the app is closed.** The exception lasts until the app quits, and is written nowhere.
- **Looking at the certificate**: who it names, who signed it, its dates. Later, as in 021.
- **A list of the exceptions**, and taking one back before the app quits. Later, with settings.
- **Anything but the tab's own page.** A picture, a frame or a script from a server whose certificate is not trusted stays refused: nobody is asked about what they cannot see.
- **A connection that could not be made secure at all** (`net.tls`: no certificate was the trouble). There is nothing to accept.

## 3. The rule

**What may be gone on with.** A tab's load fails with `net.tls.untrusted`, `net.tls.date` or `net.tls.name`, it is the load of the tab's own page, and the adapter says it **holds the certificate it refused** (it can be told to let that one through). The tab's view then says so (`certificate_passable`), and names the host whose certificate it was (`certificate_host`). Any other failure is as it was.

**The warning is the shell's, and comes first.** The failed page says what is wrong (021's words). It does not offer to go on: it offers to say more. What it then says is what going on means: somebody could be pretending to be this site, and could read and change what is sent and received. Under that is the one button that goes on, and it says so: "Go on to ‹host› (not safe)".

**Going on** (`ProceedPastCertificate`, for a tab) is the person's and nobody else's: no page can ask for it. The core:

- refuses it for a tab whose failed load is not one that may be gone on with (`certificate.not_passable`), and for no tab (`tab.unknown`);
- makes the exception (below), tells the shell to let the certificate through for that web view (`WebViewTrustCertificate`), and loads the address that failed again, in that order.

**The exception** is for **one certificate, at one host and port, in one login set, until the app quits**:

- *one certificate*: the adapter lets through the certificate it refused and no other. Another certificate at the same host is refused again, and has to be gone on with again;
- *one host and port*: nothing is accepted for any other site;
- *one login set*: another Space with its own logins is asked for itself. Spaces that share logins share the exception, as they share everything of their connection to a site;
- *until the app quits*: it is kept in memory, by the core and by the adapter, and in no store.

**A page under an exception is never shown as secure.** For as long as the core has an exception for a host and port in a login set, every page of that host and port in that login set is marked not secure beside the address: in every tab, however it was reached, and whether or not the certificate has since been put right. The padlock is for a connection the system trusts, and this is not one.

**Nothing else changes.** History, downloads and what a site was allowed are as for any page. A page under an exception that asks for the camera is asked about as any page is: its origin is one a person chose to visit.

## 4. The contract (1.20)

Added; nothing is changed or removed.

```proto
// engine.proto
message NavigationFailed {
  ...
  // With "net.tls.untrusted", "net.tls.date" or "net.tls.name", for the web
  // view's own page: the adapter holds the certificate it refused, and can
  // be told to let it through (WebViewTrustCertificate).
  bool certificate_held = 4;
}

// browser.proto
message TabView {
  ...
  bool certificate_passable = 15;     // the failed load may be gone on with
  string certificate_host = 16;       // redacted. Whose certificate it was: for the warning's words
}

message ProceedPastCertificate {      // answered with Ack, "tab.unknown" or "certificate.not_passable"
  uint64 tab_id = 1;
}

// host.proto
// WebViews: lets through, for this web view's login set, the certificate
// this web view last refused, at the host and port it was refused for: that
// certificate and no other, until the app quits. Answered with Ack;
// "webview.unknown", or "certificate.unknown" when the adapter holds none.
message WebViewTrustCertificate {
  uint64 web_view_id = 1;
}
```

Errors: `certificate.not_passable` (the core's), `certificate.unknown` (an adapter's).

## 5. The engines

Written with each adapter. What both must do:

- when the engine asks about a server's certificate that the system does not trust: let it through only if this web view's login set has an exception for that host and port and **this** certificate (compared by a hash of the certificate itself); otherwise refuse, as now, and keep what was refused, for the web view;
- say with the failure that the certificate is held, when the failure is the page's own and the certificate refused was that page's server's;
- on `WebViewTrustCertificate`, make the exception from what the web view last refused;
- forget every exception when the process ends, by keeping none anywhere else.

**The System WebView, as built** (`Certificates.kt`; `WebViewPage.onReceivedSslError`). The WebView asks the adapter about every certificate it does not trust, with the address and the certificate. The adapter tells one certificate from another by a SHA-256 of the certificate itself, and one site from another by scheme, host and port, the port written also where `https` implies it. Let through: only what a web view of that login set was told to (`handler.proceed()`); everything else is cancelled, as before. A certificate is held only for the page's own load and only for the three kinds of trouble; what a page loads from such a server is cancelled and not held. A web view that is destroyed holds nothing; what was let through stays, for its login set, until the process ends.

Seen on the emulator (WebView 133), with a certificate that signs itself, made on the device: the failure says the certificate is held, and the site is not asked for its page; told to let it through, the adapter does, and the page loads; another web view of the same login set loads it without being asked again; **a web view of another login set is refused again**, so what the WebView itself remembers of a certificate that was let through is the profile's own; and **the same host and port with a certificate made anew is refused again**, held, and its page not asked for, until that one is gone on with too.

## 6. The shells

Written with each shell.

## 7. Tests

| Where | What |
|---|---|
| Core (`wt-app/tests/certificates.rs`, 8, through `FakeShell`) | A load that fails for a certificate the adapter holds may be gone on with, and the tab names the host: for each of the three kinds; nothing is let through by saying so. Any other failure is as it was, and going on is refused: a tab with no page, no tab, a site that is not there, a page that loaded, a connection that could not be made secure at all, a certificate the adapter does not hold, an address that is no page over `https`, another failure that claims a certificate. Going on has the shell let the certificate through and then load, in that order; the page loads, and is marked not secure; so is every page of that host however it is reached, also once the certificate has been put right; a site the system trusts is as it was; the certificate is let through once. The exception is one login set's: a Space with logins of its own is asked for itself, and is not marked by another's exception; a Space that shares logins shares it. Another certificate at the same host is refused again, and may be gone on with again. An adapter that no longer holds the certificate makes no exception: the shell's own code is the answer, and the page is as secure as any once its certificate is trusted. Another load, the page's own or the person's, and a load that is stopped, end what may be gone on with. The exception is gone when the app starts again. Thirteen of the rules checked by breaking them: all caught |
| The System WebView adapter's rule (`CertificatesTest`, 5; `PageTrackerTest`, 1 more; `SystemWebViewEngineTest`, 1 more) | Nothing is let through until a web view is told to: refusing and holding lets nothing through, and a web view that refused nothing holds nothing. What a web view last refused is let through in its login set and no other; not another certificate at the same host, nor the same certificate at another host or port. A web view holds the last it refused, and nothing once it is gone, while what was let through stays. Whose certificate an address presents: its host and port over `https`, however the address is written; nothing over `http`, `wss`, or no address. A certificate is told from another by a hash of itself. A held certificate is said to be with the failure, and only for the page's own load. The engine answers `webview.unknown` for no web view, `certificate.unknown` for one that holds nothing, and lets through in the web view's own login set |
| The System WebView adapter suite, on the emulator (`SystemWebViewCertificateTest`, 2 more, and 1 changed) | With a certificate that signs itself, made on the device: the failure says it is held; a web view that refused nothing answers `certificate.unknown`; told, the certificate is let through, the page loads with the site's title, and the site was asked for its page once; another web view of the same login set loads it; one of another login set is refused, held, and the site not asked. The same host and port with a key and certificate made anew: refused again and held, its page never asked for; gone on with, it is that one that loads. Three of the rules broken on purpose and run on the emulator: each caught |
| Apple's adapter suite | Written with it |
| The shells | Written with each |

## 8. Decisions

| # | Decision | Why |
|---|---|---|
| D1 | A person may go on, with a warning before | The owner's, 2026-11-20. A router, a printer, a device of one's own: without a way past, they cannot be reached at all |
| D2 | Two presses, and the second says what it does | The warning is read before the button that goes on can be pressed; a failed page with "go on" on it is pressed without reading |
| D3 | One certificate, one host and port, one login set, until the app quits | The smallest exception that reaches the page. A certificate that changes is somebody else's until the person says otherwise; an exception that outlives the app is one nobody remembers making |
| D4 | The page is marked not secure for as long as the exception lasts | The address field never lies (CLAUDE.md): a padlock would say the system vouches for the site |
| D5 | Only the tab's own page | What a person is asked about, they can see in the address field. A script from an untrusted server is not that |
| D6 | The core keeps the exceptions as well as the adapter | The core marks the page; the adapter lets the connection through. Neither can do the other's part |

## 9. What is not verified

Written when built.

## 10. Done checklist

- [x] Contract 1.20 and its version in every place it is written.
- [x] The core: the rule of §3 (8 tests).
- [x] Android: the adapter, against the real WebView with a certificate that signs itself (on the emulator).
- [ ] Apple: the adapter, against real WebKit with a certificate that signs itself.
- [ ] The shells: the warning, and going on.
- [ ] `Tools/test.sh all` green, and `android-device`.
- [ ] Windows shell.
