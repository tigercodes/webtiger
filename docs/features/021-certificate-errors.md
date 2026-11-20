# 021 — A certificate that is not trusted says why (W1)

| | |
|---|---|
| **Status** | Built and tested in the Android shell (against the real System WebView, on an emulator, with a certificate that signs itself) and the Apple shell (WebKit's kinds as the adapter's rule; no certificate is presented to WebKit, §7). There is still no way past a certificate that is not trusted (§2). Windows is not started |
| **Phase** | W1 of [000](000-master-plan.md) §9: "error and certificate pages". Error pages are there (001); this is what they say of a certificate |
| **Contract** | No new message. Three more codes of `NavigationFailed`, under the one there is: `net.tls.untrusted`, `net.tls.date`, `net.tls.name` |
| **Builds on** | [001](001-foundations.md): a load that fails is reported with a code, and the shell has the words |
| **Last updated** | 2026-11-20 |

## 1. User story

I go to a site and WebTiger will not show it, because the site's certificate cannot be trusted. It tells me which kind of trouble it is: the certificate is not one this device trusts, or its dates do not hold today, or it is another site's. That tells me what to do next: check the address, check this device's clock, or stay away.

## 2. What is in it, and what is not

**In:** the adapters say which kind of certificate trouble a failed load had, where the engine tells them; the shells have words for each.

**Not in:**

- **Going on all the same.** There is no way past a certificate that is not trusted: the page is not shown. Whether a person may ever be let past one, for a router or a device of their own, is a decision about what the browser promises, and is the owner's to make; it is not made here. Nothing in this feature is in its way.
- **Looking at the certificate**: who it names, who signed it, its dates. Later.
- **A page of the engine's own.** Neither engine's own error page is shown: the tab shows WebTiger's failed page, as for every failure (001).

## 3. The codes

A load that fails because of the site's certificate is reported, as before, with a code that begins `net.tls`. Where the engine says more, the adapter says more:

| Code | What it means | The words |
|---|---|---|
| `net.tls.untrusted` | The certificate is not one this device trusts: it signs itself, or whoever signed it is not known here | "This device doesn't trust this site's certificate." |
| `net.tls.date` | The certificate's dates do not hold on this device's date: it has run out, or has not begun | "This site's certificate isn't valid today. It may have run out, or this device's date may be wrong." |
| `net.tls.name` | The certificate is another site's | "This site's certificate is for a different site." |
| `net.tls` | Anything else that kept a secure connection from being made, a certificate's or not | "A secure connection to the site couldn't be made." (as before) |

The core passes a code through as it always has. **A shell that has no words for a code says what it has for the code above it** (`net.tls` for `net.tls.date`), and only then the general sentence: a newer core's finer code is never worse than the one it refines.

Nothing is accepted on a person's behalf, by any adapter, for any of these: the load fails.

## 4. The engines

| | WebKit | System WebView |
|---|---|---|
| Where it is told | The load's failure, with the URL loading system's error code | `onReceivedSslError`, with the kind of error; the load is cancelled there, as before |
| Not trusted | "untrusted", and "unknown root" | "untrusted" |
| Dates | "bad date", and "not yet valid" | "expired", "not yet valid", "date invalid" |
| Another site's | Not told apart: WebKit says "untrusted" | "id mismatch" |
| The rest | `net.tls` | `net.tls` |

**As built**, and as seen:

- **The System WebView** (133, on the emulator) was served a certificate that signs itself, made on the device by the test: it said "untrusted", the adapter reported `net.tls.untrusted` and cancelled the load, no page of the site came of it, and the site was never asked for its page. (The WebView then commits its own error page at the address that failed, as after any failure, a moment later; the adapter names that address and nothing else. A first form of this test said no address was named at all, and passed only while it looked before that moment.)
- **A server that speaks no TLS**, asked for over `https`, is `net.tls` on both engines. (The test site answers a connection that begins as TLS in plain words and closes it, as a web server does: left waiting instead, WebKit gave up after a minute and called it a timeout.)
- **WebKit** was not served a certificate (§7): its four kinds are the adapter's rule, by their numbers.

## 5. Tests

| Where | What |
|---|---|
| Core (`wt-app/tests/browsing.rs`, 1 more) | A finer code under one the shells know reaches the tab as it was reported: each of the three |
| The adapters' own rules | WebKit (`testEngineErrorsBecomeCodes`): untrusted and unknown root; bad date and not yet valid; and every other way a secure connection is not made, as before. The System WebView (`NetErrorsTest`, 1 more; `PageTrackerTest`, 1 more): untrusted; expired, not yet valid and date invalid; another site's; whatever else, and a kind this build has not heard of; and the load is said to have failed with the kind it was |
| Adapter suites, against the real engines | A server that speaks no TLS, asked for over `https`, is a secure connection that could not be made, and no page of the site comes of it: WebKit on the Mac and the iPhone simulator; the System WebView on the emulator (`SystemWebViewCertificateTest`, 2), where once the engine's own error page is there the load has not finished, no title is the site's, and the only address named is the one that failed. On the emulator, a site whose certificate signs itself, with a key made on the device for the test and removed after it: not trusted, no page of it, and the page never asked for |
| The test site | A connection that begins as TLS is answered in plain words and closed, in all three of its servers |
| The shells' words | Each of the three codes has its words; a finer code with none of its own has the words of the code above it, and with nothing above it the general ones; never the code's own text (`WordsTest`, 1 more; `testACertificateThatIsNotTrustedSaysWhy`). Seven of the rules checked by breaking them |

## 6. Decisions

| # | Decision | Why |
|---|---|---|
| D1 | No way past, for now | It is a promise about safety, and the owner's to make. The failed page as it is makes none |
| D2 | Three kinds, and the rest | What a person can act on differently: the address, the clock, or neither. More kinds would be words nobody can use |
| D3 | The words say what is wrong, not what to fear | A person who knows it is the clock fixes the clock |

## 7. What is not verified

- **A certificate presented to WebKit.** No test on Apple's systems serves one: a test that makes a server's identity on a Mac does it through the system's keychain, which is the person's own and can raise the system's dialogs. WebKit's kinds of error are tested as the adapter's own rule, by their numbers.
- **A certificate whose dates do not hold, or that is another site's, against a real engine.** A certificate a test can make signs itself, and an engine that sees one says "untrusted" first, whatever else is wrong with it.
- **Another site's certificate, on WebKit**: WebKit calls it untrusted, and the words then say less than they could. Telling it apart needs the adapter to look at the certificate itself, which it does not.
- **That the failed page shows the new words in an app.** The words are tested by their codes; no app's test fails a load over a certificate.

## 8. Done checklist

- [x] The core: a code under `net.tls` passes through.
- [x] Android: the adapter's rule; the suite against the real WebView, with a certificate that signs itself; the words.
- [x] Apple: the adapter's rule; the suite against real WebKit (a server that speaks no TLS); the words.
- [x] `Tools/test.sh all` green, and `android-device` (133 on the emulator).
- [ ] Windows shell.
