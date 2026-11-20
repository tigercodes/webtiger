# 021 — A certificate that is not trusted says why (W1)

| | |
|---|---|
| **Status** | Designed. Nothing is built yet: a site whose certificate is not trusted fails with one sentence for every cause, "A secure connection to the site couldn't be made." |
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

What each does in fact, as observed, is written here when built.

## 5. Tests

| Where | What |
|---|---|
| Core (through `FakeShell`) | A code under `net.tls` reaches the tab as it was reported |
| The adapters' own rules | Each engine's kinds of certificate error as the codes of §3, and every other as before |
| Adapter suites, against the real engines | A server that does not speak TLS, asked for over `https`: `net.tls`. On Android, a server whose certificate signs itself, made by the test: `net.tls.untrusted`, and the page is not shown |
| The shells | The words for each code; a code with no words of its own has the words of the code above it |

## 6. Decisions

| # | Decision | Why |
|---|---|---|
| D1 | No way past, for now | It is a promise about safety, and the owner's to make. The failed page as it is makes none |
| D2 | Three kinds, and the rest | What a person can act on differently: the address, the clock, or neither. More kinds would be words nobody can use |
| D3 | The words say what is wrong, not what to fear | A person who knows it is the clock fixes the clock |

## 7. What is not verified

- **A certificate presented to WebKit.** No test on Apple's systems serves one: a test that makes a server's identity on a Mac does it through the system's keychain, which is the person's own and can raise the system's dialogs. WebKit's kinds of error are tested as the adapter's own rule, by their numbers.
- **A certificate whose dates do not hold, or that is another site's, against a real engine.** A certificate a test can make signs itself, and an engine that sees one says "untrusted" first, whatever else is wrong with it.
- The rest: written when built.

## 8. Done checklist

- [ ] The core: a code under `net.tls` passes through.
- [ ] Android: the adapter's rule; the suite against the real WebView; the words.
- [ ] Apple: the adapter's rule; the suite against real WebKit (a server that speaks no TLS); the words.
- [ ] `Tools/test.sh all` green, and `android-device`.
- [ ] Windows shell.
