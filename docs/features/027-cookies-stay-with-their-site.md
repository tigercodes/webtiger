# 027 — Cookies stay with their site (W1)

| | |
|---|---|
| **Status** | Built and tested. Android's adapter fetches downloads itself, each address with its own cookies: on the JVM, on an emulator against the WebView's own cookie store, and by hand in the app started for real. WebKit's own behaviour is seen on the Mac for a download a page starts and for a file a menu saves. Windows is not started |
| **Phase** | W1 of [000](000-master-plan.md) §9: downloads. A security matter found in [022](022-picture-menu.md) and left open in [013](013-downloads.md) §9 |
| **Contract** | No change |
| **Builds on** | [013](013-downloads.md): a download, and who moves its bytes. [022](022-picture-menu.md): a file a person chose, asked after hop by hop |
| **Last updated** | 2026-11-20 |

> **The owner's decision (2026-11-20): cookies must not leak across sites, on any system.**

## 1. User story

I am signed in to a site. Something of that site is fetched for me, a download, and the address it is at sends the fetch on to another site. What the first site knows me by is not handed to the second. That holds on every system, however the fetch was started, and whoever moves the bytes.

## 2. The rule

**A cookie is sent to the site it belongs to, and to no other.** Every request made for a person is made with the cookies that the login set holds *for the address being asked*, worked out afresh for each address. When an answer sends the request on, the next address is a new request: it has its own cookies, or none.

The engines keep this rule in everything they fetch themselves: pages, what pages load, and the downloads WebKit moves. It can only be broken where a **shell makes a request of its own** and puts cookies on it by hand. So the rule for shells is:

- a shell that sends a cookie itself follows whoever sends it on **by hand**, one address at a time, and never lets anything else follow for it with those cookies still attached;
- nothing but cookies is carried either: no sign-in (`Authorization`), and no address of the page that asked.

## 3. Where a shell sends cookies itself

Looked for in both shells' sources (2026-11-20):

| Where | What it sends | As it was | As built |
|---|---|---|---|
| **Android: a download's bytes** (013) | The login set's cookies for the download's address, to the system's `DownloadManager` | **Broken.** The system's downloader follows whoever sends it on, and sends the cookies it was handed to each of them: seen on the emulator in 022. A file a person chose was followed by hand first (022), which left a hole: an address that sends on only the second time it is asked. A download a page started was not followed at all | The adapter fetches the bytes itself (§4) |
| **Android: asking after a chosen file** (022, `FileAsker`) | The cookies of each address on the way, to that address | Right: followed by hand, each site with its own | Unchanged |
| **Apple** | Nothing. Every request is WebKit's: a page, a download, a file a menu saves (`startDownload`) | Right, and seen for a file a menu saves (022) | Unchanged; seen also for a download a page starts (§6) |

Nothing else in either shell reads a cookie, writes one, or makes a request: the core does no I/O, and the shells make none but these.

## 4. Android: the adapter fetches downloads itself

The system's `DownloadManager` is no longer used. A fetcher of the adapter's own (`AppFetcher`) moves a download's bytes, in the app's process, off the main thread:

- **One address at a time.** It asks for the address with the login set's cookies *for that address*, and follows nobody automatically. An answer that sends on (301, 302, 303, 307, 308 with a place to go) is followed by hand: the next address is asked with *its* cookies. It follows ten times at the most, and only to an address of the web.
- **A cookie a site sets on the way is kept, for that site**, in the login set, as it would be had a page gone the same way: the next address of the same site sees it. (The WebView's own cookie store takes it, and applies its own rules about whose a cookie may be.)
- **Nothing else is carried**: what the WebView says it is (`User-Agent`), and the cookies. No sign-in, no referring address.
- **The bytes are saved as they come**, into a file under the name the core made: in the device's Downloads through the system's `MediaStore` (the app started for real), which needs no permission, or in the app's own folder (tests and fake mode). A name that is taken gains a number, and the name used is reported. A file that is not whole is not left behind: one that fails or is dropped is removed.
- **How it goes** is asked as before (`Fetcher.status`): how much has come, of how much; whole; or failed, with one of the contract's codes (`download.network`, `download.disk`, `download.failed`).
- **A certificate that is not trusted fails the download.** The fetch trusts what the system trusts, and an exception a person made for a page ([026](026-past-a-certificate.md)) is not one for a download.

**What is lost by leaving the system's downloader:**

- **The system's notice** of a download in the status bar, which was how a person opened a file that had come (013 §6). The file is in the device's Downloads, where the system's Files app shows it, and in WebTiger's list; the list still cannot open it. Opening a file from the list was planned with the Library (013 §2, W3); with the notice gone it was wanted sooner, and is built: [030](030-open-a-download.md).
- **A download that outlives the app.** The system's downloader went on when the app was closed. The adapter's goes on while the app's process lives, as WebKit's does on Apple's systems.

## 5. Decisions

| # | Decision | Why |
|---|---|---|
| D1 | The adapter fetches itself, and the system's downloader goes | The owner's: cookies must not leak. The downloader cannot be told to drop the cookies when it is sent on, and following by hand first (022) leaves a hole for an address that answers differently the second time |
| D2 | A cookie set on the way is kept, for its site | It is what a page going the same way would do, and a download that signs in on the way needs it. The cookie store decides whose it is, not the adapter |
| D3 | No notice, and no life after the app, for now | Each wants a permission or a service of its own, with a person asked. They are features; the leak was a flaw |

## 6. Tests

| Where | What |
|---|---|
| The fetcher's rule (`AppFetcherTest`, 14, on the JVM) | With scripted sites: a file where its address says is fetched as its site knows the person, and how far it has got is said; a file that is somewhere else is fetched there with that site's cookies and no other's, and a site the person is nobody at is asked with none; every kind of sending on (301, 302, 303, 307, 308) is followed by hand, and a path is within its own site; a cookie a site sets on the way is kept for that site before the next address is asked, and the other site sees only its own; following stops after ten times, at an address that is not the web's (`file:`, `content:`, `javascript:`, `intent:`, `ftp:`), and where an answer sends on to nowhere; what is no file (nobody answering, 403, 404, 500, an answer with nothing in it, 300, 304) is a failure and nothing is saved; a file that is not whole is not left behind (fewer bytes than the site said, a connection that ends, no room, a file that cannot be finished, nowhere to save); a fetch that is stopped asks nobody else and leaves nothing. The file itself: saved under its name, a name that is taken gains a number, a file that is dropped is gone. **And over real connections**, to a server in the test's process that two names reach (`127.0.0.1` and `localhost`: two sites): one site's cookie is not sent to the site it sends on to, which is asked as it knows the person or as nobody; no sign-in and no referring address is sent to anybody; a cookie set on the way is kept for its site and seen by it; a failure is a code and leaves no file; only an address of the web is fetched; a name is held to a name. Fourteen parts of the rule checked by breaking them, each caught: among them the connection left to follow by itself, which sends the first site's cookie on |
| On the emulator, with the WebView's own cookie store (`SystemWebViewDownloadTest`, 2 more: 11) | **The fetcher itself**, in a login set signed in at one site: that site's file says who is signed in; the same file at an address that sends on to the other site is fetched there as nobody (the system's downloader fetched it as the person: 022); a cookie set on the way is in the file, and a page of that site then sees it, while a web view of another login set is nobody still. **A download a page starts** whose file is at the other site is fetched there as nobody; signed in there too as somebody else, as that site knows them; and the first site knows them as it did. The nine tests there were pass unchanged over the new fetcher: the name the core gives, a name that is taken, a name that is a path, a download that is dropped. Three of the rules broken and run on the emulator: each caught, with the file saying who the first site says is signed in |
| WebKit (`WebKitDownloadTests`, 1 more; `WebKitMenuTests` as in 022) | A download a page starts whose file is at the other site: reported as where the file is, fetched there as nobody; signed in there as somebody else, as that site knows them; and the first site knows them as it did. WebKit's own doing: the shell sends no cookie |
| By hand, once (2026-11-20, the emulator, the app started for real and not in fake mode, the test site served from the Mac) | Signed in at the site; a finger on the link to a file that is at the other site; the site's first download was asked about and allowed. The file was in the device's Downloads, whole (the system's record said it was no longer pending, and the app's own), under the name the core gave; it said nobody was signed in. The same link again: a second file, with a number after its name. WebTiger's list had both, each with its name, its site and its size. Nothing was in the app's own folder. The files and the app's data were removed afterwards |

## 7. What is not verified

- **A phone.** Android ran on an emulator.
- **A file of any size.** The tests' files are a few bytes; one by hand was 8. How a large file goes, how often progress is said, and a network that drops part way were not tried against a real network.
- **A download while the app is in the background, or closed.** It goes on while the app's process lives; nothing was measured of how long the system lets that be.
- **The device's Downloads by a test.** No test writes where a person's files are (013): that path (`MediaStore`) was seen by hand, once.
- **A WebView with no profiles** (an older one). There every login set shares one cookie store (003), and the fetcher reads that store: not run.
- **What a site makes of a fetch with no referring address**, and a download that needs a sign-in the browser was asked for ([017](017-http-sign-in.md)): neither is sent, as before.
- **A phone of Apple's**: the WebKit test passed on the Mac and on the iPhone simulator.

## 8. Done checklist

- [x] The fetcher's rule, on the JVM: each address its own cookies, a cookie kept for its site, where following stops, what failure is which code.
- [x] On the emulator, against the real WebView's cookie store: a download sent on to another site is fetched there as nobody, and as that site knows the person; a cookie set on the way is seen by the same site.
- [x] By hand, in the app started for real: the device's Downloads.
- [x] Apple: a download a page starts that is sent on to another site, against real WebKit.
- [x] 013 and 022 say what changed.
- [ ] `Tools/test.sh all` green, and `android-device`. Run on 2026-11-20 with everything but the Mac's UI suite, as [028](028-search-from-other-apps.md) §9 says: all passed. (The first whole run on the emulator failed one of this feature's own tests, which pressed a link when the page's title had come and before the page was whole: the test now waits for the load to end, on both systems.)
